# ADR-055: Require a Production Readiness Review and Acceptance Evidence

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: production-readiness, acceptance, release-gate

## Context

Passing individual tests does not demonstrate that the integrated system, organization, evidence, and support model are ready for commercial production.

## Decision

Require a named cross-functional production readiness review before pilot and general availability. The review evaluates accepted/implemented ADRs; architecture and DDD validation; scientific gates; operating envelope; hazards; threat model; privacy; legal/regulatory; licenses; tests; fuzz/HIL/soak; benchmarks/SLOs; update/rollback; recovery; observability; support; manufacturing/installation; documentation; incident exercises; and pilot economics.

Every criterion links to immutable evidence, owner, date, scope, and residual risk. Exceptions are time-bounded, approved, visible, and cannot waive consent, prohibited use, unacceptable safety risk, artifact integrity, or unsupported claims. Readiness expires after material change.

## Consequences

### Positive
- “Production ready” becomes an evidence-backed decision.
- Organizational and commercial gaps cannot hide behind code completion.

### Negative
- Reviews can block launch and require expensive remediation.
- Evidence collection needs dedicated ownership.

### Neutral
- Readiness is defined for a particular release, market, hardware, and operating envelope.

## Links

**Depends on**: [ADR-041](ADR-041-use-reproducible-ci-cd-and-evidence-bearing-releases.md), [ADR-045](ADR-045-maintain-a-safety-case-and-hazard-control-traceability.md), [ADR-047](ADR-047-control-product-claims-regulatory-classification-and-quality-records.md), [ADR-048](ADR-048-design-commercial-support-warranty-and-field-operations.md), [ADR-054](ADR-054-version-documentation-runbooks-and-support-matrices.md)
