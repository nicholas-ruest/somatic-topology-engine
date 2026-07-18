# ADR-038: Require Hardware-in-Loop, Fault Injection, and Soak Testing

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: hil, fault-injection, soak, reliability

## Context

Replay cannot expose bus hangs, Wi-Fi firmware crashes, SD wear, brownouts, thermal throttling, peripheral timing, or long-duration resource leaks.

## Decision

Maintain reference CrowPi/Pi hardware runners with controlled AP traffic and peripheral fixtures. Test boot, capture, touch, OLED/RGB, DHT, update/rollback, storage recovery, and shutdown. Inject packet loss, malformed frames, channel changes, AP loss, disk-full, corruption, time jumps, process death, bus failure, thermal load, low voltage, and power interruption.

Run release-candidate soak tests long enough to detect leaks, counter rollover, storage growth, and thermal equilibrium. Define pass criteria from SLOs and hazard controls, preserve artifacts, and quarantine flaky tests rather than ignoring them.

## Consequences

### Positive
- Validates behavior unavailable to simulators.
- Recovery and degradation claims become measurable.

### Negative
- Hardware labs are slower, costlier, and harder to parallelize.
- Controlled RF tests need environmental discipline.

### Neutral
- Replay remains the fast default; HIL is an additional gate.

## Links

**Depends on**: [ADR-011](ADR-011-abstract-crowpi-hardware-behind-rust-ports.md), [ADR-015](ADR-015-supervise-failures-and-degrade-capabilities-independently.md), [ADR-021](ADR-021-enforce-slos-and-resource-budgets-with-benchmarks.md), [ADR-022](ADR-022-adopt-a-replay-first-multi-layer-test-strategy.md)
