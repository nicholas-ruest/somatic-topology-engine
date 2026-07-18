#!/usr/bin/env bash
set -euo pipefail
project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.."&&pwd);cd "$project_root"
cargo test -p ste-personalization-memory --all-targets
cargo test -p ste-cli --test memory_commands
cargo clippy -p ste-personalization-memory -p ste-cli --all-targets -- -D warnings
output="$project_root/target/validation/phase-13-vector-memory-host.txt";mkdir -p "$(dirname "$output")";{ rustc --version;cargo run --release --quiet -p ste-cli --example vector_memory_benchmark;}>"$output"
grep -q '^ruvector_rvf_status=unverified_not_integrated$' "$output";grep -q '^cryptographic_erasure_rebuild=passed$' "$output";grep -q '^reference_pi_status=pending$' "$output";echo "phase 13 validation passed; host evidence: target/validation/phase-13-vector-memory-host.txt"
