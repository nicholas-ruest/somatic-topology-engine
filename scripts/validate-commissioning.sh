#!/usr/bin/env bash
set -euo pipefail
project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.."&&pwd);cd "$project_root"
cargo test -p ste-commissioning --all-targets
cargo test -p ste-cli --test operator_commands
cargo clippy -p ste-commissioning -p ste-cli --all-targets -- -D warnings
output="$project_root/target/validation/phase-15-commissioning-host.txt";mkdir -p "$(dirname "$output")";{ rustc --version;cargo run --release --quiet -p ste-cli --example commissioning_benchmark;}>"$output"
grep -q '^threshold_weakening=prohibited$' "$output";grep -q '^physical_site_qualification=not_performed$' "$output";grep -q '^reference_pi_status=pending$' "$output";echo "phase 15 validation passed; synthetic evidence: target/validation/phase-15-commissioning-host.txt"
