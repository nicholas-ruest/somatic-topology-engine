# Phase 11 model lifecycle benchmark

Status: current-host engineering benchmark executed; reference Pi pending.

The benchmark measures canonical package build/verification and atomic registry activation/rollback on the current host. It must record architecture, OS, Rust toolchain, iteration count, package payload size, elapsed time, and throughput. A benchmark never relaxes signature, compatibility, known-answer, promotion, policy, or rollback gates.

## Reference-device procedure

Run the identical signed fixtures on the declared Pi model/image and record package digest, CPU/RAM, kernel, firmware, compiler/build digest, storage medium, temperature, throttling, power, p50/p95/p99/max verification and activation latency, RSS, and recovery duration. Exercise corrupt signature, digest mismatch, incompatible schema/hardware, revoked package, failed known-answer, interrupted activation, and failed health check. Confirm the previous package remains or returns active with its exact known-answer output.

Until physical HIL and soak execution is attached, Pi/resource release gates remain pending.

## Observed current-host run

On 2026-07-18, Rust 1.85.1 on x86_64 Linux verified a signed 4,096-byte fixture 10,000 times in 475,510,670 ns, approximately 21,030 verifications/second. The same executable completed the activation, health suspension, rollback, and prior-package identity assertion. This is one host observation, not a latency distribution, Pi capacity claim, or HIL/soak result.
