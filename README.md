# JIT-Liquidity-Strategy
Creating a Just-in-time (“JIT”) liquidity service on top of the Ethereum pending transaction stream.

## Config

The JIT service consumes a `config.yaml` file which specified its runtime parameters. The service first establishes a connection to the Ethereum mainnet or Goerli testnet in order to monitor the pending Ethereum transactions pool. We watch for swap transactions and forward them to our local Ganache node where we perform a JIT attack without monetary limitations. To be able to launch a local Ganache node, install the Ganache CLI with `node` (>= v14.0.0)`npm` (>= 6.0.0) on your Linux / MacOS machine.

To update `node` to stable version (v16.15.0):
```bash
sudo npm install -g n
sudo n 16.15.0
sudo n prune
```

To update `npm`:
```bash
sudo npm update -g
```

To install ganache (stable version 7.4.0):
```bash
sudo chown -R $USER /usr/local/lib/node_modules
npm install -g ganache@7.4.0
```

Example `config.yaml` file:
```yaml
is_test: true
provider: infura
api_key: <INSERT_YOUR_KEY_HERE>
block_time: 12
tx_retry_times: 5
tx_retry_interval: 100
```
```
block_time is in seconds.
tx_retry_interval is in milliseconds.
```

## Run

```bash
cargo run <optional: /path/to/config.yaml>
```

## Important links
- [Ethereum RPC node providers](https://ethereumnodes.com/)
- [Using a Ganache instance with Rust](https://github.com/cl2089/rust-ethereum-tutorial/blob/9de183ee48f25b3fef9f506a4575726061df710c/src/simple_transactions.rs)
- [How to connect to a WebSocket provider](https://github.com/gakonst/ethers-rs/blob/7e7f9041b3f5a601a8fca8ccbce0287518f8cc33/book/providers/ws.md)
- [Uniswap v3 JIT Liquidity MEV Transactions] (https://dune.com/embeds/233623/437572/9faacc71-4cf9-40c9-be35-985f78d0d00c)
- [Uniswap v3 JIT Liquidity MEV Transactions Cost] (https://dune.com/embeds/233623/438791/46e30aae-9af5-4ff3-a0d7-f9754d356e5b)
- [Simple JIT Math] (https://medium.com/virtuswap/dissecting-the-jit-liquidity-attack-b283504e0de7)
- [Comprehensive Guide on JIT] (https://uniswap.org/blog/jit-liquidity)
- [Flashbot bundles js] (https://docs.flashbots.net/flashbots-auction/searchers/advanced/understanding-bundles)
- [Flashbot bundles rs] (https://crates.io/crates/ethers-flashbots)
- [Removing liquidity example] (https://github.com/gakonst/ethers-rs/blob/10310ce3ad8562476196e9ec06f78c2a27417739/examples/transactions/examples/remove_liquidity.rs)


## TODO:
Fork the Ethereum Mainnet with Ganache to interact with a snapshot of the Mainnet on our local Ganache instance. This will replicate all transactions and liquidity pool contracts (hopefully) that are available. Generate fake transactions for testing for inclusion into newly locally mined blocks. For each transaction, if it meets the necessary conditions for a JIT attack, use Flashbots middleware to bundle the transaction alongside liquidity provision and removal transactions. Measure fees required against the profit based on the size of the transaction. Optimize when JIT transactions should be executed. If there is time, we can add hedging to account for possible losses and miscalculations.
