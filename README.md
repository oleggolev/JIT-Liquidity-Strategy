# JIT-Liquidity-Strategy
Creating a Just-in-time (“JIT”) liquidity service on top of the Ethereum pending transaction stream.

## Config

The JIT service consumes a `config.yaml` file which specified its runtime parameters. The service first establishes a connection to the Ethereum mainnet in order to monitor the pending Ethereum transactions pool. This can be done via middleware like Infura or LlamaNodes or using a local Ganache node. To be able to do the latter, install the Ganache CLI with `node` (>= v14.0.0)`npm` (>= 6.0.0) on your Linux / MacOS machine.

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
provider: infura
api_key: <INSERT_YOUR_KEY_HERE>
```

## Run

```bash
cargo run <optional: /path/to/config.yaml>
```

## Important links
- [Ethereum RPC node providers](https://ethereumnodes.com/)
- [Using a Ganache instance with Rust](https://github.com/cl2089/rust-ethereum-tutorial/blob/9de183ee48f25b3fef9f506a4575726061df710c/src/simple_transactions.rs)
- [How to connect to a WebSocket provider](https://github.com/gakonst/ethers-rs/blob/7e7f9041b3f5a601a8fca8ccbce0287518f8cc33/book/providers/ws.md)
