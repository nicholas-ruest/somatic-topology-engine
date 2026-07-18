#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output="${STE_ACQUISITION_BENCH_OUTPUT:-$root/target/benchmarks/phase06-acquisition.json}"

gate() {
  local report="$1"
  jq -e '.schema == "ste-acquisition-benchmark-v1"' "$report" >/dev/null
  jq -e '.deterministic == true and .accepted == (.fixture_frames * .iterations)' "$report" >/dev/null || {
    echo "acquisition replay was incomplete or nondeterministic" >&2; exit 1;
  }
  jq -e '.rejected == 0' "$report" >/dev/null || {
    echo "known-good acquisition fixture produced rejected frames" >&2; exit 1;
  }
  jq -e '.throughput_frames_s >= 100000' "$report" >/dev/null || {
    echo "development replay throughput is below 100,000 frames/s" >&2; exit 1;
  }
  echo "Phase 06 acquisition development gate passed"
}

case "${1:-run}" in
  run)
    mkdir -p "$(dirname "$output")"
    cargo run --quiet --release --manifest-path "$root/Cargo.toml" \
      -p ste-cli --example acquisition_benchmark >"$output"
    gate "$output"
    echo "wrote $output"
    ;;
  gate) gate "${2:-$output}" ;;
  *) echo "usage: $0 [run|gate] [report]" >&2; exit 2 ;;
esac
