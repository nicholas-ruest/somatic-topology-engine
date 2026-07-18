# ADR-054: Version Documentation, Runbooks, and Support Matrices

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: documentation, runbooks, support, drift

## Context

Hardware profiles, firmware compatibility, model scope, installation, recovery, privacy, and claims change by release. Unversioned documentation can instruct unsafe or unsupported behavior.

## Decision

Treat operator, participant, installer, developer, API, security, privacy, model-card, dataset-card, incident, recovery, and support documentation as release artifacts. Bind each document set to software/model/hardware versions and generate support matrices from authoritative manifests where possible.

CI checks links, contract examples, CLI examples, configuration keys, ADR references, prohibited claims, and stale version markers. Runbooks include prerequisites, safe states, rollback, evidence collection, escalation, and data-handling constraints. Documentation changes affecting use or claims receive the same review as product changes.

## Consequences

### Positive
- Field behavior and claims match the installed release.
- Drift becomes testable.

### Negative
- Documentation requires continuous engineering ownership.
- Supporting multiple release branches multiplies maintenance.

### Neutral
- Generated reference material does not replace reviewed conceptual and safety guidance.

## Links

**Depends on**: [ADR-019](ADR-019-evolve-contracts-with-explicit-compatibility-rules.md), [ADR-041](ADR-041-use-reproducible-ci-cd-and-evidence-bearing-releases.md), [ADR-047](ADR-047-control-product-claims-regulatory-classification-and-quality-records.md), [ADR-048](ADR-048-design-commercial-support-warranty-and-field-operations.md)
