# ADR-020: Build Local-First Observability and Support Bundles

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: observability, metrics, tracing, support

## Context

Production diagnosis requires capture health, queue delay, abstention, resource use, and artifact provenance. Cloud telemetry by default would violate local-first privacy and may expose sensitive states.

## Decision

Emit structured local logs, metrics, traces, health snapshots, and audit events with stable schemas and correlation IDs. Redact or omit CSI payloads, inferred states, participant labels, secrets, and identifiers by default. Use bounded rotating storage with policy-specific retention.

Provide a user-initiated support bundle generator that previews contents, de-identifies data, records consent, signs the bundle manifest, and exports only selected diagnostics. Remote telemetry is opt-in, minimized, documented, and independently disabled from core operation.

## Consequences

### Positive
- Field failures can be diagnosed without mandatory cloud collection.
- Performance and inference coverage become measurable.

### Negative
- Redaction and cardinality controls need continual testing.
- Some failures may require an explicitly authorized replay sample.

### Neutral
- Audit events and debug logs are separate data classes and stores.

## Links

**Depends on**: [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-014](ADR-014-use-a-bounded-asynchronous-runtime-with-backpressure.md), [ADR-015](ADR-015-supervise-failures-and-degrade-capabilities-independently.md), [ADR-018](ADR-018-use-an-append-only-journal-with-versioned-projections.md)
