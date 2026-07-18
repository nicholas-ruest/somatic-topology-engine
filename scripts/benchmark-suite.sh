#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
directory="${STE_SUITE_OUTPUT_DIR:-$root/target/benchmarks/phase08}"
mkdir -p "$directory/runtime"

STE_BENCH_OUTPUT_DIR="$directory/runtime" bash "$root/scripts/benchmark-runtime.sh" run
STE_ACQUISITION_BENCH_OUTPUT="$directory/acquisition.json" bash "$root/scripts/benchmark-acquisition.sh" run
STE_OBSERVATION_BENCH_OUTPUT="$directory/observation.json" bash "$root/scripts/benchmark-observation.sh" run

jq -n --sort-keys \
  --arg revision "$(git -C "$root" rev-parse HEAD)" \
  --arg target "$(rustc -vV | sed -n 's/^host: //p')" \
  --slurpfile idle "$directory/runtime/idle.json" \
  --slurpfile synthetic "$directory/runtime/synthetic.json" \
  --slurpfile process "$directory/runtime/process-startup.json" \
  --slurpfile acquisition "$directory/acquisition.json" \
  --slurpfile observation "$directory/observation.json" \
  '{schema:"ste-benchmark-suite-v1",git_revision:$revision,target:$target,
    runtime:{idle:$idle[0],synthetic:$synthetic[0],process:$process[0]},
    acquisition:$acquisition[0],observation:$observation[0],
    pending:{reference_pi_temperature:true,reference_pi_power:true,reference_pi_soak:true}}' \
  >"$directory/baseline.json"

bash "$root/scripts/compare-regression.sh" "$directory/baseline.json" "$directory/baseline.json" >"$directory/self-comparison.json"
sha256sum "$directory"/*.json "$directory"/runtime/*.json >"$directory/SHA256SUMS"
echo "wrote Phase 08 machine-readable baseline to $directory/baseline.json"
