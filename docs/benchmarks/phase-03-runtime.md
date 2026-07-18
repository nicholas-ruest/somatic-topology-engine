# Phase 03 runtime resource baseline

**Evidence state:** development-host baseline measured; persistent-runtime and
Raspberry Pi 4 evidence pending. This report does not claim ARM qualification.

## Scope and method

The Phase 03 executable harness measures the current release CLI's warm
startup-to-exit distribution and invokes the runtime's deterministic idle and
synthetic profiles for CPU, RSS, bounded-queue latency, load shedding, startup,
and shutdown evidence. It records the Git revision, target, profile, and sample
counts in machine-readable JSON.

Run and independently gate the measurements with:

```bash
STE_BENCH_SAMPLES=100 bash scripts/benchmark-runtime.sh run
bash scripts/benchmark-runtime.sh gate target/benchmarks/phase03
```

The gate is intentionally limited to measurements the current executable can
make soundly:

| Metric | Development release gate | Classification |
| --- | ---: | --- |
| Warm startup-to-exit p95 | <= 100 ms | Release regression gate |
| Idle maximum RSS | <= 256 MiB | Release regression gate |
| Supervisor startup and shutdown | <= 2 seconds each | Release regression gate |
| Synthetic queue latency p95 | <= 50 ms | Release regression gate |
| Synthetic critical-event loss | zero | Safety regression gate |
| Synthetic operations | >= 10,000 | Probe validity gate |

CLI startup-to-exit includes process startup and normal CLI shutdown. The
separate runtime probe measures coordinated supervisor shutdown. Synthetic
operations are deterministic in-memory events, not acquisition throughput.

## Development-host result

Measured on 2026-07-18 in the project development container using Rust 1.85.1,
release profile, 100 warm startup samples, and 10,000 synthetic events:

| Metric | Result | Gate |
| --- | ---: | --- |
| Warm startup-to-exit p50 | 3.269 ms | 100 ms p95 |
| Warm startup-to-exit p95 | 3.569 ms | 100 ms p95 |
| Idle probe maximum RSS | 2,016 KiB | 256 MiB |
| Synthetic probe maximum RSS | 2,560 KiB | 256 MiB |
| Synthetic operations | 10,000 in 0.949 ms | 10,000 minimum |
| Synthetic queue latency p95 | < 1 us (timer floor) | 50 ms |
| Critical events delivered/dropped | 10 / 0 | zero dropped |
| Bounded queue peak/capacity | 256 / 256 | bounded required |
| Supervisor startup/shutdown | 0.004 / < 0.001 ms | 2 s each |

The idle probe completed in 0.038 ms and reported 0% CPU. That value is below
Linux process-tick resolution and is informational, not a sustained-idle CPU
claim; the executable gate only enforces idle CPU once the probe lasts at least
one second. Queue and shutdown values at zero similarly mean “below the probe's
microsecond/millisecond timer resolution,” not literally zero work.

The checked-in values are a point-in-time engineering baseline. Generated JSON
and `SHA256SUMS` under `target/benchmarks/phase03` are the authoritative raw
results for a run and are intentionally excluded from version control.

## Executable fault matrix

Run `bash tests/phase03/run-fault-matrix.sh`. The harness names every required
control so deleting or renaming a witness fails the gate. It covers optional
queue shedding without accepted-critical loss, safe drop-oldest behavior,
independent optional-task degradation, restart-budget circuit opening,
coordinated cancellation/shutdown, clock discontinuity, low-resource safe
state, deterministic replay, monotonic event time, and typed clock overflow.

## Required persistent-runtime follow-up

The initial deterministic pipeline probe now covers bounded saturation,
critical-event delivery, queue timing, and coordinated supervisor shutdown.
Before production acceptance, extend its duration and load model to add:

- idle CPU distribution and RSS after a five-minute stabilization period;
- synthetic pipeline throughput and p50/p95/p99 queue latency;
- bounded-memory overload and critical-event preservation evidence;
- startup-to-ready and coordinated shutdown-to-safe-state distributions.

## Raspberry Pi 4 reference procedure

Run the same locked revision and release profile on the supported 64-bit Pi OS
image and record the board revision, RAM, kernel, firmware, power supply,
governor, ambient temperature, and thermal/throttling state. Use at least 1,000
startup samples, a 30-minute synthetic load, five idle minutes, and a 24-hour
soak. Capture `vcgencmd get_throttled`, temperature, CPU, RSS, storage growth,
and p50/p95/p99 latency before and after the load. Preserve the raw environment
file and its SHA-256 checksum as an assurance report.

**ARM measurements: pending execution on reference hardware.** No ARM CPU,
memory, temperature, power, or latency result is inferred from this development
container.
