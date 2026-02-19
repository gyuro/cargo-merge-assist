# Contributing

## Local quality checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --all-targets
```

## Manual smoke test

```bash
# Build
cargo build

# Help
cargo run -- --help

# Merge command help
cargo run -- merge-manifest --help
```

## Release checklist

1. Update version in `Cargo.toml`
2. Run full checks locally
3. Tag release (`git tag vX.Y.Z`)
4. Push commits + tag
5. (Optional) `cargo publish` when crate metadata is finalized
