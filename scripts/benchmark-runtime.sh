#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
samples="${STE_BENCH_SAMPLES:-100}"
output_dir="${STE_BENCH_OUTPUT_DIR:-$root/target/benchmarks/phase03}"

usage() { echo "usage: $0 [run|gate] [measurement-directory]" >&2; }

require_number() {
  local file="$1" expression="$2" label="$3"
  jq -e "$expression | numbers" "$file" >/dev/null || {
    echo "benchmark result lacks numeric $label: $file" >&2; exit 2;
  }
}

gate() {
  local directory="${1:-$output_dir}"
  local idle="$directory/idle.json" synthetic="$directory/synthetic.json"
  [[ -s "$idle" && -s "$synthetic" ]] || {
    echo "idle and synthetic benchmark JSON are required in $directory" >&2; exit 2;
  }
  jq -e '.profile == "idle"' "$idle" >/dev/null
  jq -e '.profile == "synthetic"' "$synthetic" >/dev/null
  for specification in \
    "$idle .cpu_percent idle.cpu_percent" \
    "$idle .max_rss_kib idle.max_rss_kib" \
    "$idle .startup_ms idle.startup_ms" \
    "$idle .shutdown_ms idle.shutdown_ms" \
    "$synthetic .queue_latency_p95_us synthetic.queue_latency_p95_us" \
    "$synthetic .dropped_critical_events synthetic.dropped_critical_events"; do
    read -r file expression label <<<"$specification"
    require_number "$file" "$expression" "$label"
  done
  jq -e '(.bounded_capacity | numbers) > 0' "$synthetic" >/dev/null || {
    echo "SLO: synthetic pipeline did not report bounded capacity" >&2; exit 1;
  }

  if jq -e '.duration_ms >= 1000' "$idle" >/dev/null; then
    jq -e '.cpu_percent <= 20' "$idle" >/dev/null || { echo "SLO: idle CPU exceeds 20%" >&2; exit 1; }
  else
    echo "NOTE: idle CPU is informational because probe duration is below 1s"
  fi
  jq -e '.max_rss_kib <= 262144' "$idle" >/dev/null || { echo "SLO: idle RSS exceeds 256MiB" >&2; exit 1; }
  jq -e '.startup_ms <= 2000 and .shutdown_ms <= 2000' "$idle" >/dev/null || {
    echo "SLO: startup or coordinated shutdown exceeds 2s" >&2; exit 1;
  }
  jq -e '.queue_latency_p95_us <= 50000' "$synthetic" >/dev/null || {
    echo "SLO: synthetic queue p95 exceeds 50ms" >&2; exit 1;
  }
  jq -e '.dropped_critical_events == 0' "$synthetic" >/dev/null || {
    echo "SLO: synthetic run dropped critical events" >&2; exit 1;
  }
  jq -e '.operations >= 10000 and .critical_delivered > 0' "$synthetic" >/dev/null || {
    echo "SLO: synthetic probe did not exercise the required load and critical path" >&2; exit 1;
  }
  echo "Phase 03 development runtime budget gates passed"
}

run() {
  [[ "$samples" =~ ^[1-9][0-9]*$ ]] || { echo "STE_BENCH_SAMPLES must be positive" >&2; exit 2; }
  command -v jq >/dev/null || { echo "jq is required" >&2; exit 2; }
  mkdir -p "$output_dir"
  cargo build --manifest-path "$root/Cargo.toml" --release --locked \
    -p ste-runtime --example runtime_benchmark >/dev/null
  cargo build --manifest-path "$root/Cargo.toml" --release --locked -p ste-cli >/dev/null
  local probe="$root/target/release/examples/runtime_benchmark" cli="$root/target/release/ste"
  local timings sorted start end p50 p95
  timings="$(mktemp)"; sorted="$(mktemp)"
  trap 'rm -f "$timings" "$sorted"' RETURN

  "$probe" --profile idle --json >"$output_dir/idle.json"
  "$probe" --profile synthetic --json >"$output_dir/synthetic.json"
  jq -e . "$output_dir/idle.json" "$output_dir/synthetic.json" >/dev/null

  "$cli" >/dev/null
  for ((i=0; i<samples; i++)); do
    start="$(date +%s%N)"; "$cli" >/dev/null; end="$(date +%s%N)"
    echo $(((end - start) / 1000)) >>"$timings"
  done
  sort -n "$timings" >"$sorted"
  p50="$(awk -v n="$samples" 'NR == int((n * 50 + 99) / 100) { print; exit }' "$sorted")"
  p95="$(awk -v n="$samples" 'NR == int((n * 95 + 99) / 100) { print; exit }' "$sorted")"
  jq -n --arg schema ste-process-startup-v1 --arg revision "$(git -C "$root" rev-parse HEAD)" \
    --arg target "$(rustc -vV | sed -n 's/^host: //p')" --argjson samples "$samples" \
    --argjson p50 "$p50" --argjson p95 "$p95" \
    '{schema:$schema,git_revision:$revision,target:$target,samples:$samples,startup_to_exit_p50_us:$p50,startup_to_exit_p95_us:$p95}' \
    >"$output_dir/process-startup.json"

  # GNU time provides an independent RSS observation on qualified hosts. It is
  # optional because minimal build containers may not install it.
  if [[ -x /usr/bin/time ]]; then
    /usr/bin/time -f '{"external_max_rss_kib":%M,"elapsed_s":%e}' \
      -o "$output_dir/external-process.json" "$probe" --profile idle --json >/dev/null
  fi
  sha256sum "$output_dir"/*.json >"$output_dir/SHA256SUMS"
  gate "$output_dir"
  echo "wrote benchmark evidence to $output_dir"
}

case "${1:-run}" in
  run) run ;;
  gate) gate "${2:-$output_dir}" ;;
  *) usage; exit 2 ;;
esac
