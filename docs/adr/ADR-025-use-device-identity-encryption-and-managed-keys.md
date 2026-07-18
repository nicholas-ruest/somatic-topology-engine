# ADR-025: Use Device Identity, Encryption, and Managed Keys

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: cryptography, keys, identity, encryption

## Context

Local storage contains sensitive evidence and policy state. Support exports, updates, and administrative commands need an authentic device identity without embedding shared fleet secrets.

## Decision

Provision each device with a unique asymmetric identity and hardware-backed key storage where available, with a protected software fallback explicitly labeled by assurance level. Encrypt sensitive data at rest with per-data-class envelope keys and authenticated encryption; authenticate local administrative channels and signed exports.

Define key generation, rotation, backup eligibility, revocation, recovery, zeroization, factory reset, and cryptographic erasure. Never derive user-data encryption solely from a device serial or user password. Use maintained, reviewed cryptographic libraries and algorithm agility through versioned envelopes.

## Consequences

### Positive
- Limits fleet-wide compromise and supports verifiable artifacts.
- Retention deletion can use key destruction where appropriate.

### Negative
- Lost keys may make data intentionally unrecoverable.
- Hardware-backed assurance varies by Pi/CrowPi configuration.

### Neutral
- Encryption does not replace authorization or minimize data collection.

## Links

**Depends on**: [ADR-016](ADR-016-use-validated-layered-configuration-and-secret-isolation.md), [ADR-018](ADR-018-use-an-append-only-journal-with-versioned-projections.md), [ADR-024](ADR-024-maintain-a-living-threat-model-and-trust-boundaries.md)
