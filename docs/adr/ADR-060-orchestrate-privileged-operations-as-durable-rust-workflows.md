# ADR-060: Orchestrate Privileged Operations as Durable Rust Workflows

**Status**: Implemented
**Date**: 2026-07-18
**Deciders**:
**Tags**: workflows, commands, progress, receipts, recovery, operations

## Context

Commissioning, consent changes, model activation, calibration, deletion, experiment promotion, updates, backup/restore, incident response, recovery, reset, and decommissioning are multi-step operations. A synchronous HTTP request or browser-managed wizard cannot safely own their progress, retries, approval, crash recovery, compensation, or audit.

The current application commands return typed outcomes, but a complete UI needs durable progress, resumable steps, human decisions, evidence attachments, cancellation boundaries, and final receipts. JavaScript must not become the workflow authority.

## Decision

Implement a Rust workflow orchestration module using explicit, versioned workflow definitions and append-only workflow events. It is not a general scripting engine. Every workflow type is compiled Rust application logic with a finite state machine, permitted roles, required evidence, timeouts, retry/compensation rules, cancellation points, and terminal receipts.

Initial workflow types are:

- commissioning, site acceptance, and requalification;
- consent grant/renew/revoke and immediate capture stop;
- calibration and capture/replay diagnostics;
- model register/evaluate/promote/activate/health/suspend/rollback/revoke;
- signed capability-policy inspect/stage/activate/suspend;
- personalization anchor/correction/export/delete/erasure/rebuild and adaptation promotion/rollback;
- study/dataset/protocol validation, report export, named promotion/rejection;
- hardware probe, OLED/RGB/touch simulator, physical-off, and peripheral recovery;
- support-bundle preview/export;
- update stage/activate/health/rollback and key rotation;
- backup/restore verification, data deletion, recovery, factory reset, and decommissioning;
- incident declaration, evidence preservation, capability suspension, notification/recall/CAPA tasks.

### Workflow model

Each instance has a UUIDv7 identifier, workflow/schema version, exact type, scope, requester role/session, authorization/purpose reference, idempotency key, correlation/causation IDs, created/updated/expiry time, current step, percentage or indeterminate progress, permitted next actions, blocking reasons, evidence references, and terminal receipt.

Events are checksummed and appended atomically with projection checkpoints. Restart rebuilds the workflow projection and resumes only explicitly restartable steps. External effects use prepare/commit or idempotent application commands. A crash cannot silently repeat a destructive effect.

Step states include pending, ready, running, awaiting-human, awaiting-device, retry-wait, compensating, blocked, cancelled, succeeded, and failed. Progress is semantic; it must not fabricate a percentage when the remaining work is unknown.

### Authorization and confirmation

Reauthorize before every privileged or delayed step. Session expiry pauses rather than implicitly approving. Consent revocation and safety suspension preempt workflows immediately.

Destructive or claim-changing operations use server-generated confirmation challenges bound to workflow, exact scope, consequences, expiry, and current state. Typed confirmation, step-up authentication, named approval, dry-run, dual control, or physical action is required according to policy. Browser modal confirmation alone is insufficient.

### Receipts and progress

ADR-058 exposes workflow snapshots and bounded progress events. A receipt includes final outcome, exact affected resources, before/after state references, evidence/audit digests, warnings, recovery guidance, and whether compensation completed. Receipts never include secrets or deleted payloads.

Exact idempotent retry returns the same instance/receipt. Reusing a key with different scope or body is rejected. Cancellation is accepted only at declared safe points and returns an authoritative outcome.

### Isolation

Bound concurrency by workflow class and resource. Updates, restore, reset, model activation, and calibration take explicit locks with observable ownership/timeout. Long workflows cannot starve authorization revocation, capture supervision, audit, or safety controls.

## Consequences

### Positive

- Makes complex UI operations resumable, auditable, crash-safe, and role-correct.
- Provides real progress and receipts without placing authority in the browser.
- Unifies CLI, web, recovery, and physical interaction around the same application workflows.

### Negative

- Adds durable workflow schemas, migrations, locks, compensation, and operational recovery complexity.
- Human/device waiting states require careful expiry and notification behavior.
- Poorly designed workflows can become overly coupled orchestration logic.

### Neutral

- Short read-only commands remain synchronous.
- Workflow orchestration is local and deterministic; it is not Ruflo or a general agent framework.

## Validation

- State-machine transition, invariant, authorization, timeout, cancellation, compensation, and idempotency property tests.
- Crash/restart at every effect boundary; torn journal/checkpoint and migration tests.
- Concurrent/conflicting workflow, resource-lock, revocation-preemption, and starvation tests.
- Golden receipts and parity across CLI/web/recovery entry points.
- Destructive confirmation, dual-control, audit failure, and deleted-data leakage tests.

## Implementation evidence

Implemented by `ste-workflows`, with a closed 59-operation catalog, checksummed optimistic journal, deterministic projections, UUIDv7 instances, server-bound confirmation challenges, authorization and revocation checks, idempotency, scoped locks, prepare/commit effects, recovery, compensation, and bounded receipts. The gateway composition and browser client preserve expected versions, monotonic progress, conflicts, retries, cancellation, and terminal receipts. See `docs/reports/adr-060-workflow-validation.md` and `docs/benchmarks/adr-060-workflow-engine.md`.

## Links

**Depends on**: [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-014](ADR-014-use-a-bounded-asynchronous-runtime-with-backpressure.md), [ADR-018](ADR-018-use-an-append-only-journal-with-versioned-projections.md), [ADR-027](ADR-027-authenticate-and-authorize-local-administration.md), [ADR-028](ADR-028-govern-retention-export-backup-recovery-and-deletion.md), [ADR-051](ADR-051-provide-a-stable-cli-and-minimal-local-control-api.md), [ADR-058](ADR-058-run-a-production-local-web-gateway-over-rust-application-services.md)

**Related**: [ADR-026](ADR-026-use-signed-reproducible-updates-with-atomic-rollback.md), [ADR-033](ADR-033-govern-model-registration-promotion-activation-and-rollback.md), [ADR-043](ADR-043-use-guided-installation-site-qualification-and-acceptance.md), [ADR-046](ADR-046-establish-security-privacy-and-model-incident-response.md), [ADR-049](ADR-049-define-disaster-recovery-factory-reset-and-decommissioning.md)
