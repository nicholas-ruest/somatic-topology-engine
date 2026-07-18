#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cargo test --manifest-path "$root/Cargo.toml" --locked -p ste-observability --all-targets
cargo test --manifest-path "$root/Cargo.toml" --locked -p ste-cli --test diagnostics_commands
bash "$root/tests/phase03/run-fault-matrix.sh"
bash "$root/tests/phase05/run-storage-fault-matrix.sh"
cargo test --manifest-path "$root/Cargo.toml" --locked -p ste-radio-acquisition --test live_adapter
echo "Phase 08 observability/fault matrix passed"
