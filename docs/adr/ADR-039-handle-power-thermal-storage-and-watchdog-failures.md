# ADR-039: Handle Power, Thermal, Storage, and Watchdog Failures

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: hardware, thermal, power, watchdog, storage

## Context

Pi throttling, low voltage, SD corruption, and hung processes can degrade timing or silently stop sensing. Production behavior must remain safe under resource and power faults.

## Decision

Monitor voltage flags, temperature, throttling, free space, filesystem health, write latency, task heartbeats, and journal progress. Apply staged degradation: suspend nonessential sidecars, reduce noncritical work, stop model inference, stop capture safely, and enter a visible fault state before hard limits.

Use an independent system watchdog fed only after critical health checks. Implement graceful shutdown and power-loss-safe journals. Reserve emergency storage, rate-limit writes, document supported media/endurance, and perform boot-time integrity/recovery checks. Never reinterpret DHT ambient data as CPU thermal state.

## Consequences

### Positive
- Hardware faults become controlled service states.
- Reduces corrupt storage and stale-output risks.

### Negative
- Platform-specific monitoring and destructive fault tests are required.
- Conservative thresholds may reduce availability.

### Neutral
- A watchdog restart does not substitute for root-cause diagnostics.

## Links

**Depends on**: [ADR-015](ADR-015-supervise-failures-and-degrade-capabilities-independently.md), [ADR-020](ADR-020-build-local-first-observability-and-support-bundles.md), [ADR-021](ADR-021-enforce-slos-and-resource-budgets-with-benchmarks.md), [ADR-038](ADR-038-require-hardware-in-loop-fault-injection-and-soak-testing.md)
