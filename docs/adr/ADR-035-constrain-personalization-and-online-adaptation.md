# ADR-035: Constrain Personalization and Online Adaptation

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: personalization, online-learning, feedback, safety

## Context

Anchors and feedback can improve personalization but can also encode accidental touches, biased labels, poisoning, catastrophic forgetting, and evaluation leakage.

## Decision

Separate retrieval personalization from parameter adaptation. Enable retrieval first using immutable anchors and scoped similarity. Parameter updates occur only in a sandbox from explicit, quality-qualified feedback with minimum evidence, provenance, rate limits, and rollback.

Never learn from implicit UI acceptance, abstentions, policy-denied data, or held-out partitions. Evaluate a candidate adaptation prospectively against the active model, baselines, safety gates, and regression suites before promotion. Show users their anchors and provide correction/deletion controls.

## Consequences

### Positive
- Personalization remains explainable and reversible.
- Prevents uncontrolled self-reinforcing drift.

### Negative
- Improvement is slower than unconstrained online learning.
- Requires per-user evaluation and storage budgets.

### Neutral
- AgentDB or RuVector learning features cannot bypass domain lifecycle rules.

## Links

**Depends on**: [ADR-007](ADR-007-use-ruvector-as-primary-personalization-memory.md), [ADR-028](ADR-028-govern-retention-export-backup-recovery-and-deletion.md), [ADR-033](ADR-033-govern-model-registration-promotion-activation-and-rollback.md), [ADR-034](ADR-034-calibrate-uncertainty-detect-out-of-distribution-inputs-and-abstain.md)
