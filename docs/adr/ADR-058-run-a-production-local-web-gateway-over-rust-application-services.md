# ADR-058: Run a Production Local Web Gateway over Rust Application Services

**Status**: Implemented
**Date**: 2026-07-18
**Deciders**:
**Tags**: rust, http, sse, gateway, ipc, security, ui

## Context

ADR-057 created the static local UI, a transport-neutral `ste-ui-gateway`, and a fixture client. The browser still lacks a running production adapter connected to the Rust application services. It cannot open Unix-domain sockets, and exposing domain crates or a broad REST surface would bypass the supported application boundary.

The complete UI needs versioned snapshots, bounded live streams, authenticated commands, workflow progress, asset delivery, and reconnect behavior. The gateway will be a security boundary carrying consent, operational, model, data-lifecycle, and release functions. Loopback binding alone is not authentication.

## Decision

Add a Rust `ste-web-gateway` binary or runtime composition module. It serves the verified ADR-057 asset manifest and a deliberately small HTTP interface on an exact configured loopback address. It adapts browser requests to existing Rust application services and authenticated Unix-domain IPC; it never exposes domain repositories, aggregate mutation, raw filesystem paths, or arbitrary IPC methods.

Use a maintained Rust HTTP implementation selected through dependency/security review. Do not implement HTTP parsing manually. The production binary must not include Vite's development server.

Expose only:

- `GET /healthz` with payload-free process readiness;
- `GET /api/v1/session` for the current authenticated browser session and role capabilities;
- `GET /api/v1/read-models/{area}` for policy-filtered versioned snapshots;
- `GET /api/v1/streams/{stream}` as bounded Server-Sent Events;
- `POST /api/v1/commands/{command}` for allowlisted application commands;
- `GET /api/v1/workflows/{id}` and an SSE workflow-progress stream;
- immutable hashed static assets and the generated asset manifest.

The exact endpoint set is generated from Rust route metadata and included in the API support matrix. Unknown versions, routes, fields, content types, or safety-relevant enum values fail closed.

### Transport security

The gateway must enforce:

- exact loopback binding by default and explicit refusal of wildcard/non-loopback addresses;
- locally provisioned HTTPS where browser platform features or deployment policy require it;
- exact `Host` and `Origin` allowlists, CSRF tokens, secure role-session cookies, `SameSite=Strict`, and no credentials in URLs;
- restrictive CSP, `frame-ancestors 'none'`, nosniff, referrer, permissions, cache, and cross-origin isolation headers as compatible;
- bounded URI, header, body, JSON depth, field count, connection, request-rate, and stream counts;
- request deadlines, cancellation, backpressure, slow-client eviction, heartbeat, sequence, resume cursor, and observable loss;
- structured typed errors without secrets, participant identifiers, stack traces, filesystem paths, raw frames, features, or model tensors;
- fresh Rust authorization for every snapshot, stream subscription, and command rather than trusting route visibility;
- audit correlation, causation, role, session, idempotency, and command receipt references;
- graceful restart and session/stream invalidation on policy, consent, role, key, or schema change.

### Application integration

Define Rust `ReadModelProvider`, `StreamProvider`, `CommandExecutor`, `WorkflowQuery`, `SessionAuthority`, and `AssetProvider` ports. Infrastructure adapters call existing context application services or the authenticated IPC client. The web gateway must not import private domain infrastructure or directly write journals, vectors, model registries, device ports, or configuration.

Snapshot and stream payloads are ADR-012/019 contracts generated from Rust types. Generate JSON Schema and JavaScript bindings; validate golden compatibility fixtures in both Rust and UI CI.

Commands preserve the exact CLI/application semantics: authorization, idempotency, dry-run, confirmation, typed exits, immutable audit, and redaction. HTTP success never implies domain success; the body contains the authoritative typed receipt.

### Availability and deployment

The gateway is independently supervised and bounded. Its crash, restart, resource exhaustion, or deliberate shutdown cannot stop capture, authorization enforcement, persistence, inference, physical safety projection, or recovery-mode CLI. Recovery remains available in-process or over the authenticated Unix socket.

Package the exact static assets and gateway binary in signed release evidence. Run offline by default. No remote CDN, analytics, telemetry, fonts, scripts, images, or runtime package fetches are allowed.

## Consequences

### Positive

- Converts the fixture UI into a real local application without moving authority into JavaScript.
- Centralizes browser security, schemas, backpressure, and redaction.
- Reuses existing application commands and audit semantics.
- Gateway failure is isolated from the Rust sensing and safety path.

### Negative

- Adds an HTTP parser/server dependency and a high-value attack surface.
- Session, certificate, CSP, streaming, compatibility, and slow-client behavior require continuous testing.
- More Rust composition adapters and read projections must be maintained.

### Neutral

- Loopback is the default deployment boundary, not proof of identity.
- Remote administration remains out of scope and requires a separate secured bridge and ADR.

## Validation

- Host/origin/CSRF/session/authz, request-smuggling, traversal, slowloris, oversized/deep JSON, decompression, XSS, cache, and redaction tests.
- Golden Rust/JavaScript schema compatibility and unsupported-version rejection.
- SSE reconnect, resume, duplicate, gap, heartbeat, cancellation, backpressure, and slow-consumer tests.
- Command idempotency, conflict, timeout, cancellation, denial, and receipt parity with `ste-cli`.
- Static asset exact-manifest, CSP, offline, source-map, SBOM, license, and reproducibility gates.
- Gateway restart/load/soak tests proving the core runtime remains healthy.

## Implementation evidence

Implemented by `ste-web-gateway` and its `ProductionServices` composition adapter. The loopback-only Axum binary verifies the generated UI manifest before serving, requires explicit session/CSRF/authorization/purpose bindings, and connects the real bounded query plane and durable workflow engine. Hostile HTTP tests and an authenticated composition smoke cover session, SSE, bounded history, workflow launch, transition, and projection. The repository-wide test, strict Clippy, rustdoc, dependency-policy, audit, and secret/architecture gates pass.

## Links

**Depends on**: [ADR-012](ADR-012-use-versioned-contracts-and-an-anti-corruption-layer.md), [ADR-014](ADR-014-use-a-bounded-asynchronous-runtime-with-backpressure.md), [ADR-019](ADR-019-evolve-contracts-with-explicit-compatibility-rules.md), [ADR-027](ADR-027-authenticate-and-authorize-local-administration.md), [ADR-051](ADR-051-provide-a-stable-cli-and-minimal-local-control-api.md), [ADR-057](ADR-057-build-a-policy-safe-local-operations-ui-with-vite-tailwind-and-threejs.md)

**Related**: [ADR-024](ADR-024-maintain-a-living-threat-model-and-trust-boundaries.md), [ADR-052](ADR-052-separate-audit-security-and-diagnostic-records.md)
