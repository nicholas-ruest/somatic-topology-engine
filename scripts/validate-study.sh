#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$project_root"

cargo test -p ste-experiment-validation --all-targets
cargo test -p ste-cli --test validation_commands
cargo clippy -p ste-experiment-validation -p ste-cli --all-targets -- -D warnings

report_dir="$project_root/target/validation"
mkdir -p "$report_dir"
cargo run --quiet -p ste-cli --example validation_report > "$report_dir/phase-09-validation.first"
cargo run --quiet -p ste-cli --example validation_report > "$report_dir/phase-09-validation.second"
cmp "$report_dir/phase-09-validation.first" "$report_dir/phase-09-validation.second"
cp "$report_dir/phase-09-validation.first" "$report_dir/phase-09-validation.txt"

grep -q "Identity and authority" docs/templates/preregistration.md
grep -q "Partition and leakage controls" docs/templates/dataset-card.md
echo "phase 09 validation passed; reproducible evidence: target/validation/phase-09-validation.txt"
