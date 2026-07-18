# ADR-057: Build a Policy-Safe Local Operations UI with Vite, Tailwind CSS, and Three.js

**Status**: Implemented
**Date**: 2026-07-18
**Deciders**:
**Tags**: ui, vite, javascript, tailwind, threejs, visualization, accessibility, ipc, security

## Context

STE has a Rust-first edge runtime, stable operator commands, deterministic device projections, governed model and capability lifecycles, commissioning, validation, observability, recovery, and commercial-readiness controls. It does not yet have a full browser-based operator and participant experience. The CrowPi OLED and RGB outputs remain intentionally narrow and cannot provide the spatial, temporal, diagnostic, and administrative views needed to understand or operate the complete system.

The local UI must cover every functional area without becoming a second authority, bypassing Rust domain invariants, widening claims, or turning visually compelling graphics into unsupported scientific conclusions. In particular, the original concept's inferred valence, decision phase, cardiac coherence, generic cognitive load, and similar constructs remain disabled unless separately validated and promoted. A visualization may explain signal topology, data quality, provenance, latency, scope, and approved projections; it must not manufacture a label from raw data.

Three-dimensional rendering can make CSI propagation, room geometry, evidence flow, temporal windows, model scope, personalization neighborhoods, peripheral health, and release dependencies substantially easier to understand. It also introduces GPU cost, motion/accessibility risk, and a temptation to imply more physical or cognitive precision than the evidence supports.

## Decision

Build a local-first single-page application in `ui/` using:

- Vite with standards-based JavaScript ES modules;
- Tailwind CSS for a tokenized responsive design system;
- Three.js for bounded, optional, progressively enhanced 3D visualizations;
- Vitest and browser-level accessibility/contract tests;
- generated, versioned client bindings derived from Rust-owned schemas.

Do not add React or another component framework initially. Use small JavaScript modules, Web Components or DOM composition helpers, and explicit application state. A future framework adoption requires measured maintainability or capability evidence and a superseding ADR.

The UI is a presentation and command client. Rust remains authoritative for authentication, authorization, consent, policy, data lifecycle, model state, inference, persistence, auditing, commissioning, release readiness, and hardware control. The UI must not embed domain decisions in JavaScript.

### Trust boundary and connectivity

The production UI is served as immutable static assets by a minimal local Rust HTTP adapter or equivalent loopback-only presentation host. That adapter bridges to the authenticated Unix-domain IPC application boundary; the browser never receives filesystem or Unix-socket access.

The bridge must:

- bind only to an explicitly configured local interface and use a strict origin policy;
- authenticate sessions with short-lived, role-bound, device-bound credentials;
- enforce CSRF protection, origin checks, secure cookies, a restrictive Content Security Policy, and request/body/rate limits;
- translate only allowlisted versioned read models and application commands;
- reauthorize every command in Rust and preserve idempotency, confirmation, audit, and typed-error behavior;
- redact secrets, participant identifiers, raw payloads, and prohibited fields before serialization;
- use bounded Server-Sent Events or WebSocket streams only for approved read models, with sequence numbers, heartbeats, staleness, reconnection, and backpressure;
- default to unavailable rather than stale-looking data when disconnected or schema-incompatible.

Development may use Vite's server and deterministic fixtures. Production must not ship the Vite development server, source maps containing sensitive paths, remote CDNs, analytics, fonts, textures, model assets, or telemetry. All runtime assets are pinned, locally packaged, integrity checked, covered by the release SBOM, and usable offline.

### Information architecture and functional coverage

The application provides role-aware routes or workspaces for all implemented functional areas:

