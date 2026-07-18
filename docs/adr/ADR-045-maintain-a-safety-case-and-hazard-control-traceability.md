# ADR-045: Maintain a Safety Case and Hazard-Control Traceability

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: safety, hazards, assurance, traceability

## Context

False certainty, covert sensing, stale displays, incorrect anchors, thermal faults, bad updates, and misuse can harm users even if STE is positioned as non-medical.

## Decision

Maintain a hazard analysis and structured safety case for intended use and reasonably foreseeable misuse. For each hazard record severity, likelihood, detectability, controls, residual risk, verification, owner, and user-facing warning. Trace controls to ADRs, domain invariants, code, tests, release evidence, and field monitoring.

Define safe states for unauthorized capture, uncertain inference, peripheral failure, storage corruption, overtemperature, update failure, and revoked models. Require safety review for new capabilities and changed claim levels. Unacceptable residual risk blocks release regardless of feature completeness.

## Consequences

### Positive
- Safety arguments become reviewable and test-backed.
- Cross-cutting mitigations remain visible through implementation.

### Negative
- Assurance maintenance is substantial and requires qualified review.
- Some features may be delayed or prohibited.

### Neutral
- A safety case supports but does not itself establish regulatory compliance.

## Links

**Depends on**: [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md), [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-015](ADR-015-supervise-failures-and-degrade-capabilities-independently.md), [ADR-024](ADR-024-maintain-a-living-threat-model-and-trust-boundaries.md), [ADR-044](ADR-044-declare-and-enforce-the-supported-operating-envelope.md)
