use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Infura,
    Llama,
}

#[derive(Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum UniswapVersion {
    Two = 2,
    Three = 3,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub provider: Provider,
    pub api_key: Option<String>,
    pub abi_json_path: String,
    pub tx_retry_times: u64,
    pub tx_retry_period: u64,
    pub api_server_address: String,
    pub uniswap_version: UniswapVersion,
}

pub fn read_config(path: impl AsRef<std::path::Path>) -> Config {
    serde_yaml::from_reader(BufReader::new(File::open(path).unwrap())).unwrap()
}
