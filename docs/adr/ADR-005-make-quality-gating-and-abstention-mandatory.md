# ADR-005: Make Quality Gating and Abstention Mandatory

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: uncertainty, quality, safety, inference

## Context

Motion, interference, missing packets, multipath change, unsupported model scope, and insufficient evidence windows can invalidate STE estimates. Always emitting a label would hide these failures and create unjustified certainty.

## Decision

Every evidence-producing context must implement quality assessment and a first-class abstention result. Estimates require all applicable gates: capture health, signal quality, motion/stillness, minimum evidence horizon, model scope, calibration, modality/construct promotion, and active authorization.

An abstention records a stable reason code, failed gates, evidence horizon, and recovery guidance. The UI must render unauthorized, calibrating, contaminated, insufficient-evidence, out-of-scope, and unavailable states explicitly.

Measure coverage and abstention rate alongside accuracy and latency.

## Consequences

### Positive

- Prevents low-quality evidence from becoming a confident cognitive label.
- Makes failure observable, testable, and useful for hardware placement.

### Negative

- The system may emit no estimate frequently in early stages.
- Calibration of gates is itself a validation task.

### Neutral

- Improving coverage must not weaken preregistered error thresholds post hoc.

## Links

**Depends on**: [ADR-003](ADR-003-separate-observations-physiology-and-latent-state.md), [ADR-004](ADR-004-use-rvcsi-and-nexmon-behind-a-versioned-capture-port.md)
**Related**: [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md)
