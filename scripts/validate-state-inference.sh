#!/usr/bin/env bash
set -euo pipefail
project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd); cd "$project_root"
cargo test -p ste-state-inference --all-targets
cargo test -p ste-cli --test state_projection_commands
cargo clippy -p ste-state-inference -p ste-cli --all-targets -- -D warnings
output_dir="$project_root/target/validation"; mkdir -p "$output_dir"
{ rustc --version; cargo run --release --quiet -p ste-cli --example state_projection_benchmark; } > "$output_dir/phase-12-state-inference-host.txt"
grep -q '^trained_cognitive_model=absent$' "$output_dir/phase-12-state-inference-host.txt"
grep -q '^unsupported_constructs=disabled$' "$output_dir/phase-12-state-inference-host.txt"
grep -q '^raw_output_leakage=none_detected$' "$output_dir/phase-12-state-inference-host.txt"
grep -q '^reference_pi_status=pending$' "$output_dir/phase-12-state-inference-host.txt"
echo "phase 12 validation passed; host evidence: target/validation/phase-12-state-inference-host.txt"
