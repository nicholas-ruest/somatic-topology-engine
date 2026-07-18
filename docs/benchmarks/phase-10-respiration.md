# Phase 10 respiration benchmark

## Status

Current-host benchmark available; reference Raspberry Pi benchmark pending.

Run `scripts/validate-respiration.sh` to record the host architecture, OS, Rust toolchain, benchmark iteration count, elapsed time, and derived throughput beneath `target/validation/`. These engineering measurements do not establish scientific accuracy or production capacity on different hardware.

One observed run on 2026-07-18 used Rust 1.85.1 on x86_64 Linux. A release build completed 100,000 deterministic known-answer estimates in 177,719,823 ns (approximately 562,683 iterations/second). This is a single current-host engineering observation, not a statistically characterized latency distribution, Raspberry Pi result, soak result, thermal result, or release SLO claim. Re-run artifacts supersede this observation only when their environment and raw output are retained.

## Reference Pi procedure

On the declared reference Pi image and hardware revision:

1. Record model, CPU architecture, RAM, OS/kernel, Rust/compiler artifact, firmware, power supply, cooling, ambient temperature, and signed STE build digest.
2. Disable unrelated variable workloads and document CPU governor; do not disable production safety limits.
3. Run known-answer parity before measurement.
4. Execute release-mode warmup and repeated 30-second evidence-window assessments under nominal and worst-case bounded input loads.
5. Record p50/p95/p99 and maximum latency, throughput, RSS, allocations where available, CPU, temperature, throttling, power, queue delay, and numerical parity.
6. Run the declared soak duration and preserve failures, abstentions, restarts, and thermal excursions.
7. Attach immutable output and environment digests to the frozen validation study.

Until that procedure is performed on physical hardware, Pi latency, thermal, power, memory, and sustained-throughput gates remain **pending**, and `respiration-v1` remains disabled.
