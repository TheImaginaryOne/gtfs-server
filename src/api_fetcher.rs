use derive_more::{Display, From};
use futures::future::FutureExt;
use log::{debug, error, info};
use prost::Message;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::gtfs_data::RealtimeUpdateManager;
use crate::protobuf::gtfs_realtime::FeedMessage;

#[derive(serde::Deserialize)]
struct UrlConfig {
    url: String,
    header: HashMap<String, String>,
}

#[derive(From, Display)]
enum RealtimeApiError {
    Reqwest(reqwest::Error),
    Decode(prost::DecodeError),
    Tokio(tokio::task::JoinError),
    Io(std::io::Error),
    ParseUrlConfig(toml::de::Error),
}

async fn get_realtime_feed_config(path: &str) -> Result<UrlConfig, RealtimeApiError> {
    let s = tokio::fs::read_to_string(path).await?;

    let config = toml::from_str(&s)?;

    Ok(config)
}

pub async fn fetch_data(realtime_manager: Arc<Mutex<RealtimeUpdateManager>>) {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

    let realtime_config_filepath = std::env::var("REALTIME_CONFIG_FILEPATH")
        .expect("REALTIME_CONFIG_FILEPATH must be defined");

    interval.tick().await; // 0 second tick

    loop {
        let config = match get_realtime_feed_config(&realtime_config_filepath).await {
            Ok(c) => c,
            Err(e) => {
                error!("Error fetching gtfs realtime configs: {}", e);
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };

        let timer_future = interval.tick().fuse();
        let c = client.clone();
        let rm = realtime_manager.clone();
        let request_future = async move {
            match send_request(&c, &config).await {
                Ok(feed) => {
                    debug!("Fetched {} entities from {}", feed.entity.len(), config.url);
                    (*rm.lock().unwrap()).load_feed(feed);
                }
                Err(e) => {
                    error!("Error fetching gtfs data: {}", e);
                }
            }
        }
        .fuse();

        futures::pin_mut!(timer_future, request_future);

        info!("Fetching gtfs realtime data");

        #[allow(clippy::unnecessary_mut_passed)]
        loop {
            futures::select! {
                response = request_future => (),
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
