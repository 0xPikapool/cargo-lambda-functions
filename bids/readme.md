# bids

Entrypoint for new bids into the Pikapool mempool

- Validates correctness of the Bid EIP712 TypedData
- Validates the Bid signature
- Looks up whether the signer has approved enough WETH and has enough WETH balance (using in-memory cached data from chain-state-service)
- Looks up whether the auction exists and is open to bids (using in-memory cached data from chain-state-service)
- Finally, adds Bid to the mempool

## Install

[See installation instructions for you OS](https://www.cargo-lambda.info/guide/installation.html)

## Development

```bash
cargo lambda watch
```

## Test

```bash
cargo test
```

## Configure

See `[package.metadata.lambda.deploy]` in `Cargo.toml`

## Release

1. [Install AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html)

2. Run `aws configure`

3. Build for Graviton `cargo lambda build --release --arm64`

4. Deploy `cargo lambda deploy --enable-function-url`
