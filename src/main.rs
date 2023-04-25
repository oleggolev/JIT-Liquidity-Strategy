mod config;
mod datapoint;
mod feed;
mod server;

use datapoint::DataPoint;

use crate::config::read_config;
use std::sync::{Arc, Mutex};
use std::{net, thread};

const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[tokio::main]
async fn main() {
    let config = read_config(
        std::env::args()
            .collect::<Vec<String>>()
            .get(1)
            .unwrap_or(&DEFAULT_CONFIG_PATH.to_owned()),
    );

    // This data is shared across threads.
    let data = Arc::new(Mutex::new(Vec::<DataPoint>::new()));

    // Start up the API server.
    server::Server::start(
        config
            .api_server_address
            .parse::<net::SocketAddr>()
            .expect("Error parsing API server address"),
        &data,
    );

    // Begin the data feed from an external provider.
    let server_data = Arc::clone(&data);
    tokio::spawn(async move {
        feed::start(config, server_data).await;
    });

    thread::park();
}
