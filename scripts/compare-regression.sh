#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 2 ]]; then
  echo "usage: $0 <baseline.json> <candidate.json>" >&2
  exit 2
fi
baseline="$1"
candidate="$2"
jq -e '.schema == "ste-benchmark-suite-v1"' "$baseline" "$candidate" >/dev/null

report="$(jq -n --slurpfile b "$baseline" --slurpfile c "$candidate" '
  def lower($name; $old; $new; $fraction):
    {metric:$name,baseline:$old,candidate:$new,passed:
      (if $old == 0 then $new == 0 else $new <= ($old * (1 + $fraction)) end)};
  def higher($name; $old; $new; $fraction):
    {metric:$name,baseline:$old,candidate:$new,passed:($new >= ($old * (1 - $fraction)))};
  {schema:"ste-regression-comparison-v1",checks:[
    lower("runtime_startup_p95_us";$b[0].runtime.process.startup_to_exit_p95_us;$c[0].runtime.process.startup_to_exit_p95_us;0.10),
    lower("runtime_idle_rss_kib";$b[0].runtime.idle.max_rss_kib;$c[0].runtime.idle.max_rss_kib;0.10),
    lower("runtime_queue_p95_us";$b[0].runtime.synthetic.queue_latency_p95_us;$c[0].runtime.synthetic.queue_latency_p95_us;0.10),
    higher("acquisition_frames_s";$b[0].acquisition.throughput_frames_s;$c[0].acquisition.throughput_frames_s;0.10),
    higher("observation_frames_s";$b[0].observation.frames_s;$c[0].observation.frames_s;0.10),
    higher("observation_windows_s";$b[0].observation.windows_s;$c[0].observation.windows_s;0.10)
  ]} | . + {passed:(.checks | all(.passed))}
')"
printf '%s\n' "$report"
jq -e '.passed == true' <<<"$report" >/dev/null || exit 1
