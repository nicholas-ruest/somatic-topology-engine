# ADR-046: Establish Security, Privacy, and Model Incident Response

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: incident-response, vulnerabilities, privacy, models

## Context

Production operation will encounter vulnerabilities, privacy complaints, compromised artifacts, model regressions, unsafe outputs, and potentially exploited devices. Response cannot be invented during an incident.

## Decision

Maintain coordinated incident plans for security, privacy, safety, model, data, and availability events. Define severity, on-call/ownership, evidence preservation, containment, device/model revocation, update channels, notification, recovery, postmortem, and regulatory/customer timelines.

Publish a vulnerability disclosure policy and supported-version window. Maintain device inventory and release/model reachability sufficient for targeted response without collecting behavioral telemetry. Test tabletop and technical exercises, including offline-device remediation and cryptographic key compromise.

## Consequences

### Positive
- High-impact failures have rehearsed containment and communication.
- Model and privacy incidents receive first-class handling.

### Negative
- Requires sustained staffing, inventory, and update support.
- Offline fleets complicate reachability and notification.

### Neutral
- Incident records follow strict access and retention policy.

## Links

**Depends on**: [ADR-020](ADR-020-build-local-first-observability-and-support-bundles.md), [ADR-024](ADR-024-maintain-a-living-threat-model-and-trust-boundaries.md), [ADR-026](ADR-026-use-signed-reproducible-updates-with-atomic-rollback.md), [ADR-033](ADR-033-govern-model-registration-promotion-activation-and-rollback.md), [ADR-045](ADR-045-maintain-a-safety-case-and-hazard-control-traceability.md)
