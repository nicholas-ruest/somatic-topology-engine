# ADR-047: Control Product Claims, Regulatory Classification, and Quality Records

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: regulatory, claims, quality, commercial

## Context

Terms such as BCI, cognitive load, emotional valence, cardiac coherence, stress detection, and health monitoring may create scientific, consumer-protection, biometric, employment, or medical-device obligations depending on use and jurisdiction.

## Decision

Define intended use, excluded uses, target markets, users, environments, and claim levels before commercial release. Obtain qualified legal/regulatory review per jurisdiction and reassess whenever claims, models, users, or deployment contexts change.

Maintain a claim-evidence matrix linking every UI, packaging, sales, API, and documentation statement to promoted validation evidence and operating envelope. Prohibit medical diagnosis/treatment and high-impact decision use unless a separate approved regulatory program is completed. Retain design, risk, verification, release, complaint, and change-control records under a proportionate quality system.

## Consequences

### Positive
- Commercial language cannot outrun evidence.
- Regulatory and quality obligations are planned rather than discovered late.

### Negative
- Legal review and quality records increase cost and constrain marketing.
- Jurisdictional differences may require product variants.

### Neutral
- Calling STE “wellness” or “research” does not override actual functionality and marketing.

## Links

**Depends on**: [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md), [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-044](ADR-044-declare-and-enforce-the-supported-operating-envelope.md), [ADR-045](ADR-045-maintain-a-safety-case-and-hazard-control-traceability.md)
