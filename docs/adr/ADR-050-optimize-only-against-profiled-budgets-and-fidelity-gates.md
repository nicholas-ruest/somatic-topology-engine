# ADR-050: Optimize Only Against Profiled Budgets and Fidelity Gates

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: optimization, profiling, fidelity, performance

## Context

Quantization, SIMD, window changes, allocation reduction, and model simplification can improve Pi performance while subtly changing signal or scientific behavior. Premature optimization can freeze the wrong architecture.

## Decision

Profile end-to-end and attribute CPU, memory, allocation, I/O, queue, thermal, and energy costs before optimizing. Prioritize algorithmic removal, bounded data movement, batching, zero-copy where safe, and Rust-native acceleration. Isolate unsafe/SIMD/FFI code behind verified interfaces.

Every optimization must pass numerical replay, model parity, calibration, selective-risk, SLO, thermal, and operating-envelope gates. Record benchmark methodology and effect sizes. Do not trade scientific error or privacy controls for speed without a new ADR and claim review.

## Consequences

### Positive
- Performance work preserves high fidelity and is reproducible.
- Optimization effort targets measured bottlenecks.

### Negative
- Scientific and HIL revalidation slows optimization cycles.
- Fast third-party kernels may be rejected.

### Neutral
- Different signed deployment profiles may use different verified optimization paths.

## Links

**Depends on**: [ADR-021](ADR-021-enforce-slos-and-resource-budgets-with-benchmarks.md), [ADR-030](ADR-030-version-the-dsp-pipeline-and-require-numerical-replay.md), [ADR-032](ADR-032-standardize-edge-model-packages-and-the-rust-inference-port.md), [ADR-034](ADR-034-calibrate-uncertainty-detect-out-of-distribution-inputs-and-abstain.md), [ADR-038](ADR-038-require-hardware-in-loop-fault-injection-and-soak-testing.md)
