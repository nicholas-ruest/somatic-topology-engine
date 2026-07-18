#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$project_root"
cargo test -p ste-physiology-estimation --all-targets
cargo test -p ste-cli --test respiration_commands
cargo clippy -p ste-physiology-estimation -p ste-cli --all-targets -- -D warnings

output_dir="$project_root/target/validation"
mkdir -p "$output_dir"
{
  rustc --version
  cargo run --release --quiet -p ste-cli --example respiration_benchmark
} > "$output_dir/phase-10-respiration-host.txt"
grep -q '^reference_pi_status=pending$' "$output_dir/phase-10-respiration-host.txt"
grep -q '^human_reference_status=pending$' "$output_dir/phase-10-respiration-host.txt"
grep -q '^capability_status=disabled_not_promoted$' "$output_dir/phase-10-respiration-host.txt"
echo "phase 10 validation passed; host benchmark: target/validation/phase-10-respiration-host.txt"