1. **Live overview** — authorization, capture state, approved projection, evidence age, abstention reason, operating-envelope status, signal quality, latency, queue health, peripheral state, and a prominent sensing indicator.
2. **Spatial signal topology** — a Three.js room scene showing the qualified device profile, AP/device placement, participant zone, bounded paths, packet continuity, contamination, and calibration geometry. It is explicitly illustrative unless backed by measured geometry and must display scale, provenance, age, and uncertainty.
3. **Radio acquisition** — adapter/firmware compatibility, packet rate, sequence gaps, link/AP health, replay controls, calibration binding, backpressure, and capture diagnostics. Raw CSI access is restricted to authorized diagnostic roles and is not a cognitive display.
4. **Signal observation** — approved amplitude/phase-derived observation windows, filters, contamination, missingness, artifact digests, event time, algorithm versions, and replay comparison. Visuals describe observations only.
5. **Physiology estimation** — respiration status, reference/promotion state, confidence calibration, scope, coverage, abstentions, and evidence lineage. It stays non-medical and disabled when real reference or capability evidence is absent. No cardiac or other unimplemented estimator is implied.
6. **State inference** — baseline/no-claim and separately promoted task-specific workload projections, temporal debounce, evidence lineage, claim level, scope, OOD state, and abstention. Raw model output never becomes UI copy. Valence and decision-phase production views are absent, not greyed-in as if imminent.
7. **Personalization memory** — participant-scoped anchors, provenance, feedback/corrections, retrieval neighborhoods, candidate adaptation lineage, prospective evidence, promotion/rollback, view/export/delete, cryptographic-erasure progress, and derived-index rebuild status. Cross-participant relationships are not queryable.
8. **Device interaction** — OLED/RGB simulator parity, text-plus-color projection preview, brightness/accessibility settings, touch debounce and anchor confirmation, DHT covariate, visible sensing indicator, physical-off state, and isolated peripheral faults. RGB cannot represent inferred valence.
9. **Consent and governance** — current participant/space/purpose/policy authorization, expiry, revocation, prohibited-use boundaries, retention, deletion, and audit-safe history. Consent changes and destructive operations require fresh Rust authorization and deliberate confirmation.
10. **Experiment validation** — frozen protocols, dataset cards, split/leakage checks, reference synchronization, metrics, negative results, promotion decisions, and reproducible report export. The UI cannot promote a capability without the named human decision and complete immutable evidence.
11. **Models and capabilities** — signed package integrity, compatibility, model card, calibration/OOD/selective-risk reports, registry lifecycle, known-answer health, activation/suspension/rollback, and signed capability-policy bindings. Experimental results use a persistent visual watermark and isolated route/state.
12. **Observability and reliability** — local metrics, traces, owned SLOs, queue/backpressure, resource/thermal/power state, watchdogs, faults, support-bundle preview, benchmark regressions, recovery posture, and data freshness. A Three.js pipeline graph may show flow and failure containment without exposing sensitive payloads.
13. **Commissioning and site qualification** — guided hardware/firmware/peripheral/power/thermal/link/packet/geometry/consent/storage/clock/calibration/interference checks, exact capability coverage, signed acceptance, blocked capabilities, recovery mode, and requalification lineage.
14. **Operations and data lifecycle** — status, doctor, capture test, hardware probe, calibration, replay, validation export, update, backup/restore, support bundle, participant data view/export/delete, recovery, reset, and decommissioning. Mutations require idempotency keys; destructive actions require dry-run plus typed confirmation.
15. **Security and incidents** — security posture, key/update state, vulnerability and incident status, credential-safe audit summaries, recovery contacts, capability suspension, recall/CAPA tasks, and evidence verification. Secrets and exploit details inappropriate to the current role are never rendered.
16. **Release and commercial readiness** — artifact manifest/SBOM coverage, compatibility, migrations, candidate/pilot/production channel identity, intended/prohibited claims, legal/pilot/HIL/support criteria, residual risks, exceptions, corrective actions, and the signed readiness decision. The current `NOT_APPROVED` decision must be unmissable and cannot be cosmetically overridden.

Navigation, route visibility, command affordances, and returned data are all role-scoped for participant, operator, support, validation, security, and release functions. Hiding a control in JavaScript is never treated as authorization.

### Visualization system

Three.js scenes are used where spatial or dependency relationships materially help comprehension:

- room and RF topology with confidence/quality volumes;
- event-time signal ribbons and window lineage;
- evidence provenance and bounded-context flow graphs;
- personalization neighborhoods scoped to one participant;
- model/capability dependency constellations;
- commissioning coverage and release-readiness dependency maps;
- fault propagation and containment simulations.

Every scene must have:

