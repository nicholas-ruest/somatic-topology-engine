# ADR-009: Enforce Consent, Privacy, and Prohibited Use in the Domain

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: privacy, consent, governance, safety

## Context

Ambient RF sensing can reveal presence, behavior, physiology, and possibly identity without a worn device. Local processing and absence of cameras reduce some risks but do not create consent or eliminate surveillance harms.

## Decision

Make `SensingAuthorization` a domain aggregate and fail closed. Capture requires active authorization for the space, purpose, and participants. Revocation stops new capture immediately and triggers retention/deletion workflows.

Classify raw CSI, observations, physiology, state estimates, anchors, and audit data separately. Default to local encryption, minimal retention, participant access/correction/deletion, a visible sensing indicator, and a physical off control.

Prohibit identity inference, clinical diagnosis, employment scoring, deception detection, covert sensing, and unrelated secondary use. External adapters cannot widen purpose or bypass the Rust policy decision point.

## Consequences

### Positive

- Privacy constraints are testable runtime invariants rather than policy prose.
- Enables fine-grained deletion and retention by data class.

### Negative

- Multi-person spaces and consent changes complicate operation.
- Tamper-evident audit and cryptographic erasure require careful key design.

### Neutral

- Jurisdiction-specific legal and ethics review remains necessary.

## Links

**Depends on**: [ADR-002](ADR-002-keep-the-edge-runtime-local-and-deterministic.md)
**Related**: [Consent and Governance context](../ddd/contexts/consent-governance.md), [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md)
