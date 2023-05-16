# JIT-Liquidity-Strategy
Building a Just-in-time (“JIT”) liquidity bot on top of the Ethereum pending transaction stream.

## Config

The JIT service consumes a `config.yaml` file which specified its runtime parameters. The service establishes a connection to the Ethereum mainnet in order to monitor the pending Ethereum transactions pool. We watch for Uniswap V2 transactions and use the Web3 API with a local Anvil node and an external mainnet provider to extract all information necessary to determine whether performing a JIT attack is desirable.

To install Anvil, enter the following in the terminal window:
```bash
curl -L https://foundry.paradigm.xyz | bash
brew install libusb
foundaryup
```

Example `config.yaml` file:
```yaml
provider: infura
api_key: <YOUR INFURA KEY HERE>
abi_json_path: ./abi
tx_retry_times: 2
tx_retry_period: 1000
api_server_address: 127.0.0.1:8000
```
`tx_retry_interval` is in milliseconds.


## Run

```bash
cargo run <optional: /path/to/config.yaml>
```

## Important links
- [Ethereum RPC node providers](https://ethereumnodes.com/)
- [Using a Ganache instance with Rust](https://github.com/cl2089/rust-ethereum-tutorial/blob/9de183ee48f25b3fef9f506a4575726061df710c/src/simple_transactions.rs)
- [How to connect to a WebSocket provider](https://github.com/gakonst/ethers-rs/blob/7e7f9041b3f5a601a8fca8ccbce0287518f8cc33/book/providers/ws.md)
- [Uniswap v3 JIT Liquidity MEV Transactions](https://dune.com/embeds/233623/437572/9faacc71-4cf9-40c9-be35-985f78d0d00c)
- [Uniswap v3 JIT Liquidity MEV Transactions Cost](https://dune.com/embeds/233623/438791/46e30aae-9af5-4ff3-a0d7-f9754d356e5b)
- [Simple JIT Math](https://medium.com/virtuswap/dissecting-the-jit-liquidity-attack-b283504e0de7)
- [Comprehensive Guide on JIT](https://uniswap.org/blog/jit-liquidity)
- [Flashbot bundles js](https://docs.flashbots.net/flashbots-auction/searchers/advanced/understanding-bundles)
- [Flashbot bundles rs](https://crates.io/crates/ethers-flashbots)
- [Removing liquidity example](https://github.com/gakonst/ethers-rs/blob/10310ce3ad8562476196e9ec06f78c2a27417739/examples/transactions/examples/remove_liquidity.rs)
