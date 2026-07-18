# ADR-061: Build the Complete Interactive STE Operations Workbench

**Status**: Implemented
**Date**: 2026-07-18
**Deciders**:
**Tags**: ui, workbench, workflows, charts, forms, operations

## Context

ADR-057 implements the visual shell and representative functional workspaces, but most content remains fixture summaries. A complete product UI requires live analytical tools, real forms, dialogs, workflow progress, drill-down navigation, and safe command execution across every bounded context.

A single dashboard cannot adequately support participants, installers, operators, validation scientists, support, security, and release reviewers. Conversely, independently designed pages would create inconsistent safety language, confirmation, provenance, filtering, and authorization behavior.

## Decision

Evolve `ui/` into a role-aware operations workbench backed exclusively by ADR-058/059/060 contracts. Retain Vite JavaScript, Tailwind CSS, and progressively enhanced Three.js. Use reusable view primitives and explicit state rather than embedding Rust domain rules in client components.

### Shared interaction framework

Implement:

- workspace overview, detail routes, nested tabs, deep links to non-sensitive identifiers, and breadcrumbs;
- responsive split panes, drawers, inspectors, command palettes, tables, timelines, charts, comparison grids, forms, and dialogs;
- a synchronized time cursor, range, filters, source mode, quality masks, bookmarks, and live/replay controls;
- virtualized bounded tables and lists with cursor pagination;
- typed loading, empty, disconnected, stale, unauthorized, out-of-scope, experimental, blocked, conflict, partial, and fault states;
- provenance/evidence inspectors showing permitted digests, versions, clocks, scope, and lineage;
- workflow launch, step forms, progress, human-action requests, cancellation, retry, recovery guidance, and final receipts;
- URL-safe non-sensitive view state and versioned non-sensitive workspace layouts from ADR-062;
- export/print views that preserve watermarks, provenance, units, scope, and blocked/experimental status.

Every displayed metric provides definition, units, time basis, data source, quality, and freshness. Every destructive or claim-changing command previews exact scope and consequences. The UI never treats a disabled button, hidden route, client validation, or modal as authorization.

### Functional workbenches

1. **Live system and CSI** — live packet/continuity/quality/resource charts, stream pause/resume, bounded diagnostic capture, channel/link details, geometry/topology, contamination overlays, and crosshair inspection. Raw CSI requires diagnostic authorization and prominent observation-only language.
2. **Signal and replay laboratory** — multi-track amplitude/phase-derived observation charts, window/artifact inspectors, filter/config versions, synchronized history scrubbing, play/pause/seek/step/speed, event markers, baseline comparison, numerical deltas, and reproducible replay manifest.
3. **Commissioning** — step-by-step hardware, firmware, peripheral, power, thermal, AP/link, packet, geometry, consent, storage, clock, calibration, interference, and capability coverage forms; evidence capture; retry/remediation; signed acceptance and requalification comparison.
4. **Consent and participant rights** — understandable policy review, participant/space/purpose selection, grant/renew/revoke, expiry, prohibited-use notices, retention choice, history, export, deletion status, and immediate sensing stop. Revocation is always directly reachable.
5. **Models, calibration, and capabilities** — package/card/integrity/compatibility inspectors; calibration/OOD/selective-risk charts; registry timeline; known-answer health; activate/suspend/rollback/revoke workflows; signed capability-policy scope, jurisdiction, purpose, envelope, expiry, and experimental isolation.
6. **Personalization** — participant-only anchor timeline and Three.js neighborhood, provenance, corrections, feedback quality, retrieval explanation, adaptation lineage/comparison, candidate promotion/rollback, export, delete, key-erasure, and index-rebuild progress. Cross-participant navigation is structurally absent.
7. **Experiment validation** — protocol editor with immutable freeze transition, dataset-card forms, split/leakage explorer, reference synchronization, metric/baseline/comparison charts, negative-result registry, reproducible report preview/export, and named promotion/rejection workbench.
8. **Device interaction** — interactive OLED and text-plus-color preview, permitted RGB modes, brightness/accessibility controls, touch gesture/debounce visualization, anchor confirmation, DHT covariate history, visible sensing indicator, physical-off, peripheral fault injection, simulator/hardware profile comparison, and HIL evidence.
9. **Operations and data lifecycle** — doctor, capture test, probe, calibration, replay, support preview/export, update, backup/restore verification, retention, export, deletion, recovery, reset, and decommission workflows with progress and receipts.
10. **Reliability and security** — SLO/resource timelines, queue and fault drill-down, watchdogs, benchmark comparisons, incident timeline, signed evidence verification, key/update posture, capability suspension, notification/recall/CAPA workflow, and role-redacted audits.
11. **Release evidence** — manifest/SBOM/card/compatibility/migration/support coverage, exact channel digest comparison, residual risks, exceptions, corrective actions, and signed readiness decision. It is accessible from status rather than dominating routine navigation.

