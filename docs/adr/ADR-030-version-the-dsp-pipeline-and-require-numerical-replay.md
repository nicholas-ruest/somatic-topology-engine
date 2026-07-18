# ADR-030: Version the DSP Pipeline and Require Numerical Replay

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: dsp, numerics, replay, signal-processing

## Context

Filtering, phase handling, subcarrier selection, windowing, and feature extraction determine every downstream estimate. Minor numerical or ordering changes can invalidate trained models and historical comparisons.

## Decision

Represent the DSP graph as a versioned immutable specification with ordered transforms, parameters, input/output units, missing-data behavior, state reset semantics, and numerical tolerances. Prefer Rust rvCSI/RuView primitives when pinned and verified; wrap them behind STE traits.

Require golden vectors for impulse, constant, sinusoidal, noisy, gapped, saturated, and recorded CSI inputs. Replay must match within declared platform tolerance. Record CPU architecture and implementation path when SIMD/FFI can alter results. Any material DSP change creates a new feature schema and triggers model compatibility evaluation.

## Consequences

### Positive
- Features and models are reproducible across releases.
- Signal regressions are isolated before model evaluation.

### Negative
- Golden vectors and cross-platform tolerance need expert maintenance.
- Optimization cannot freely reorder numerically sensitive operations.

### Neutral
- Bit-for-bit equality is preferred but not required where documented floating-point tolerances are scientifically harmless.

## Links

**Depends on**: [ADR-017](ADR-017-standardize-identifiers-time-units-and-numeric-semantics.md), [ADR-022](ADR-022-adopt-a-replay-first-multi-layer-test-strategy.md), [ADR-029](ADR-029-version-csi-calibration-baselines-and-operating-geometry.md)
