# ADR-062: Use Real Browser Identity, Role Sessions, and Safe Workspace Preferences

**Status**: Proposed
**Date**: 2026-07-18
**Deciders**:
**Tags**: identity, sessions, roles, preferences, browser, privacy

## Context

The fixture UI changes roles with an in-page selector. A production participant, installer, operator, support engineer, validator, security responder, or release reviewer needs a real authenticated session. Role selection in JavaScript cannot establish authority.

Users also need persistent accessibility settings, chart preferences, saved filters, and workspace layouts. Browser storage can leak participant data, operational history, or sensitive identifiers and can retain stale authorization after logout or deletion.

## Decision

Implement a Rust `SessionAuthority` integrated with ADR-027 device/operator identity and ADR-058. Browser sessions are short-lived, server-side, opaque, role- and device-bound, purpose-scoped, revocable, and reauthorized on every request. The browser receives only a secure session cookie and CSRF binding; it never receives a reusable device credential or authorization token in JavaScript-readable storage.

### Identity and role sessions

Support explicit identity classes: participant, installer/operator, support, validation, security, release, and local service. Each authentication mechanism and recovery path is documented and separately threat-modeled. Physical possession or access to loopback does not grant participant consent or administrator role.

The session records identity reference, active role, permitted role set, device/deployment, consent/purpose scope where applicable, issue/expiry/last-reauthentication time, authentication strength, policy version, CSRF binding, and revocation generation. Sensitive identity attributes remain server-side.

Role switching is a server command. It requires membership in the permitted set and may require step-up authentication. The response rotates session and CSRF identifiers, invalidates streams and pending privileged UI state, and returns a fresh capability map. The frontend role selector becomes a request, not local authority.

Require step-up authentication or named approval for exports, deletion, consent changes, model/capability promotion, update activation, restore, reset, incident/recall actions, and release decisions as policy defines. Idle, absolute, device, policy, consent, and revocation expiry terminate the session and streams promptly.

### Authentication UX and recovery

Provide sign-in, role selection, step-up, locked, expired, revoked, logout, and recovery views with typed, non-enumerating errors. Rate-limit failures and audit security events without storing entered secrets. Do not offer insecure security questions or place secrets in command-line arguments, URLs, browser logs, analytics, or support bundles.

Account/device credential recovery is an explicit offline operational workflow with identity verification and payload-minimized evidence. Recovery cannot alter participant consent or waive prohibited-use policy.

### Preferences and layouts

Separate preferences into:

- **local non-sensitive**: theme, contrast, reduced motion, font scale, density, chart rendering preference, sidebar state;
- **server-synchronized non-sensitive**: workspace layout version, visible permitted panels, saved bounded filters, table columns, default time range;
- **forbidden to persist in the browser**: participant identifiers, consent records, CSI/history samples, evidence/model data, anchors, workflow bodies/receipts, audit/security records, secrets, roles, or capability decisions.

Local preferences use one versioned namespaced object with a strict allowlist, size limit, migration, reset, and corruption fallback. Do not use IndexedDB, service workers, or arbitrary key/value persistence without a later decision. Logout and role change clear ephemeral state, query caches, streams, dialogs, command drafts, and unauthorized layouts.

Server layouts are scoped to identity and role, versioned, size/bounds checked, and optimistic-concurrency protected. They contain only panel identifiers and presentation configuration. Missing/removed permissions automatically prune inaccessible panels. Preferences never widen data or commands.

### Client state boundaries

Keep live/read-model data in bounded memory only. URLs may contain route, non-sensitive view mode, bounded time range, and opaque non-sensitive artifact references; never secrets, participant labels, raw queries, command bodies, or receipts. Crash/error reporting is local, redacted, opt-in for export, and contains no application state dump.

## Consequences

### Positive

- Replaces the fake role selector with attributable, revocable production sessions.
- Makes preferences useful without turning browser storage into a shadow database.
- Role changes and logout reliably remove unauthorized state and streams.
- Centralizes step-up and session expiry across web workflows.

### Negative

- Identity provisioning, recovery, session storage, key rotation, and multi-role UX add operational complexity.
- Strict cache clearing can reduce convenience and requires thorough browser testing.
- Offline headless and participant workflows need distinct authentication designs.

### Neutral

- The initial deployment may have one human assigned multiple roles, but each active session has one explicit role.
- Preferences are not authoritative configuration.

## Validation

- Session fixation, theft, replay, CSRF, role escalation, logout, expiry, revocation, step-up, recovery, and rate-limit tests.
- Role-switch stream/cache/dialog/draft invalidation and cross-role leakage tests.
- Preference allowlist, migration, corruption, quota, concurrent update, permission pruning, and reset property tests.
- Browser storage canaries proving forbidden fields and payloads never persist.
- Accessible authentication, timeout warning, step-up, recovery, and error-flow task tests.

## Links

**Depends on**: [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-016](ADR-016-use-validated-layered-configuration-and-secret-isolation.md), [ADR-025](ADR-025-use-device-identity-encryption-and-managed-keys.md), [ADR-027](ADR-027-authenticate-and-authorize-local-administration.md), [ADR-052](ADR-052-separate-audit-security-and-diagnostic-records.md), [ADR-058](ADR-058-run-a-production-local-web-gateway-over-rust-application-services.md)

**Related**: [ADR-028](ADR-028-govern-retention-export-backup-recovery-and-deletion.md), [ADR-057](ADR-057-build-a-policy-safe-local-operations-ui-with-vite-tailwind-and-threejs.md), [ADR-061](ADR-061-build-the-complete-interactive-ste-operations-workbench.md)