- a semantically equivalent table, text summary, or two-dimensional view;
- legends, units, timestamps, provenance, uncertainty, scope, and simulation/experimental labels;
- deterministic seeded fixture mode for tests and demonstrations;
- bounded geometry, draw calls, pixel ratio, history, memory, and update cadence;
- pause, reduced-motion, low-power, and WebGL-disabled fallbacks;
- no shader or color mapping that encodes a prohibited or unavailable construct;
- disposal of geometries, materials, textures, listeners, animation frames, and GPU contexts on route changes.

Prefer instancing, typed-array ring buffers, level of detail, offscreen computation where measured, and update-on-change rather than recreating scenes. WebGL failure must not block consent, authorization, safety state, recovery, or destructive-operation confirmation.

### Design system, accessibility, and responsive behavior

Tailwind design tokens define color, typography, spacing, elevation, density, focus, motion, and status semantics. Status always uses text/icon/shape in addition to color. The design targets WCAG 2.2 AA, keyboard-only operation, visible focus, screen readers, zoom/reflow, touch targets, color-vision deficiencies, reduced motion, high contrast, and configurable data density.

Use a distinctive technical visual language—layered translucent surfaces, topology grids, luminous evidence paths, and restrained depth—without sacrificing legibility. Critical authorization, abstention, stale, fault, experimental, and release-blocked states take precedence over decorative effects.

Layouts support CrowPi-attached displays where practical, tablets, laptops, and large operator displays. Dense diagnostics progressively disclose detail; the participant view remains calm, minimal, and free of unsupported interpretation.

### State, schemas, and failure handling

Maintain a small normalized client store containing only approved ephemeral read models, session state, view preferences, and pending command receipts. Do not persist sensitive domain data in `localStorage`, IndexedDB, service-worker caches, URLs, analytics, crash reports, or browser logs. Non-sensitive accessibility preferences may be persisted under a versioned schema.

Every read model carries schema version, source and emission time, sequence, provenance reference where permitted, scope/capability state, and staleness information. Unknown safety-relevant fields or incompatible versions fail closed. The UI renders explicit loading, disconnected, unauthorized, stale, unsupported, experimental, blocked, and fault states.

The UI must remain useful against deterministic simulators and fixtures, but fixture mode is visibly marked and cannot invoke production mutations or generate production evidence.

### Testing and release gates

Required automated coverage includes:

- unit tests for stores, formatters, policy-safe copy, reducers, and command receipts;
- generated-schema and Rust contract compatibility tests;
- route/role authorization and prohibited-field leakage tests;
- projection snapshot tests for every approved and abstention state;
- command tests for authorization, idempotency, confirmation, timeout, retry, and typed failures;
- Three.js deterministic scene, disposal, bounded-resource, WebGL-loss, reduced-motion, and fallback tests;
- accessibility tests plus manual keyboard, screen-reader, contrast, color-vision, and zoom review;
- responsive visual regression at supported viewport classes;
- CSP, dependency, license, SBOM, secret, XSS, CSRF, clickjacking, and hostile-payload checks;
- offline production-build, bundle-size, startup, frame-time, memory, long-session, and thermal/resource budgets on the reference Pi;
- end-to-end simulator tests for every functional workspace and physical HIL tests for device-control workflows.

A visually complete UI does not enable a capability. Production release remains governed by signed capability policy and ADR-055 acceptance evidence.

## Implementation Evidence

- `ui/` contains the Vite JavaScript and Tailwind application, all sixteen role-scoped workspaces, fail-closed client state, same-origin gateway client, fixture safeguards, and dynamically loaded Three.js scenes.
- `crates/ste-ui-gateway/` provides all sixteen versioned read-model areas, recursive prohibited-field enforcement, session/device/CSRF/origin checks, restrictive CSP validation, request bounds, role/command allowlists, idempotent application dispatch, bounded streams, loopback host policy, and exact static-asset verification.
- Topology, provenance, pipeline, release-readiness, participant-neighborhood, model-constellation, commissioning-coverage, and fault-containment scenes use deterministic seeded models, bounded resources, semantic fallbacks, reduced-motion/low-power behavior, WebGL failure recovery, and complete teardown.
- `scripts/validate-ui.sh` passes 32 JavaScript tests, 11 Rust adversarial tests, strict Clippy and rustdoc, a source-map-free production build, npm vulnerability audit, local-asset checks, and the visualization performance budget.
- Development-host scene-model construction measures approximately 0.032 ms per maximum bounded scene against the 5 ms budget. Three.js is isolated in a dynamic chunk of approximately 120 kB gzip; measured budgets do not justify additional complexity.
- [The local UI runbook](../operations/local-ui.md), [visualization budget](../benchmarks/adr-057-visualization-budget.md), and [implementation report](../reports/adr-057-visualization-implementation.md) document operation and evidence.

