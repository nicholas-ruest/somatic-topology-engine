#!/usr/bin/env bash
set -euo pipefail
project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.."&&pwd);cd "$project_root"
cargo test -p ste-device-interaction --all-targets
cargo test -p ste-cli --test device_simulator_commands
cargo clippy -p ste-device-interaction -p ste-cli --all-targets -- -D warnings
output="$project_root/target/validation/phase-14-device-simulator-host.txt";mkdir -p "$(dirname "$output")";{ rustc --version;cargo run --release --quiet -p ste-cli --example device_simulator_benchmark;}>"$output"
grep -q '^physical_crowpi_status=unqualified_pending_revision_hil$' "$output";echo "phase 14 validation passed; simulator evidence: target/validation/phase-14-device-simulator-host.txt"
