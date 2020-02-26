#[macro_use]
extern crate diesel;

mod api_fetcher;
mod database;
mod gtfs_data;
mod model;
mod protobuf;
mod schema;

use log::{debug, info};
use serde::Deserialize;
use warp::Filter;

use crate::gtfs_data::{RealtimeQueryKey, RealtimeUpdate, RealtimeUpdateManager};
use chrono::prelude::*;
use database::ConnectionPool;
use dotenv::dotenv;

use std::sync::{Arc, Mutex};

#[derive(Deserialize, Debug)]
struct StopTimesParams {
    range_start_mins: Option<u32>,
    range_end_mins: Option<u32>,
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // IMPORTANT
    env_logger::init();

    println!("Starting web server");

    let pool = database::create_connection_pool();
    info!("Created database connection pool");

    // pass in a database connection pool
    let data = warp::any().map(move || pool.clone());

    let realtime_manager = RealtimeUpdateManager::new();
    let arc_mutex = Arc::new(Mutex::new(realtime_manager));
    let arc_mutex_clone = arc_mutex.clone();

    let rt_filter = warp::any().map(move || arc_mutex_clone.clone());

    // stop/{code}/..
    let stop = warp::any()
        .and(data)
        .and(rt_filter)
        .and(warp::path!("stop" / String / ..));

    // stop/{code}/times
    let times = stop
        .and(warp::path("times"))
        .and(warp::query::query()) // fetch query parameters from url
        .and_then(fetch_stop_times);

    futures::future::join(
        warp::serve(times).run(([127, 0, 0, 1], 6789)),
        api_fetcher::fetch_data(arc_mutex.clone()),
    )
    .await;
}

#[derive(Debug)]
enum ServerError {
    DbError(diesel::result::Error),
    TokioError(tokio::task::JoinError),
}
impl warp::reject::Reject for ServerError {}

impl Into<warp::reject::Rejection> for ServerError {
    fn into(self) -> warp::reject::Rejection {
        warp::reject::custom(self)
    }
}

async fn fetch_stop_times(
    pool: ConnectionPool,
    realtime_manager: Arc<Mutex<RealtimeUpdateManager>>,
    stop_code: String,
    params: StopTimesParams,
) -> Result<warp::reply::Json, warp::Rejection> {
    let connection = pool.get().unwrap();
    use diesel::pg::types::sql_types::Timestamptz;
    use diesel::prelude::*;
    use diesel::sql_types::Text;

    let now = chrono::Utc::now();
    // TODO 30 is a magic value to handle delays
    let a = now - chrono::Duration::minutes(params.range_start_mins.unwrap_or(2).into());
    let b = now + chrono::Duration::minutes(params.range_end_mins.unwrap_or(720).into());
    debug!("now: {}, from -{} to {}", now, a, b);

    let x: Vec<model::StopTimeByStop> = tokio::task::spawn_blocking(move || {
        let r = diesel::sql_query(include_str!("sql_queries/stop_times.sql"))
            .bind::<Timestamptz, _>(a - chrono::Duration::minutes(30))
            .bind::<Timestamptz, _>(b)
            .bind::<Text, _>(stop_code)
            .load(&connection)
            .map_err(|e| warp::reject::custom(ServerError::DbError(e)))?;
        Ok::<_, warp::reject::Rejection>(r)
    }).await.map_err(|e| warp::reject::custom(ServerError::TokioError(e)))??;

    let realtime = (*realtime_manager.lock().unwrap()).get_realtime_updates(x.iter().map(|y| {
        RealtimeQueryKey {
            start_date: y.service_date,
            trip_id: &y.trip_id,
            stop_sequence: y.stop_sequence as u32, // this should be a positive integer
        }
    }));

    #[derive(serde::Serialize, Debug)]
    struct T {
        base: model::StopTimeByStop,
        realtime: Option<CombinedRealtimeUpdate>,
    }

    #[derive(serde::Serialize, Debug)]
    struct R {
        // for client to get accurate UTC time
        current_time: DateTime<Utc>,
        trips: Vec<T>,
    }
    #[derive(serde::Serialize, Debug)]
    struct CombinedRealtimeUpdate {
        departure_time: DateTime<Utc>,
        #[serde(flatten)]
        realtime_update: RealtimeUpdate,
    }

    let trips = x
        .into_iter()
        .zip(realtime)
        .filter_map(|(base, realtime)| match realtime {
            Some(realtime) => {
                let departure_time = base.departure_time
                    + chrono::Duration::seconds(realtime.delay.unwrap_or(0) as i64);
                if departure_time > b || departure_time < a {
                    return None;
                }
                Some(T {
                    base,
                    realtime: Some(CombinedRealtimeUpdate {
                        realtime_update: realtime,
                        departure_time,
                    }),
                })
            }
            None => {
                if base.departure_time > b || base.departure_time < a {
                    return None;
                }
                Some(T {
                    base,
                    realtime: None,
                })
            }
        })
        .collect::<Vec<T>>();

    Ok(warp::reply::json(&R {
        current_time: now,
        trips,
    }))
}
