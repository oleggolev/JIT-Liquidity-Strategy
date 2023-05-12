use ethers_core::abi::Abi;
use serde::Deserialize;
use std::{fs, sync::Arc};

use crate::config::UniswapVersion;

const ERC20_ABI_FILENAME: &str = "ERC20.json";
const UNISWAP_V2_ROUTER_ABI_FILENAME: &str = "UniswapV2Router.json";
const UNISWAP_V2_FACTORY_ABI_FILENAME: &str = "UniswapV2Factory.json";
const UNISWAP_V3_ROUTER_ABI_FILENAME: &str = "UniswapV3Router.json";
const UNISWAP_V3_FACTORY_ABI_FILENAME: &str = "UniswapV3Factory.json";

#[derive(Debug, Clone)]
pub struct AbiWrapper {
    pub erc20_token_abi: Abi,
    pub uniswap_v2_router_abi: Abi,
    pub uniswap_v2_factory_abi: Abi,
    pub tx_input_decoder: Arc<ethereum_abi::Abi>,
}

impl AbiWrapper {
    pub fn new(base_path: String, uniswap_version: UniswapVersion) -> Self {
        let (router_path, factory_path) = match uniswap_version {
            UniswapVersion::Two => (
                UNISWAP_V2_ROUTER_ABI_FILENAME,
                UNISWAP_V2_FACTORY_ABI_FILENAME,
            ),
            UniswapVersion::Three => (
                UNISWAP_V3_ROUTER_ABI_FILENAME,
                UNISWAP_V3_FACTORY_ABI_FILENAME,
            ),
        };
        AbiWrapper {
            erc20_token_abi: Self::read_abi(base_path.clone(), ERC20_ABI_FILENAME),
            uniswap_v2_router_abi: Self::read_abi(base_path.clone(), router_path),
            uniswap_v2_factory_abi: Self::read_abi(base_path.clone(), factory_path),
            tx_input_decoder: Arc::new(Self::read_abi(base_path, router_path)),
        }
    }

    fn read_abi<T: for<'a> Deserialize<'a>>(base_path: String, filename: &str) -> T {
        let json = fs::read_to_string(base_path + "/" + filename)
            .unwrap_or_else(|_| panic!("Cannot read {filename}"));
        serde_json::from_str(&json).unwrap()
    }
}
