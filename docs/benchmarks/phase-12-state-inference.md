# Phase 12 state-inference benchmark

Status: deterministic baseline current-host measurement executed; reference Pi and trained model pending.

The Phase 12 benchmark exercises no-op/baseline policy evaluation, temporal debounce, immutable persistence, and safe projection mapping. It reports host architecture, OS, Rust toolchain, iteration count, elapsed time, throughput, and a raw-output leakage assertion. It does not benchmark or imply a trained cognitive model.

Reference-Pi execution must additionally record hardware/image/build digests, p50/p95/p99/max assessment and projection latency, CPU, RSS, allocation behavior, queue delay, temperature, throttling, power, and soak failures. The same run must prove unsupported constructs stay unavailable and raw inference artifacts never reach a peripheral or API payload.

One run on 2026-07-18 used Rust 1.85.1 on x86_64 Linux and completed 100,000 no-claim baseline assessments plus safe JSON projections in 33,065,933 ns, approximately 3,024,261 iterations/second. The run asserted no raw probability or physiology reference appeared. This is a single host observation, not a trained-model, Pi, tail-latency, thermal, or production-capacity result.
