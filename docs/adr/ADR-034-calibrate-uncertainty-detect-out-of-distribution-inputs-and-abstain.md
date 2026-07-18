# ADR-034: Calibrate Uncertainty, Detect Out-of-Distribution Inputs, and Abstain

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: uncertainty, calibration, ood, abstention

## Context

Neural scores are not automatically calibrated probabilities. New rooms, users, postures, interference, and hardware can produce confident errors outside training scope.

## Decision

Calibrate probabilities on partitions separate from training and report reliability, proper scoring rules, and calibration error. Define per-capability OOD and scope checks using feature distribution, quality, profile compatibility, and model-card constraints. Combine model uncertainty with acquisition/physiology gates through a documented decision policy.

Thresholds are versioned artifacts chosen before held-out evaluation. Output an abstention with reason when calibration is invalid, evidence is OOD, confidence is insufficient, or required inputs disagree. Evaluate selective risk versus coverage rather than accuracy alone.

## Consequences

### Positive
- Confidence and abstention have measurable semantics.
- Domain shift is less likely to become a confident claim.

### Negative
- OOD detection is imperfect and itself needs validation.
- Conservative gates reduce coverage.

### Neutral
- Calibration is model-, participant-, task-, and deployment-scope specific.

## Links

**Depends on**: [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md), [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md), [ADR-031](ADR-031-use-immutable-feature-and-evidence-artifacts.md), [ADR-033](ADR-033-govern-model-registration-promotion-activation-and-rollback.md)
