# ADR-008: Promote Capabilities Through Preregistered Validation Gates

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: validation, science, model-governance, claims

## Context

The research supports CSI presence, motion, and respiration more strongly than cardiac intervals, workload, valence, or decision phase. Upstream software capability does not validate an STE scientific claim, and window-level random splits can leak session or participant information.

## Decision

Gate each capability through frozen protocols and immutable evidence. Promote in stages: acquisition/replay, signal observations, respiration, cardiac rate, interval-derived metrics, task-specific personalized workload, then separate affect and decision experiments.

Before confirmatory evaluation, freeze target definitions, reference sensors, cohort, session-level split strategy, nuisance covariates, baselines, metrics, thresholds, and required coverage. Report error, calibration, confidence intervals, abstention, failure rate, and held-out-day performance. Never split windows from one session across train and test.

Only promoted capabilities can be enabled in State Inference or user-facing projections.

## Consequences

### Positive

- Claims remain falsifiable and negative results guide scope honestly.
- Prevents convenient implementation metrics from becoming product claims.

### Negative

- Higher-level features may remain unavailable for a long time.
- Reference equipment, ethics review, and repeated sessions increase cost.

### Neutral

- Engineering gates and scientific validity gates are tracked separately.

## Links

**Depends on**: [ADR-003](ADR-003-separate-observations-physiology-and-latent-state.md), [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md)
**Related**: [Experiment Validation context](../ddd/contexts/experiment-validation.md), [research validation program](../research.md#validation-program)
