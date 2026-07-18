# Phase 08 integrated development baseline

Run `bash scripts/benchmark-suite.sh`. It executes the runtime, acquisition, and
observation release benchmarks, combines their JSON into
`target/benchmarks/phase08/baseline.json`, records revision/target/pending
hardware fields, self-compares the baseline, and writes SHA-256 checksums. Use
`scripts/compare-regression.sh <baseline> <candidate>` to enforce a 10% maximum
latency/RSS regression and 10% maximum throughput regression.

Measured 2026-07-18 on `x86_64-unknown-linux-gnu`, revision
`59af419fcdc40b09d16771ab23029c1c5e2e9afd`:

| Metric | Baseline |
| --- | ---: |
| CLI startup-to-exit p50/p95 | 3.243 / 3.758 ms |
| Idle/synthetic max RSS | 2,252 / 2,432 KiB |
| Synthetic operations/critical loss | 10,000 / 0 |
| Acquisition replay | 17,395,905.94 frames/s |
| Acquisition accepted/rejected | 200,000 / 0 |
| Observation replay | 1,139,912.20 frames/s; 2,226.39 windows/s |
| Observation numerical delta | 0.0 |

The runtime idle probe lasted 0.038 ms, so its reported 0% CPU is beneath Linux
process-tick resolution and is not a sustained-idle claim. Queue and shutdown
zeroes are likewise timer-floor observations. This baseline excludes live radio,
disk persistence, UI, physical peripherals, temperature, and power.

Reference Pi temperature, power/undervoltage, storage growth, multi-day soak,
and full-pipeline distributions remain explicitly pending and are release
blockers rather than inferred from this container.
