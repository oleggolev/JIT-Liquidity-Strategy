use std::{fs, sync::Arc};

const ERC20_ABI_FILENAME: &str = "ERC20.json";
const UNISWAP_V2_ABI_FILENAME: &str = "UniswapV2.json";
const UNISWAP_V2_ABI_FACTORY_FILENAME: &str = "UniswapV2Factory.json";

#[derive(Debug, Clone)]
pub struct Abi {
    pub uniswap_v2_abi: Arc<ethereum_abi::Abi>,
    pub erc20_token_abi: ethers_core::abi::Abi,
    pub uniswap_v2_factory_abi: ethers_core::abi::Abi,
}

impl Abi {
    pub fn new(base_path: String) -> Self {
        // Create an ABI to decode UniswapV2 transaction input.
        let uniswap_v2_abi_json =
            fs::read_to_string(base_path.clone() + "/" + UNISWAP_V2_ABI_FILENAME)
                .unwrap_or_else(|_| panic!("Cannot read {UNISWAP_V2_ABI_FILENAME}"));
        let uniswap_v2_abi: ethereum_abi::Abi = serde_json::from_str(&uniswap_v2_abi_json).unwrap();
        let uniswap_v2_abi = Arc::new(uniswap_v2_abi);

        // Create an ABI for ERC20 tokens to call their contract methods.
        let erc20_token_abi_json = fs::read_to_string(base_path.clone() + "/" + ERC20_ABI_FILENAME)
            .unwrap_or_else(|_| panic!("Cannot read {ERC20_ABI_FILENAME}"));
        let erc20_token_abi: ethers_core::abi::Abi =
            serde_json::from_str(&erc20_token_abi_json).unwrap();

        // Create an ABI for UniswapV2 factory requests.
        let uniswap_v2_factory_abi_json =
            fs::read_to_string(base_path + "/" + UNISWAP_V2_ABI_FACTORY_FILENAME)
                .unwrap_or_else(|_| panic!("Cannot read {ERC20_ABI_FILENAME}"));
        let uniswap_v2_factory_abi: ethers_core::abi::Abi =
            serde_json::from_str(&uniswap_v2_factory_abi_json).unwrap();

        Abi {
            uniswap_v2_abi,
            erc20_token_abi,
            uniswap_v2_factory_abi,
        }
    }
}