Manual screen-reader, color-vision, zoom/reflow, exact-CrowPi HIL, WebGL/GPU-memory, thermal, and long-session testing remain ADR-055 release evidence. Their absence does not enable production use; the UI continues to display the signed `NOT_APPROVED` decision.

## Alternatives Considered

### Render server-generated HTML only

Rejected as the sole approach because rich local topology, temporal, and diagnostic interaction benefits from a bounded client application. Server-rendered fallback summaries remain desirable for critical views.

### Use React with React Three Fiber

Deferred. It would improve component ecosystem and declarative scene composition but adds runtime and dependency weight without a demonstrated need. Plain Vite JavaScript and Three.js are sufficient for the initial local application.

### Make the UI a TypeScript sidecar

Rejected. Static browser assets are a presentation client, not a privileged TypeScript runtime sidecar. The UI receives no hardware or authoritative-store access and does not enter the capture-to-projection critical path.

### Expose a broad REST API directly from the Rust domain

Rejected because it would widen attack surface and duplicate application authorization. The bridge exposes only versioned read models and existing authenticated application commands.

### Stream raw CSI, features, and model tensors into the browser

Rejected for ordinary operation due to privacy, bandwidth, resource, scientific-interpretation, and claim-safety risks. Narrow diagnostics require explicit role authorization and redacted, bounded contracts.

## Consequences

### Positive

- Covers the complete product lifecycle through one coherent, role-aware local interface.
- Makes topology, provenance, scope, abstention, reliability, and readiness relationships understandable without weakening Rust authority.
- Enables visually ambitious offline operation with deterministic and accessible fallbacks.
- Reuses the stable command and projection boundaries rather than duplicating domain rules.
- Keeps unsupported constructs absent and makes experimental or blocked status visually explicit.

### Negative

- Introduces a browser, JavaScript toolchain, GPU renderer, dependency/SBOM surface, and additional security boundary.
- Three.js scenes require performance, memory, accessibility, and long-session testing on reference hardware.
- Generated contracts and the local HTTP/stream bridge require compatibility and lifecycle maintenance.
- A comprehensive UI substantially expands visual-regression, accessibility, and support obligations.

### Neutral

- Vite and Tailwind are build-time/development tools; production ships immutable static assets and locally packaged runtime JavaScript.
- Three.js is optional progressive enhancement, not a dependency of capture, inference, authorization, persistence, recovery, or safe physical projection.
- The UI may be high fidelity before scientific or commercial capabilities are approved; it must continue to show those capabilities as unavailable or blocked.

## Links

**Depends on**: [ADR-001](ADR-001-adopt-a-rust-first-modular-monolith.md), [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md), [ADR-010](ADR-010-use-deterministic-policy-approved-ui-projections.md), [ADR-012](ADR-012-use-versioned-contracts-and-an-anti-corruption-layer.md), [ADR-014](ADR-014-use-a-bounded-asynchronous-runtime-with-backpressure.md), [ADR-016](ADR-016-use-validated-layered-configuration-and-secret-isolation.md), [ADR-027](ADR-027-authenticate-and-authorize-local-administration.md), [ADR-040](ADR-040-version-peripheral-drivers-and-design-accessible-interactions.md), [ADR-044](ADR-044-declare-and-enforce-the-supported-operating-envelope.md), [ADR-051](ADR-051-provide-a-stable-cli-and-minimal-local-control-api.md), [ADR-053](ADR-053-gate-capabilities-with-signed-feature-policy.md), [ADR-055](ADR-055-require-a-production-readiness-review-and-acceptance-evidence.md)

**Related**: [ADR-042](ADR-042-isolate-and-supervise-typescript-sidecars.md), [DDD context map](../ddd/context-map.md), [Device Interaction context](../ddd/contexts/device-interaction.md), [implementation prompts](../implementation-prompts.md)
