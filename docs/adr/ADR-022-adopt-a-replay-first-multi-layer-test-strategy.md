# ADR-022: Adopt a Replay-First Multi-Layer Test Strategy

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: testing, replay, property-testing, hil

## Context

Hardware and physiological signals are nondeterministic, while production confidence requires repeatable regression testing. Unit tests alone cannot prove cross-layer correctness or field resilience.

## Decision

Use a layered test portfolio: domain unit/property tests, parser fuzzing, golden DSP vectors, contract tests, deterministic capture replay, model parity tests, persistence migration/corruption tests, integration tests, hardware-in-loop tests, fault injection, thermal/power tests, soak tests, security tests, and site acceptance tests.

Version and license every fixture. Synthetic data tests invariants and faults but never substitutes for real validation data. Require traceability from each ADR invariant and hazard control to automated tests or a documented manual verification.

## Consequences

### Positive
- Most regressions reproduce without live participants or hardware.
- Architectural decisions become verifiable obligations.

### Negative
- Fixtures, hardware labs, and long-running suites are costly.
- Golden tests must distinguish intentional algorithm change from regression.

### Neutral
- Test tiers run at different frequencies according to duration and hardware need.

## Links

**Depends on**: [ADR-004](ADR-004-use-rvcsi-and-nexmon-behind-a-versioned-capture-port.md), [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md), [ADR-018](ADR-018-use-an-append-only-journal-with-versioned-projections.md), [ADR-019](ADR-019-evolve-contracts-with-explicit-compatibility-rules.md), [ADR-021](ADR-021-enforce-slos-and-resource-budgets-with-benchmarks.md)
