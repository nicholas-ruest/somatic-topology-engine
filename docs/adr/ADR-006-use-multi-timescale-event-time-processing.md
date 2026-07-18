# ADR-006: Use Multi-Timescale Event-Time Processing

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: streaming, time, latency, windows

## Context

The brief requests 500 ms updates, while respiration, cardiac, HRV-like, and latent-state features require different and generally longer evidence windows. Equating refresh cadence with measurement resolution would be scientifically invalid.

## Decision

Use event-time processing with monotonic source timestamps, explicit window policies, watermarks/gap handling, and multiple evidence horizons. Maintain separate policies for signal health, motion, respiration, cardiac estimates, interval-derived features, and latent-state transitions.

The UI may refresh at 2 Hz, but every projection carries the age and horizon of its underlying evidence. Slow estimates are reused until replaced or marked stale. MidStream may be adapted behind a Rust port only after proving it meets these contracts; it is not assumed to provide sensor inference natively.

## Consequences

### Positive

- Honest real-time behavior without discarding responsive feedback.
- Deterministic replay of window closure and late/missing frame behavior.

### Negative

- Window coordination, buffering, and staleness add complexity.
- End-to-end latency is modality-specific rather than one headline number.

### Neutral

- Display rate remains a UX decision, not a scientific sampling claim.

## Links

**Depends on**: [ADR-003](ADR-003-separate-observations-physiology-and-latent-state.md), [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md)
**Related**: [Signal Observation context](../ddd/contexts/signal-observation.md)
