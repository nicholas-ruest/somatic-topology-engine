#!/usr/bin/env bash
set -euo pipefail
project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.."&&pwd);cd "$project_root"
cargo test -p ste-runtime --test slo_faults --test ipc_security --test supervision
cargo test -p ste-storage --test journal_recovery
cargo test -p ste-model-runtime --test package_lifecycle
cargo test -p ste-release-hardening --all-targets
bash scripts/check-assurance-controls.sh
bash scripts/test-architecture.sh
cargo clippy --workspace --all-targets -- -D warnings
output_dir="$project_root/target/validation/phase17-benchmarks"
STE_SUITE_OUTPUT_DIR="$output_dir" bash scripts/benchmark-suite.sh
jq -e '.passed == true' "$output_dir/self-comparison.json" >/dev/null
jq -e '.pending.reference_pi_temperature and .pending.reference_pi_power and .pending.reference_pi_soak' "$output_dir/baseline.json" >/dev/null
output="$project_root/target/validation/phase-17-hardening-status.json"
jq -n --sort-keys --arg host "$(rustc -vV|sed -n 's/^host: //p')" '{schema:"ste-phase17-status-v1",host:$host,current_host_regression_harness:"passed",synthetic_fault_matrix:"passed",optimization:"none-no-measured-blocking-bottleneck",reference_pi_hil:"pending",reference_pi_thermal_power:"pending",multi_day_soak:"pending",penetration_test:"pending",fuzz_campaigns:"pending",incident_exercise:"pending",production_release_gate:"blocked"}' > "$output"
echo "phase 17 current-host validation passed; production gates remain blocked: $output"
