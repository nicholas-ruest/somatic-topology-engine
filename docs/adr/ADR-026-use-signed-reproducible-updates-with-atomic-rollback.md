# ADR-026: Use Signed Reproducible Updates with Atomic Rollback

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: updates, rollback, signing, provisioning

## Context

Production devices require security and model updates without bricking the Pi, losing policy, or mixing incompatible firmware, schemas, and models.

## Decision

Distribute signed release bundles containing OS/firmware compatibility, binaries, configuration schemas, migrations, models, SBOMs, provenance, and rollback constraints. Verify signature, device compatibility, free space, power state, migrations, and artifact digests before activation.

Use an A/B or equivalent atomic deployment strategy with health-confirmed commit and automatic rollback. Protect against unauthorized downgrade while permitting signed emergency rollback to explicitly supported versions. Separate software, model, and policy activation but validate their compatibility matrix as one release candidate.

## Consequences

### Positive
- Failed releases recover without manual media replacement.
- Every running artifact maps to a signed release manifest.

### Negative
- Requires extra storage and complex migration/rollback testing.
- Some irreversible data migrations must be deferred or dual-written.

### Neutral
- Offline update media follows the same verification path as network delivery.

## Links

**Depends on**: [ADR-019](ADR-019-evolve-contracts-with-explicit-compatibility-rules.md), [ADR-023](ADR-023-govern-dependencies-licenses-and-software-supply-chain.md), [ADR-025](ADR-025-use-device-identity-encryption-and-managed-keys.md)
