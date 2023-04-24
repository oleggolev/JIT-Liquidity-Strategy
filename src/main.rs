mod config;
mod feed;
mod server;

use crate::config::read_config;
use crate::server::{run_server, DataPoint};
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

    tokio::spawn(async move {
        let feed_thread = feed::start(config, data.clone()).await;
        let server_thread = run_server(data.clone());

        tokio_join!(feed_thread, server_thread);
    });

    thread::park();
}
