#[macro_use]
extern crate diesel;

mod api_fetcher;
mod database;
mod model;
mod protobuf;
mod schema;

use log::{debug, info};
use serde::Deserialize;
use warp::Filter;

use database::ConnectionPool;
use dotenv::dotenv;

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

    let cloned_pool = pool.clone();

    // pass in a database connection pool
    let data = warp::any().map(move || pool.clone());

    // stop/{code}/..
    let stop = warp::any().and(data).and(warp::path!("stop" / String / ..));

    // stop/{code}/times
    let times = stop
        .and(warp::path("times"))
        .and(warp::query::query()) // fetch query parameters from url
        .and_then(fetch_stop_times);

    futures::future::join(
        warp::serve(times).run(([127, 0, 0, 1], 6789)),
        api_fetcher::fetch_data(cloned_pool),
    )
    .await;
}

#[derive(Debug)]
enum ServerError {
    DbError(diesel::result::Error),
}
impl warp::reject::Reject for ServerError {}

async fn fetch_stop_times(
    pool: ConnectionPool,
    stop_code: String,
    params: StopTimesParams,
) -> Result<warp::reply::Json, warp::Rejection> {
    let connection = pool.get().unwrap();
    use diesel::pg::types::sql_types::Timestamptz;
    use diesel::prelude::*;
    use diesel::sql_types::Text;

    let now = chrono::Utc::now();
    let a = now - chrono::Duration::minutes(params.range_start_mins.unwrap_or(2).into());
    let b = now + chrono::Duration::minutes(params.range_end_mins.unwrap_or(720).into());
    debug!("now: {}, from -{} to {}", now, a, b);

    let x: Vec<model::StopTimeByStop> =
        diesel::sql_query(include_str!("sql_queries/stop_times.sql"))
            .bind::<Timestamptz, _>(a)
            .bind::<Timestamptz, _>(b)
            .bind::<Text, _>(stop_code)
            .load(&connection)
            .map_err(|e| warp::reject::custom(ServerError::DbError(e)))?;
    #[derive(serde::Serialize, Debug)]
    struct R {
        trips: Vec<model::StopTimeByStop>,
    }

    Ok(warp::reply::json(&R { trips: x }))
}
