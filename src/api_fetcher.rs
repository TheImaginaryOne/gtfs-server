use derive_more::{Display, From};
use diesel::prelude::*;
use futures::future::FutureExt;
use log::{debug, error, info};
use prost::Message;
use std::collections::HashMap;

use crate::database::ConnectionPool;
use crate::model::RealtimeFeed;
use crate::protobuf::gtfs_realtime::FeedMessage;

#[derive(serde::Deserialize)]
struct UrlConfig {
    url: String,
    header: HashMap<String, String>,
}

#[derive(From, Display)]
enum RealtimeApiError {
    ReqwestError(reqwest::Error),
    DecodeError(prost::DecodeError),
    TokioError(tokio::task::JoinError),
    DieselError(diesel::result::Error),
    ParseUrlConfigError(toml::de::Error),
}

async fn get_realtime_feed_configs(
    pool: ConnectionPool,
) -> Result<HashMap<String, UrlConfig>, RealtimeApiError> {
    use crate::schema::realtime_feed::dsl::*;

    let results: Vec<RealtimeFeed> = tokio::task::spawn_blocking(move || {
        realtime_feed.load::<RealtimeFeed>(&pool.get().unwrap())
    })
    .await??;

    let configs: HashMap<String, UrlConfig> = results
        .into_iter()
        .map(|r| Ok((r.region_id, toml::from_str(&r.url_config)?)))
        .collect::<Result<_, RealtimeApiError>>()?;

    debug!("Found {} realtime feed configs", configs.len());

    Ok(configs)
}

pub async fn fetch_data(pool: ConnectionPool) {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

    interval.tick().await; // 0 second tick

    loop {
        let configs = match get_realtime_feed_configs(pool.clone()).await {
            Ok(c) => c,
            Err(e) => {
                error!("Error fetching gtfs realtime configs: {}", e);
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };

        let timer_future = interval.tick().fuse();
        let request_futures = futures::future::join_all(
            configs
                .into_iter()
                .map(|(k, config)| {
                    let client = &client;
                    async move {
                        match send_request(client, &config).await {
                            Ok(feed) => {
                                debug!(
                                    "Region {}: fetched {} entities from {}",
                                    k,
                                    feed.entity.len(),
                                    config.url
                                );
                            }
                            Err(e) => {
                                error!("Region {}: Error fetching gtfs data: {}", k, e);
                            }
                        }
                    }
                })
                .collect::<Vec<_>>(),
        )
        .fuse();

        futures::pin_mut!(timer_future, request_futures);

        info!("Fetching gtfs realtime data");

        loop {
            futures::select! {
                response = request_futures => (),
                t = timer_future => break
            };
        }
    }
}

async fn send_request(
    client: &reqwest::Client,
    url_config: &UrlConfig,
) -> Result<FeedMessage, RealtimeApiError> {
    let mut builder = client.get(&url_config.url);

    for (k, v) in url_config.header.iter() {
        builder = builder.header(k, v);
    }

    let bytes = builder.send().await?.bytes().await?;

    let message = FeedMessage::decode(bytes)?;
    Ok(message)
}
