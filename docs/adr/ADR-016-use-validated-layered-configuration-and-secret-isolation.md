# ADR-016: Use Validated Layered Configuration and Secret Isolation

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: configuration, secrets, profiles

## Context

Hardware profiles, capture settings, feature windows, policy, and deployment modes must be reproducible. Environment-variable sprawl and mutable ad hoc settings would invalidate replay and expose secrets.

## Decision

Define versioned strongly typed Rust configuration with layers: compiled safe defaults, signed deployment profile, local device configuration, and narrowly allowed runtime overrides. Validate the complete configuration before starting capture and record its non-secret digest with every session.

Secrets use a separate provider backed by protected files or hardware-supported keys; they never appear in config serialization, logs, diagnostics, or support bundles. Safety-, consent-, and claim-relevant settings cannot be overridden through ordinary environment variables.

## Consequences

### Positive
- Reproducible sessions and explicit configuration provenance.
- Secrets and non-secret settings have distinct lifecycles.

### Negative
- Migration tooling is required when schemas change.
- Deployment profile signing adds provisioning complexity.

### Neutral
- Developer profiles may relax operational settings but never prohibited-use policy.

## Links

**Depends on**: [ADR-002](ADR-002-keep-the-edge-runtime-local-and-deterministic.md), [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-013](ADR-013-define-the-cargo-workspace-and-crate-dependency-policy.md)
