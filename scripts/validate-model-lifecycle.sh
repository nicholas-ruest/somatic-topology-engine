#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$project_root"
cargo test -p ste-model-runtime --all-targets
cargo test -p ste-cli --test model_lifecycle_commands --test model_package_fixtures
cargo clippy -p ste-model-runtime -p ste-cli --all-targets -- -D warnings

output_dir="$project_root/target/validation"
mkdir -p "$output_dir"
{
  rustc --version
  cargo run --release --quiet -p ste-cli --example model_lifecycle_benchmark
} > "$output_dir/phase-11-model-lifecycle-host.txt"
grep -q '^rollback_kat=passed$' "$output_dir/phase-11-model-lifecycle-host.txt"
grep -q '^reference_pi_status=pending$' "$output_dir/phase-11-model-lifecycle-host.txt"
echo "phase 11 validation passed; host benchmark: target/validation/phase-11-model-lifecycle-host.txt"
