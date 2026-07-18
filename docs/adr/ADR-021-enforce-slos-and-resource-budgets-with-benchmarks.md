# ADR-021: Enforce SLOs and Resource Budgets with Benchmarks

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: performance, slo, benchmarks, capacity

## Context

“Real time” and “runs on Pi” are not testable requirements. CPU, memory, storage, temperature, queue delay, and modality-specific latency compete on fixed hardware.

## Decision

Define versioned service-level objectives and budgets per deployment profile: capture continuity, valid-window coverage, queue delay, projection freshness, modality latency, startup/recovery time, CPU, RSS, storage growth, temperature, and power. Distinguish hard safety limits, release gates, and aspirational targets.

Maintain replay microbenchmarks, end-to-end benchmarks, worst-case interference loads, and 24-hour-plus steady-state tests on the reference Pi. Compare distributions and tail latency, not averages alone. Block releases on statistically significant regressions beyond declared tolerance.

## Consequences

### Positive
- Production capacity and “real time” become measurable.
- Optimization is evidence-driven.

### Negative
- Stable hardware runners and benchmark governance are required.
- Tight budgets may limit optional integrations.

### Neutral
- Scientific accuracy gates remain separate from system SLOs.

## Links

**Depends on**: [ADR-014](ADR-014-use-a-bounded-asynchronous-runtime-with-backpressure.md), [ADR-020](ADR-020-build-local-first-observability-and-support-bundles.md)