### Forms and dialogs

Forms are generated or hand-bound to versioned Rust command schemas. Server responses remain authoritative. Support save-as-draft only when the workflow explicitly permits it. Prevent accidental loss, duplicate submission, and stale-version overwrite with workflow version/ETag checks.

Dialogs use focus trapping, labelled consequences, escape/cancel behavior where safe, and server-issued confirmation challenges. Long forms support step summaries and validation-error navigation. Sensitive values are never echoed after submission.

### Accessibility and performance

Target WCAG 2.2 AA. Provide keyboard, screen-reader, contrast, color-vision, zoom/reflow, touch, reduced-motion, low-power, and non-WebGL equivalents. Charts expose semantic summaries and downloadable authorized tables. Do not rely on hover.

Lazy-load workspaces, charts, editors, and Three.js; suspend hidden live views; virtualize long data; bound caches; dispose GPU resources; and enforce bundle, startup, frame-time, heap, network, Pi thermal, and long-session budgets.

## Consequences

### Positive

- Turns the current visual shell into a complete participant, scientific, installation, and operations product.
- Standardizes complex interactions, evidence inspection, progress, receipts, and safety language.
- Preserves the Rust authority and supports live/history/replay without duplicating domain logic.

### Negative

- Represents a large product surface with significant UX, accessibility, test, documentation, and support cost.
- Complex charts and workflows increase client-state and compatibility maintenance.
- Each role requires user research and task-specific validation.

### Neutral

- Not every role sees every workbench.
- High visual fidelity does not promote a scientific capability or approve production use.

## Validation

- Unit and contract tests for every form, reducer, chart adapter, table, dialog, receipt, and error state.
- End-to-end simulator scenarios for every listed workbench and command workflow.
- Cross-role/participant route, field, action, and cache leakage tests.
- Visual regression across supported viewports, densities, themes, zoom, and error states.
- Automated and manual accessibility review with keyboard and screen-reader task completion.
- Browser hostile-payload, XSS, CSRF, stale overwrite, duplicate submit, and destructive confirmation tests.
- Reference Pi startup, bundle, memory, FPS, thermal, power, HIL, and multi-day live-stream soak gates.

## Implementation evidence

Implemented in `ui/` with all functional workbenches, role-bounded routes and deep links, shared charts/tables/timelines/drawers/forms/dialogs, synchronized live/history/replay state, workflow progress and receipts, provenance-rich print/export surfaces, typed failure states, destructive previews, and progressive Three.js views. The production gateway supplies real authenticated Rust query/workflow contracts. Automated UI coverage includes all roles and authorized workbenches, leakage, hostile values, stale/conflict/duplicate paths, workflow states, and interaction semantics. External device, manual accessibility, penetration, cross-browser visual, long-soak, and human-product acceptance evidence remains a deployment/release gate rather than a software implementation gap; see `docs/reports/adr-061-ui-validation-matrix.md`.

## Links

**Depends on**: [ADR-040](ADR-040-version-peripheral-drivers-and-design-accessible-interactions.md), [ADR-057](ADR-057-build-a-policy-safe-local-operations-ui-with-vite-tailwind-and-threejs.md), [ADR-058](ADR-058-run-a-production-local-web-gateway-over-rust-application-services.md), [ADR-059](ADR-059-build-a-bounded-live-history-and-replay-query-plane.md), [ADR-060](ADR-060-orchestrate-privileged-operations-as-durable-rust-workflows.md)

**Related**: [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md), [ADR-033](ADR-033-govern-model-registration-promotion-activation-and-rollback.md), [ADR-043](ADR-043-use-guided-installation-site-qualification-and-acceptance.md), [ADR-052](ADR-052-separate-audit-security-and-diagnostic-records.md)
