mod config;
mod proxy;

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
    tokio::spawn(async move {
        proxy::start(config).await;
    });

    thread::park();
}
