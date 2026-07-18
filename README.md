# Somatic Topology Engine

Somatic Topology Engine (STE) is a local-first, Rust-first platform for
consent-gated radio acquisition, signal observation, conservative physiology
estimation, state inference, personalization, and deterministic device
interaction.

The system is under active implementation. It is research infrastructure, not
a medical device, and its outputs must not be interpreted as diagnoses or
clinical advice.

## Build and verify

Install the Rust toolchain declared in `rust-toolchain.toml`, then run:

```sh
cargo build --workspace --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
bash scripts/verify.sh
```

The architecture and implementation sequence are documented in
[`docs/architecture`](docs/architecture),
[`docs/ddd`](docs/ddd), and
[`docs/implementation-prompts.md`](docs/implementation-prompts.md).

## License

Licensed under the [MIT License](LICENSE).
