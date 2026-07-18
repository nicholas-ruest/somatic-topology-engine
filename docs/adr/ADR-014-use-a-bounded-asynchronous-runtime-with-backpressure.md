# ADR-014: Use a Bounded Asynchronous Runtime with Backpressure

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: runtime, async, backpressure, latency

## Context

CSI capture, windowing, inference, persistence, and UI operate at different rates. Unbounded queues can exhaust Pi memory and convert overload into stale but apparently live output.

## Decision

Use Tokio as the initial Rust runtime with supervised tasks and bounded channels. Assign each stream a documented capacity, overflow policy, latency budget, and shutdown behavior. Prefer dropping explicitly classified intermediate observations over blocking capture or presenting stale state; never drop authorization revocations, audit events, anchors, or model lifecycle commands.

Propagate cancellation, correlation IDs, monotonic timestamps, and backpressure metrics. The runtime composition root owns task spawning; domain code does not spawn tasks.

## Consequences

### Positive
- Predictable memory and overload behavior.
- Queue delay and loss become measurable.

### Negative
- Capacity and shedding policies require workload benchmarks.
- Async tests need deterministic clocks and schedulers where possible.

### Neutral
- Tokio can be replaced behind application ports if benchmarks justify it.

## Links

**Depends on**: [ADR-002](ADR-002-keep-the-edge-runtime-local-and-deterministic.md), [ADR-006](ADR-006-use-multi-timescale-event-time-processing.md), [ADR-013](ADR-013-define-the-cargo-workspace-and-crate-dependency-policy.md)
