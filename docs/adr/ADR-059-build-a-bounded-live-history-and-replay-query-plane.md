# ADR-059: Build a Bounded Live, History, and Replay Query Plane

**Status**: Implemented
**Date**: 2026-07-18
**Deciders**:
**Tags**: streaming, query, replay, telemetry, csi, charts, history

## Context

Real-time charts, CSI diagnostics, signal observations, history scrubbing, filtering, comparison, and replay inspection need more than periodic status cards. Directly streaming raw runtime channels or querying authoritative journals from the browser would couple presentation load to capture, disclose sensitive payloads, and make unbounded history/resource use possible.

Live, persisted-history, and deterministic-replay timelines also have different clocks and truth semantics. Combining them without explicit source and cursor metadata can make replay appear live or stale evidence appear current.

## Decision

Create a Rust query plane behind ADR-058 with three explicit sources:

1. `Live` — bounded projections sampled from runtime observation and health ports;
2. `History` — read-only queries over versioned projections and approved chunk summaries;
3. `Replay` — isolated deterministic replay sessions with virtual event time.

Every response and sample includes source mode, schema version, stream ID, sequence, source/event/emission time, units, algorithm/configuration versions, provenance, authorization scope, retention class, quality/contamination, staleness, and gap/drop indicators.

### Live streams

Provide separate allowlisted streams for capture health, packet continuity, bounded CSI diagnostic summaries, observation windows, approved physiology/state projections, device/peripheral health, runtime/SLO metrics, workflow progress, and security-safe status. Do not create a generic subscribe-to-any-topic endpoint.

Raw CSI, high-rate phase/amplitude, feature arrays, embeddings, model inputs/outputs, and participant-linked samples require a narrow diagnostic contract, explicit role/purpose authorization, bounded duration/resolution, active audit, and shortest practical retention. Ordinary participant views receive approved aggregates only.

Use fixed-capacity ring buffers and per-stream sampling policies. Capture and safety events never block on a browser. Slow consumers receive an explicit gap and must request a fresh snapshot. Critical revocation, policy, fault, and workflow state is delivered through resumable state projections rather than relying on lossy telemetry.

### History and filtering

Expose typed query objects rather than free-form expressions. Bound time range, result count, resolution, fields, sort, filters, comparison series, and concurrent queries by role and data class. Use cursor pagination and server-selected downsampling. Preserve min/max/mean/count and quality/gap metadata so downsampling does not hide faults.

Support drill-down from aggregate to authorized evidence references without returning secret or prohibited payloads. Queries are cancellable and read-only. Expensive query budgets are observable and rate limited.

### Replay inspection

Create isolated replay sessions bound to immutable artifact/config/model digests. Support play, pause, seek, step, speed, range, bookmarks, synchronized multi-track cursors, baseline comparison, and deterministic reset. Replay cannot write production projections, personalization, audit decisions, capability promotions, or production evidence.

The UI clearly differentiates wall clock, event time, processing time, and replay virtual time. Seeking reconstructs state from verified checkpoints and deterministic reducers rather than mutating current live state.

### Chart and visualization contracts

Define typed series schemas for scalar, interval, band, categorical state, event marker, quality mask, topology snapshot, and provenance edge. The Rust query plane chooses units and semantic type; JavaScript chooses presentation only. Unsupported constructs cannot be requested by naming an arbitrary series.

The client uses typed-array ring buffers with fixed limits, discards obsolete rendering data, and pauses hidden routes. Charts and Three.js scenes share the same cursor and filter state but not mutable data buffers.

## Consequences

### Positive

- Enables genuine live charts, history scrubbing, filtering, comparisons, and replay without coupling browsers to capture.
- Preserves timing, provenance, gaps, quality, and replay/live distinctions.
- Gives privacy and resource limits enforceable server-side semantics.

### Negative

- Requires query projections, downsampling, cursor protocols, checkpoints, and replay session management.
- High-rate diagnostic access increases privacy and performance risk.
- Multi-track timeline testing is substantial.

### Neutral

- Visual smoothness may use interpolation, but authoritative values and timestamps remain discrete and inspectable.
- History availability is constrained by ADR-028 retention.

## Validation

- Deterministic live/history/replay parity fixtures and numerical tolerance tests.
- Gap, reorder, duplicate, late event, stale, cursor expiry, checkpoint corruption, and cancellation tests.
- Property tests for range/filter/pagination/downsampling bounds and preservation of extrema/quality masks.
- Cross-role, cross-participant, prohibited-field, retention, deletion, and support-bundle leakage tests.
- Browser long-session heap/GPU bounds, hidden-route suspension, chart frame-time, and Pi thermal/load gates.

## Implementation evidence

Implemented by `ste-query-plane`, the authenticated `/api/v1/query` gateway contract, and the bounded browser timeline adapter. Fixed-capacity live rings report loss explicitly; history is bounded, cursor-based, quality-filtered, and extrema-preserving; replay is deterministic, isolated, seekable, and checkpoint-validated. Rust and browser suites cover bounds, diagnostic denial, gaps, conflict handling, filters, replay controls, and accessible summaries. See `docs/benchmarks/adr-059-query-plane.md`.

## Links

**Depends on**: [ADR-006](ADR-006-use-multi-timescale-event-time-processing.md), [ADR-018](ADR-018-use-an-append-only-journal-with-versioned-projections.md), [ADR-020](ADR-020-build-local-first-observability-and-support-bundles.md), [ADR-028](ADR-028-govern-retention-export-backup-recovery-and-deletion.md), [ADR-030](ADR-030-version-the-dsp-pipeline-and-require-numerical-replay.md), [ADR-058](ADR-058-run-a-production-local-web-gateway-over-rust-application-services.md)

**Related**: [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md), [ADR-031](ADR-031-use-immutable-feature-and-evidence-artifacts.md)
