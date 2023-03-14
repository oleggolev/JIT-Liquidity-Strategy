use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    LlamaNodes,
    Infura,
    Ganache,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub is_test: bool,
    pub provider: Provider,
    pub api_key: Option<String>,
}

pub fn read_config(path: impl AsRef<std::path::Path>) -> Config {
    serde_yaml::from_reader(BufReader::new(File::open(path).unwrap())).unwrap()
}
