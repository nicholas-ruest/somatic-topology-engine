# ADR-027: Authenticate and Authorize Local Administration

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: authentication, authorization, api, administration

## Context

Configuration, exports, deletion, updates, diagnostics, and capability promotion are privileged actions. Binding a local API only to loopback is insufficient when local processes or physical users may be untrusted.

## Decision

Expose the smallest possible local administrative API over Unix-domain sockets by default. Authenticate clients with OS credentials and device-bound credentials where needed. Authorize explicit roles/capabilities for operator, participant, service, support, and release functions; deny by default.

Require step-up confirmation for export, deletion, factory reset, consent override attempts, model promotion, and update activation. Audit privileged actions without logging secrets or sensitive payloads. Rate-limit commands and protect against confused-deputy use by TypeScript sidecars.

## Consequences

### Positive
- Local compromise does not automatically grant every STE capability.
- Administrative actions are attributable and reviewable.

### Negative
- Provisioning and account recovery are more complex.
- Headless workflows need secure non-interactive credentials.

### Neutral
- Physical possession alone is not treated as participant consent.

## Links

**Depends on**: [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-024](ADR-024-maintain-a-living-threat-model-and-trust-boundaries.md), [ADR-025](ADR-025-use-device-identity-encryption-and-managed-keys.md)
