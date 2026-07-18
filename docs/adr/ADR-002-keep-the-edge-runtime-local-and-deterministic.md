# ADR-002: Keep the Edge Runtime Local and Deterministic

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: edge, offline, determinism, reliability

## Context

The project promises no cloud dependency and handles sensitive ambient physiological evidence. Network calls and agent-generated behavior in the live path would add privacy exposure, nondeterministic latency, and failure modes.

## Decision

Run capture, validation, DSP, inference, quality gates, authorization, memory, and device projection locally on the CrowPi/Raspberry Pi. The live path must operate without internet connectivity and produce deterministic results for identical frames, configuration, model, calibration, and state.

External services are prohibited from the runtime decision path. Optional development, model-building, or documentation workflows may use external tools only on explicitly exported, policy-compliant data.

Use bounded queues, backpressure, monotonic time, supervised tasks, and explicit degraded states. Preserve deterministic capture replay as an acceptance requirement.

## Consequences

### Positive

- Predictable operation and privacy even when disconnected.
- Failures can be reproduced from pinned artifacts and replay data.

### Negative

- Models and dependencies must fit Pi CPU, memory, storage, and thermal budgets.
- Updates and synchronization need an explicit offline-safe mechanism.

### Neutral

- “Local” does not remove the need for consent, encryption, retention, or threat modeling.

## Links

**Depends on**: [ADR-001](ADR-001-adopt-a-rust-first-modular-monolith.md)
**Related**: [ADR-004](ADR-004-use-rvcsi-and-nexmon-behind-a-versioned-capture-port.md), [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md)
