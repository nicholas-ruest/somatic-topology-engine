# Bounded Context: Consent and Governance

## Purpose

Own authorization to sense, participant consent, purpose limitation, retention, deletion, audit, and prohibited-use policy. It makes privacy a runtime invariant for an ambient sensor.

## Aggregate: `SensingAuthorization`

**Identity:** `SensingAuthorizationId`

**Entities:** `ParticipantConsent`, `SpaceAuthorization`, `PurposeGrant`, `RetentionRule`, `Revocation`

**Value objects:** `ParticipantPseudonym`, `SpaceId`, `Purpose`, `DataClass`, `RetentionPeriod`, `ConsentVersion`, `PolicyVersion`, `AuthorizationState`, `RevocationReason`

### Invariants

1. Authorization is denied unless the space, purpose, and required participant consents are all active.
2. Consent is specific, versioned, time-bounded, and revocable.
3. Revocation stops new capture and schedules required deletion without waiting for an agent or cloud service.
4. Raw CSI, derived observations, physiology estimates, latent-state estimates, and anchors have explicit data classes and retention rules.
5. Identity inference, diagnosis, employment scoring, deception detection, covert sensing, and unrelated secondary use are prohibited.
6. Audit records are tamper-evident but exclude unnecessary sensitive payloads.
7. No TypeScript or external adapter may widen a granted purpose.

### Commands

- `GrantSensingAuthorization`
- `RecordParticipantConsent`
- `AuthorizePurpose`
- `RevokeConsent`
- `ApplyRetentionPolicy`
- `RecordDeletionCompletion`

### Events

- `SensingAuthorizationGranted`
- `ParticipantConsentRecorded`
- `SensingAuthorizationRevoked`
- `RetentionExpired`
- `ParticipantDeletionRequested`
- `ParticipantDeletionCompleted`

### Repository port

`SensingAuthorizationRepository` persists authorization state. `PolicyDecisionPoint` exposes fail-closed queries to other contexts. Encryption and key destruction are infrastructure capabilities invoked through audited ports.

## Rust boundaries

Policy evaluation and capture gating are in-process Rust and remain available offline. Administrative TypeScript tools may request policy commands but cannot bypass the Rust decision point.
