use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Infura,
    Llama,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub is_test: bool,
    pub provider: Provider,
    pub api_key: Option<String>,
    pub abi_json_path: String,
    pub tx_retry_times: u64,
    pub tx_retry_period: u64,
    pub api_server_address: String,
}

pub fn read_config(path: impl AsRef<std::path::Path>) -> Config {
    serde_yaml::from_reader(BufReader::new(File::open(path).unwrap())).unwrap()
}
