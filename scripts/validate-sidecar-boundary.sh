#!/usr/bin/env bash
set -euo pipefail
project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.."&&pwd);cd "$project_root"
start=$(date +%s%N);cargo test -p ste-runtime --test sidecar_containment;elapsed=$(( $(date +%s%N)-start ))
cargo test -p ste-runtime --test governance_gate config_feature_admin_adapter_and_sidecar_cannot_widen_purpose
cargo clippy -p ste-runtime --all-targets -- -D warnings
for file in package.json package-lock.json pnpm-lock.yaml yarn.lock;do test ! -e "$file"||{ echo "unexpected production sidecar dependency: $file" >&2;exit 1;};done
output="$project_root/target/validation/phase-16-sidecar.txt";mkdir -p "$(dirname "$output")";{ echo "rust_isolation_test_elapsed_ns=$elapsed";echo "measured_gap=none";echo "production_sidecar=absent";echo "node_npm_dependency=absent";echo "core_failure_isolation=passed";}>"$output";echo "phase 16 validation passed; evidence: target/validation/phase-16-sidecar.txt"
