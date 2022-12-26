# new-bid-handler

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
