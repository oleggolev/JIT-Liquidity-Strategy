mod config;
mod simulation;

use crate::config::read_config;
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
    let ganache_ws_endpoint = simulation::start(config).await;
    println!("Ganache blockchain simulation node started at: {ganache_ws_endpoint}");
    thread::park();
}
