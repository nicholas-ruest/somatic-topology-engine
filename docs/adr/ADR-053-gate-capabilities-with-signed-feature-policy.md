# ADR-053: Gate Capabilities with Signed Feature Policy

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: feature-flags, capabilities, policy, rollout

## Context

Engineering code may contain experimental, pilot, or jurisdiction-limited features that must not activate merely because a binary supports them. Ordinary feature flags are insufficient for scientific and regulatory gates.

## Decision

Represent runtime capability enablement as signed policy bound to device/deployment profile, software/model versions, validation promotion, jurisdiction, consent purpose, operating envelope, and expiration. The Rust policy decision point evaluates all conditions locally and defaults disabled.

Separate compile-time inclusion, operator rollout, participant choice, scientific promotion, and regulatory authorization. Use staged rollout with measurable rollback criteria. Experimental modes are visibly marked, isolate their data, and cannot emit production claim projections.

## Consequences

### Positive
- Shipping code does not automatically ship a claim.
- Pilot and jurisdictional rollout remain auditable and reversible.

### Negative
- Policy signing and compatibility add release complexity.
- Misconfigured policy can disable expected capabilities.

### Neutral
- Feature policy narrows capabilities; it cannot bypass a failed validation or consent gate.

## Links

**Depends on**: [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md), [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-016](ADR-016-use-validated-layered-configuration-and-secret-isolation.md), [ADR-026](ADR-026-use-signed-reproducible-updates-with-atomic-rollback.md), [ADR-033](ADR-033-govern-model-registration-promotion-activation-and-rollback.md), [ADR-047](ADR-047-control-product-claims-regulatory-classification-and-quality-records.md)
