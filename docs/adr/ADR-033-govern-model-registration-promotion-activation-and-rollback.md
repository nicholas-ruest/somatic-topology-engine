# ADR-033: Govern Model Registration, Promotion, Activation, and Rollback

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: mlops, registry, promotion, rollback

## Context

Models can regress accuracy, calibration, fairness, coverage, performance, or safety even when software is unchanged. Model lifecycle requires stronger controls than copying a weights file.

## Decision

Maintain a local signed model registry with states: quarantined, evaluated, promoted, active, suspended, retired, and revoked. Promotion requires ADR-008 scientific gates, security scan, license/provenance checks, known-answer parity, resource benchmarks, compatibility checks, model card, and named approval.

Activation is atomic and records the previous model. Continuously monitor allowed local health signals and suspend on integrity, scope, or safety violations. Support immediate rollback and revocation independent of full software release. Preserve immutable decision evidence.

## Consequences

### Positive
- Model changes are auditable, reversible production changes.
- Scientific and operational gates converge before activation.

### Negative
- Registry, approval, and compatibility matrices add process.
- Monitoring cannot rely on unavailable ground truth in normal use.

### Neutral
- A personalized adaptation is a model version and follows the same lineage rules.

## Links

**Depends on**: [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md), [ADR-026](ADR-026-use-signed-reproducible-updates-with-atomic-rollback.md), [ADR-032](ADR-032-standardize-edge-model-packages-and-the-rust-inference-port.md)
