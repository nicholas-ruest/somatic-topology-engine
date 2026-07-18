#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
bash scripts/test-architecture.sh
bash scripts/check-unsafe.sh
bash scripts/check-assurance-controls.sh
bash scripts/check-secrets.sh
