# ADR-048: Design Commercial Support, Warranty, and Field Operations

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: commercial, support, warranty, operations

## Context

Commercial viability depends on installation success, support cost, repair, returns, update coverage, customer expectations, and privacy-preserving diagnosis—not only technical accuracy.

## Decision

Define supported hardware/revisions, regions, installation model, warranty, repair/replace process, spare strategy, end-of-life, software/model support window, security-update commitment, response targets, escalation, and customer data-handling agreement.

Instrument opt-in, privacy-safe field quality metrics and support bundles sufficient to measure installation failure, abstention, reliability, returns, and support effort. Maintain runbooks and trained support boundaries; support staff cannot bypass consent or retrieve raw sensitive data by default. Feed anonymized operational findings into risk, validation, and roadmap review.

## Consequences

### Positive
- Unit economics and customer experience are measurable.
- Support workflows preserve privacy and architecture controls.

### Negative
- Long-term support, inventory, and staffing create material cost.
- Narrow operating envelopes can increase returns if sales qualification is weak.

### Neutral
- Commercial go/no-go requires pilot evidence beyond engineering release gates.

## Links

**Depends on**: [ADR-020](ADR-020-build-local-first-observability-and-support-bundles.md), [ADR-041](ADR-041-use-reproducible-ci-cd-and-evidence-bearing-releases.md), [ADR-043](ADR-043-use-guided-installation-site-qualification-and-acceptance.md), [ADR-046](ADR-046-establish-security-privacy-and-model-incident-response.md), [ADR-047](ADR-047-control-product-claims-regulatory-classification-and-quality-records.md)
