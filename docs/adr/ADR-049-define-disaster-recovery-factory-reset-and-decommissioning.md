# ADR-049: Define Disaster Recovery, Factory Reset, and Decommissioning

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: recovery, reset, decommissioning, lifecycle

## Context

Devices can suffer corrupt media, lost credentials, failed updates, ownership transfer, or end-of-life. Recovery must not resurrect deleted data or leave sensing credentials behind.

## Decision

Define recovery tiers: service restart, projection rebuild, journal recovery, signed reinstall, participant-approved backup restore, and factory reset. Each tier documents preserved and destroyed data, required authorization, verification, and audit behavior.

Factory reset revokes device credentials where reachable, destroys user-data keys, clears journals/vectors/configuration, restores a signed base image, and returns to visibly unprovisioned capture-disabled state. Decommissioning adds media sanitization or destruction, inventory retirement, certificate revocation, and disposal guidance. Test corrupt/no-backup and offline scenarios.

## Consequences

### Positive
- Failure and ownership transfer have safe, documented endpoints.
- Reset cannot silently retain participant evidence.

### Negative
- Strong erasure may make recovery impossible by design.
- Certificate revocation is harder for permanently offline devices.

### Neutral
- Recovery objectives vary by data class and customer contract.

## Links

**Depends on**: [ADR-025](ADR-025-use-device-identity-encryption-and-managed-keys.md), [ADR-026](ADR-026-use-signed-reproducible-updates-with-atomic-rollback.md), [ADR-028](ADR-028-govern-retention-export-backup-recovery-and-deletion.md), [ADR-039](ADR-039-handle-power-thermal-storage-and-watchdog-failures.md)
