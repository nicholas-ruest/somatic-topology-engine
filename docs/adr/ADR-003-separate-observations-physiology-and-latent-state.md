# ADR-003: Separate Observations, Physiology, and Latent State

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: domain-model, evidence, safety, types

## Context

CSI directly measures a radio channel. Motion or periodicity are derived observations; respiration or cardiac rate are physiology estimates; workload, arousal, valence, and decision phase are latent constructs. Conflating these layers would turn correlation into unsupported claims.

## Decision

Model three compile-time-distinct schemas and bounded contexts:

1. signal observations with frame evidence and signal quality;
2. physiology estimates with modality validation, evidence horizon, and error metadata; and
3. latent-state estimates with operational construct, model scope, calibration, and claim level.

No implicit conversion is allowed. Each transition uses a versioned application contract and records provenance. User-facing projections consume policy-approved state, never raw CSI features or model tensors.

## Consequences

### Positive

- Scientific claims remain traceable and falsifiable.
- The system can deliver useful sensing while higher-level claims remain disabled.
- Type boundaries reduce accidental overstatement.

### Negative

- More explicit types, mappings, and persistence records.
- Some upstream “end-to-end” models require wrapping or decomposition.

### Neutral

- Confidence is meaningful only within the layer and model that produced it.

## Links

**Depends on**: [ADR-001](ADR-001-adopt-a-rust-first-modular-monolith.md)
**Related**: [State Inference context](../ddd/contexts/state-inference.md), [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md)
