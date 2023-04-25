mod config;
mod feed;
mod server;

use crate::config::read_config;
use crate::server::DataPoint;
use std::sync::{Arc, Mutex};
use std::thread;

const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[tokio::main]
async fn main() {
    let config = read_config(
        std::env::args()
            .collect::<Vec<String>>()
            .get(1)
            .unwrap_or(&DEFAULT_CONFIG_PATH.to_owned()),
    );

    let data = Arc::new(Mutex::new(Vec::<DataPoint>::new()));
    let server_data = Arc::clone(&data);
    tokio::spawn(async move {
        server::start(server_data);
    });
    let feed_data = Arc::clone(&data);
    tokio::spawn(async move {
        feed::start(config, feed_data).await;
    });

    thread::park();
}
