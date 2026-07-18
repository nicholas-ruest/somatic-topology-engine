#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output="${STE_OBSERVATION_BENCH_OUTPUT:-$root/target/benchmarks/phase07-observation.json}"

gate() {
  local report="$1"
  jq -e '.schema == "ste-observation-benchmark-v1"' "$report" >/dev/null
  jq -e '.deterministic == true and .numerical_delta <= 1e-12' "$report" >/dev/null || {
    echo "observation replay numerical determinism gate failed" >&2; exit 1;
  }
  jq -e '.digest | test("^[0-9a-f]{64}$")' "$report" >/dev/null || {
    echo "observation artifact digest is invalid" >&2; exit 1;
  }
  jq -e '.frames_s >= 100000 and .windows_s >= 100' "$report" >/dev/null || {
    echo "observation development throughput gate failed" >&2; exit 1;
  }
  echo "Phase 07 observation benchmark gate passed"
}

case "${1:-run}" in
  run)
    mkdir -p "$(dirname "$output")"
    cargo run --quiet --release --manifest-path "$root/Cargo.toml" \
      -p ste-cli --example observation_benchmark >"$output"
    gate "$output"
    echo "wrote $output"
    ;;
  gate) gate "${2:-$output}" ;;
  *) echo "usage: $0 [run|gate] [report]" >&2; exit 2 ;;
esac
