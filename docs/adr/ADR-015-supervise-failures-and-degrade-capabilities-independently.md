# ADR-015: Supervise Failures and Degrade Capabilities Independently

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: resilience, errors, supervision, recovery

## Context

A peripheral failure, corrupt frame, storage outage, or model error must not silently crash the system or leave a stale healthy display. Not every failure warrants restarting capture or the entire process.

## Decision

Define a typed error taxonomy: invalid input, transient dependency, resource exhaustion, policy denial, corruption, incompatible artifact, and invariant violation. Supervise each capability with bounded retry, jitter, circuit breaking, and restart budgets. Invariant violations and corruption fail closed; repeated transient faults disable only the affected capability and publish health state.

Use coordinated shutdown with capture stop, journal flush, display-safe state, and watchdog notification. Panics are bugs, captured at the process boundary, and never used for expected errors.

## Consequences

### Positive
- Failures are visible, isolated, and recoverable.
- Safety-relevant errors cannot be masked by generic retries.

### Negative
- Error classification and recovery tests add substantial work.
- Partial availability creates more UI and operational states.

### Neutral
- Process restart remains the final recovery layer, not the first response.

## Links

**Depends on**: [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md), [ADR-014](ADR-014-use-a-bounded-asynchronous-runtime-with-backpressure.md)
