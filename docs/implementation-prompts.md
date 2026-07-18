# Somatic Topology Engine Sequential Implementation Prompts

**Date:** 2026-07-18
**Architecture source:** [ADR index](adr-index.md)
**Domain source:** [DDD model](ddd/README.md)
**Readiness source:** [Production readiness map](production-readiness-map.md)

## How to use this document

Execute these prompts in order. Each prompt is a build phase, not a suggestion to implement every listed ADR in one commit. Within a phase, work in small reviewable increments while preserving the phase exit gate.

For every phase:

1. Read every referenced ADR and DDD document before editing code.
2. Inspect existing work and preserve compatible implementation; do not regenerate completed modules blindly.
3. Keep the critical path Rust-first. Use TypeScript only at an explicitly authorized adapter boundary, and another language only when neither Rust nor TypeScript is adequate.
4. Add traceability from ADR obligations and aggregate invariants to code, tests, benchmarks, and documentation.
5. Run formatting, linting, unit, integration, boundary, security, and relevant replay/HIL tests.
6. Do not mark an ADR `Implemented` merely because scaffolding exists. Link objective evidence first.
7. Stop at the exit gate. If implementation reveals a materially different architectural choice, create or amend an ADR before proceeding.

## Phase 1 — Bootstrap the Rust architecture and dependency boundaries

> Implement the initial STE Cargo workspace and enforce its architectural boundaries. Read ADR-001, ADR-012, ADR-013, ADR-017, ADR-019, ADR-022, ADR-023, and ADR-054. Read the DDD overview, context map, and ubiquitous language.
>
> Create the workspace-level toolchain and manifest, `ste-kernel`, `ste-contracts`, the eight bounded-context crates or explicitly documented initial modules, `ste-runtime`, and `ste-cli`. Keep domain, application, and infrastructure layers distinct. Implement foundational newtypes and traits for identifiers, monotonic/UTC time, units, finite probabilities, provenance, domain events, clocks, and ID generation. Create versioned integration-event DTOs and generated schema/binding hooks without coupling domain code to serialization or external libraries.
>
> Add CI checks for formatting, Clippy, tests, documentation, unsafe-code policy, dependency licenses/advisories, forbidden dependency direction, crate cycles, and direct cross-context imports. Check in the lockfile and pin the Rust toolchain. Create test conventions and ADR/invariant traceability metadata.
>
> Deliver: compiling workspace, public/private boundary tests, dependency-graph validator, initial schemas, contributor build commands, and an architecture test proving contexts cannot import each other's private domain modules.
>
> Exit gate: clean build/test/lint on the supported development host; no dependency cycles or boundary violations; all foundational types reject invalid values; no runtime or context depends on Node.js.

## Phase 2 — Establish threat, safety, supply-chain, and quality baselines

> Establish assurance controls before adding sensitive capture. Read ADR-009, ADR-022, ADR-023, ADR-024, ADR-041, ADR-045, ADR-046, ADR-047, ADR-052, and ADR-054. Read the Consent and Governance DDD and production-readiness map.
>
> Write the initial intended-use and excluded-use statement, asset/data classification, trust-boundary diagram, STRIDE/privacy/abuse threat model, preliminary hazard analysis, claim-evidence matrix, license policy, dependency intake policy, vulnerability-disclosure policy, and verification traceability format. Cover patched Wi-Fi firmware, malformed CSI, physical access, local IPC, storage, model poisoning, covert sensing, and optional TypeScript sidecars.
>
> Configure reproducible dependency scanning, SBOM generation, secret scanning, fuzz targets for future parsers, signed-build placeholders, and quality-record storage. Define which evidence is mandatory before any ADR can move from `Accepted` to `Implemented`.
>
> Deliver: versioned threat model, hazard register, intended-use/claim baseline, SBOM/license reports, assurance ownership, and CI gates that fail on forbidden licenses, exposed secrets, or unreviewed unsafe code.
>
> Exit gate: named owners accept the initial residual risks and prohibited uses; every identified critical hazard has a planned control and verification reference; no unresolved commercial-license blocker exists in the chosen foundation.

## Phase 3 — Build configuration, runtime supervision, and deterministic execution

> Build the local execution substrate. Read ADR-002, ADR-014, ADR-015, ADR-016, ADR-017, ADR-019, ADR-020, ADR-021, ADR-022, ADR-039, and ADR-050. Read the DDD dependency rules in the context map.
>
> Implement strongly typed, versioned, layered configuration with safe defaults, signed-profile support, migration validation, secret isolation, and non-secret configuration digests. Implement Tokio-based composition and supervised tasks with bounded channels, explicit capacities, overflow policies, cancellation, monotonic event time, deterministic test clocks, typed errors, retries, circuit breakers, restart budgets, and coordinated shutdown.
>
> Build runtime health and safe-state primitives without implementing CSI yet. Add queue saturation, task crash, cancellation, clock discontinuity, low-resource, and shutdown tests. Benchmark idle and synthetic-load CPU, memory, queue latency, startup, and shutdown on development and reference ARM targets.
>
> Deliver: `ste-runtime` composition root, config loader/validator, supervisor, health state, synthetic pipeline, deterministic runtime tests, and initial resource-budget report.
>
> Exit gate: bounded memory under overload; deterministic synthetic replay; critical events are never shed; failed optional tasks degrade independently; coordinated shutdown leaves a verifiable safe state.

## Phase 4 — Implement Consent and Governance as the capture gate

> Implement the Consent and Governance bounded context before any live sensing. Read ADR-009, ADR-016, ADR-018, ADR-025, ADR-027, ADR-028, ADR-045, ADR-047, ADR-052, and ADR-053. Read `ddd/contexts/consent-governance.md`, ubiquitous language, and context-map policy relationships.
>
> Implement `SensingAuthorization` with its entities, value objects, commands, events, repository port, and `PolicyDecisionPoint`. Enforce space, participant, purpose, version, expiration, revocation, prohibited use, retention class, and fail-closed behavior. Implement test clocks and deterministic policy evaluation. Model revocation propagation and deletion requests without requiring network or agent availability.
>
> Add authorization checks to the runtime's not-yet-connected capture port and privileged command boundary. Separate participant-visible history, domain audit, security events, and diagnostics. Add property tests for all aggregate invariants and attempts by infrastructure/sidecars to widen purpose.
>
> Deliver: governance crate/application service, in-memory and durable repository contract tests, policy API, audit events, consent fixtures, and safe unauthorized UI/runtime state.
>
> Exit gate: no capture-source implementation can publish a frame without active permission; revocation blocks publication immediately; prohibited purposes cannot be enabled by config, feature policy, administrator, or adapter.

## Phase 5 — Implement encrypted journals, projections, and data lifecycle

> Build durable state and recovery. Read ADR-007, ADR-018, ADR-019, ADR-025, ADR-027, ADR-028, ADR-049, and ADR-052. Read repository-port requirements in all DDD contexts, especially Consent and Governance and Personalization Memory.
>
> Implement append-only, checksummed, data-class-partitioned journals; atomic projection checkpoints; idempotent handlers; schema versions/upcasters; corruption detection; bounded chunk storage; compaction; and recovery to the last verified checkpoint. Implement unique device identity abstraction, envelope encryption, key rotation/zeroization interfaces, and protected development-key fallback clearly separated from production assurance.
>
> Implement lifecycle policy for retention, export, backup eligibility, restore, deletion propagation, cryptographic erasure, factory reset, and decommissioning. Test torn writes, disk-full, corrupt checksums, interrupted migration, restore, deletion across projections/caches/vectors, and reset to capture-disabled state.
>
> Deliver: durable repository adapters, journal inspection/rebuild tools, encrypted portable export manifest, recovery/reset commands behind authorization, and migration/corruption test corpus.
>
> Exit gate: deterministic rebuild from valid journals; explicit failure on unrecoverable corruption; restore and deletion tests cover every current data class; secrets and sensitive payloads never appear in diagnostics.

## Phase 6 — Implement Radio Acquisition and replay before live hardware

> Implement the Radio Acquisition context using file/replay first, then live Nexmon/rvCSI. Read ADR-004, ADR-005, ADR-014, ADR-017, ADR-022, ADR-029, ADR-038, ADR-043, and ADR-044. Read `ddd/contexts/radio-acquisition.md` and the Radio Acquisition relationships in the context map.
>
> Implement `CaptureSession`, `CaptureProfile`, `CaptureLink`, `FrameSequence`, calibration metadata, commands, events, repository, `FrameJournal`, `CsiCaptureSource`, and versioned `ValidatedCsiFrameV1`/`CaptureHealthV1`. Reject malformed/non-finite/implausible frames before publication; preserve gaps, quality, hardware/firmware/channel/link provenance, and bounded backpressure behavior.
>
> Add deterministic `.rvcsi`/PCAP replay fixtures and rvCSI anti-corruption adapter tests. Only after replay passes, add a pinned Raspberry Pi 4/Nexmon adapter and compatibility manifest for chipset, OS, kernel, firmware, AP, band, channel, bandwidth, packet source, and geometry. Capture remains policy-gated.
>
> Deliver: replay CLI, capture health, file and live adapters, known-good image instructions, capture fixtures, calibration-profile draft, and acquisition benchmarks.
>
> Exit gate: identical replay yields identical validated frame sequence; malformed fuzz corpus cannot panic; live reference hardware records and replays a qualified session with explicit accepted/degraded/rejected/missing statistics.

## Phase 7 — Implement Signal Observation, DSP, quality, and evidence artifacts

> Build observations without physiological or mental labels. Read ADR-003, ADR-005, ADR-006, ADR-017, ADR-021, ADR-022, ADR-029, ADR-030, ADR-031, ADR-034, ADR-044, and ADR-050. Read `ddd/contexts/signal-observation.md` and the ubiquitous definitions of observation, quality gate, evidence horizon, and abstention.
>
> Implement `ObservationWindow`, window policies, frame evidence, DSP graph specification, quality assessment, motion energy, presence score, periodicity candidates, baseline drift, interference/missingness handling, contamination, commands/events, repository, and `ObservationWindowClosedV1`. Wrap pinned rvCSI/RuView DSP primitives behind Rust ports; do not expose upstream types.
>
> Create immutable content-addressed feature/evidence artifacts with units, algorithm/DSP version, calibration, window bounds, source references, quality, missingness, and partition role. Build golden tests for constant, impulse, sinusoid, noise, gaps, saturation, phase wrap, motion, and recorded CSI; add cross-architecture tolerances and performance profiles.
>
> Deliver: deterministic DSP pipeline, golden corpus, observation replay reports, feature schema, quality/abstention reasons, drift events, and observation benchmarks.
>
> Exit gate: replay meets declared numerical tolerance; contaminated windows never become clean estimates; feature artifacts retain complete provenance; no observation type contains physiology, emotion, workload, or decision semantics.

## Phase 8 — Add local observability, SLOs, benchmarks, and fault harnesses

> Instrument the working acquisition-to-observation pipeline. Read ADR-020, ADR-021, ADR-022, ADR-038, ADR-039, ADR-050, and ADR-052. Read the production-readiness map's observability, performance, quality, and hardware rows.
>
> Implement structured local metrics, traces, diagnostics, health snapshots, audit separation, rotating storage, redaction, cardinality bounds, dropped-record counters, and user-previewed support bundles. Define measurable SLOs for capture continuity, valid-window coverage, queue delay, projection freshness, startup/recovery, CPU, RSS, storage growth, temperature, and power.
>
> Build benchmark and fault harnesses for packet loss, malformed frames, AP loss, queue overload, disk-full, corruption, task death, time jumps, low voltage, thermal pressure, and power interruption. Establish development replay tests and reference-Pi benchmark baselines.
>
> Deliver: local diagnostics command, benchmark suite, regression comparison, redaction tests/canaries, support-bundle manifest, and fault-injection framework.
>
> Exit gate: no sensitive fields appear in default logs/bundles; budgets have owners and pass/fail thresholds; overload/fault behavior matches ADR-015; baseline performance is recorded before optimization.

## Phase 9 — Implement Experiment Validation and dataset governance

> Build the validation system before enabling physiology models. Read ADR-008, ADR-022, ADR-028, ADR-031, ADR-034, ADR-036, ADR-037, ADR-044, ADR-045, and ADR-047. Read `ddd/contexts/experiment-validation.md` and the research validation program.
>
> Implement `ValidationStudy`, frozen protocols, cohorts, participant/session/room-aware split manifests, metrics, baselines, gates, study runs, immutable results, repository, de-identified evidence export, and promotion registry. Implement dataset manifests/cards with consent, purpose, license, acquisition profile, transformations, digests, retention, missingness, and partition roles.
>
> Build reference-sensor adapters and synchronization artifacts for respiratory belt and ECG/validated PPG plus task/self-report timestamps. Track alignment uncertainty and reference quality. Implement agreement, calibration, selective-risk/coverage, confidence interval, failure-rate, and baseline comparisons. Prohibit session/window leakage in code.
>
> Deliver: preregistration templates, dataset/split tooling, reference sync test rig, metrics library, promotion registry, negative-result preservation, and reproducible validation report generation.
>
> Exit gate: a synthetic/pilot protocol can be frozen, executed, reproduced, and rejected/promoted without editing results; leakage tests fail deliberately contaminated splits; human collection remains blocked without recorded ethics/consent authority.

## Phase 10 — Implement and validate respiration-first Physiology Estimation

> Implement physiology in promoted stages, beginning with respiration only. Read ADR-003, ADR-005, ADR-006, ADR-008, ADR-021, ADR-029, ADR-030, ADR-031, ADR-032, ADR-033, ADR-034, ADR-037, ADR-044, and ADR-050. Read `ddd/contexts/physiology-estimation.md` and Experiment Validation.
>
> Implement `PhysiologyAssessment`, modality definitions, evidence horizons, stillness requirements, error bounds, validation status, confidence, abstention, commands/events, repository, `ValidationRegistry` query, and `PhysiologyEvidenceUpdatedV1`. Create a Rust `PhysiologyModel` port and deterministic estimator baseline before considering ruv-FANN.
>
> Train/evaluate respiration candidates only through frozen protocols and session/day-held-out splits. Compare against reference belt data using predefined agreement and coverage metrics. Enforce motion, quality, OOD, calibration, evidence-duration, and operating-envelope abstention. Do not implement cardiac coherence, HRV, stress, valence, workload, or decision labels in this phase.
>
> Deliver: respiration estimator/model package, model card, known-answer/parity tests, reference agreement report, calibrated confidence/abstention policy, and reference-Pi benchmarks.
>
> Exit gate: respiration passes its preregistered gate and resource budget or remains disabled with preserved negative evidence; UI/API cannot represent it as medical-grade; failed gates cannot be overridden by config.

## Phase 11 — Build model packaging, registry, uncertainty, and capability policy

> Build the generic edge-model lifecycle before latent-state inference. Read ADR-008, ADR-016, ADR-021, ADR-023, ADR-026, ADR-031, ADR-032, ADR-033, ADR-034, ADR-041, ADR-044, ADR-050, and ADR-053. Read State Inference and Experiment Validation DDDs.
>
> Implement the Rust `InferenceModel` port, signed `ModelPackage`, compatibility and license metadata, feature/preprocessing digests, operating scope, calibration artifact, resource profile, model card, known-answer tests, and ruv-FANN adapter where verified. Implement registry states from quarantined through revoked, atomic activation, health checks, suspension, rollback, and immutable promotion decisions.
>
> Implement calibrated probabilities, OOD/scope checks, selective-risk/coverage evaluation, and signed capability policy bound to software/model/hardware, participant purpose, validation promotion, jurisdiction, operating envelope, and expiration. Experimental capability output must remain visibly non-production and isolated.
>
> Deliver: package builder/verifier, local registry, activation/rollback tests, capability-policy evaluator, calibration/OOD reports, and corrupt/incompatible/adversarial model test fixtures.
>
> Exit gate: an unsigned, incompatible, unpromoted, revoked, OOD, or policy-disabled model cannot serve; activation is atomic and rollback restores the previous known-answer behavior.

## Phase 12 — Implement State Inference without unsupported claims

> Implement latent-state mechanics while keeping unvalidated constructs disabled. Read ADR-003, ADR-005, ADR-006, ADR-008, ADR-010, ADR-032, ADR-033, ADR-034, ADR-044, ADR-045, ADR-047, and ADR-053. Read `ddd/contexts/state-inference.md`, ubiquitous language, and Physiology Estimation.
>
> Implement `StateAssessment`, operational construct definitions, claim levels, evidence bundles, model scope, calibrated probability, provenance, temporal debounce policy, abstention, commands/events, repository, and `DisplayProjectionV1` mapping boundary. Require promoted construct plus valid physiology, profile, model, calibration, policy, and operating envelope.
>
> Build a no-op/baseline construct path and full invariant tests before any trained cognitive model. If a task-specific workload study is authorized, implement it through Experiment Validation with strong time/task/motion baselines and held-out days. Keep valence and decision phase disabled until separate protocols pass. Never convert generic arousal or periodicity directly into those labels.
>
> Deliver: state domain/application implementation, temporal replay tests, claim-level policy, model-scope/OOD abstention tests, and validation hooks.
>
> Exit gate: only specifically promoted constructs can emit; raw model output never reaches UI; every assessment traces through physiology and observation evidence; disabled constructs are unrepresentable as production projections.

## Phase 13 — Implement Personalization Memory and constrained adaptation

> Add personalized anchors and retrieval after model governance exists. Read ADR-007, ADR-018, ADR-025, ADR-028, ADR-031, ADR-033, ADR-034, ADR-035, ADR-036, and ADR-052. Read `ddd/contexts/personalization-memory.md` and its interaction with State Inference and Device Interaction.
>
> Implement `PatternProfile`, participant-scoped anchors, embeddings, feedback, adaptation lineage, partition roles, commands/events, repository, and `VectorMemory` port. Use the append-only journal plus Rust RuVector/RVF adapter as authority. Ensure corrections append rather than rewrite and evaluation partitions are read-only.
>
> Implement similarity retrieval before parameter adaptation. Then add sandboxed candidate adaptation only with explicit quality-qualified feedback, minimum evidence, rate limits, exact training set lineage, prospective comparison, promotion, and rollback. Implement participant view/correct/delete flows and cryptographic erasure across vector indexes.
>
> Deliver: RuVector adapter benchmarks, deterministic retrieval tests, anchor provenance UI/API, poisoning/leakage tests, adaptation candidate workflow, and deletion/rebuild verification.
>
> Exit gate: cross-user retrieval is impossible by default; test/evaluation data cannot update memory; “improved” is shown only after prospective held-out evidence; deleting a participant removes retrievable payloads and rebuilds indexes correctly.

## Phase 14 — Implement CrowPi Device Interaction and deterministic projections

> Implement interaction using simulator-first hardware ports. Read ADR-005, ADR-010, ADR-011, ADR-017, ADR-039, ADR-040, ADR-044, and ADR-053. Read `ddd/contexts/device-interaction.md` and the context-map contracts from State Inference and to Personalization Memory.
>
> Implement `InteractionSession`, display/LED/touch/peripheral-health entities, approved `Projection` enum, refresh/staleness policies, commands/events, repository, and audit journal. Build simulator adapters first, then versioned physical OLED, RGB, touch, DHT11, visible sensing indicator, and physical-off adapters for the verified CrowPi revision.
>
> Render text plus color; support color-vision accessibility, brightness, stable transitions, explicit unauthorized/calibrating/contaminated/insufficient/stale/fault states, and anchor confirmation. Initially RGB represents signal quality or explicitly user-labeled arousal, never inferred valence. DHT data remains a timestamped covariate and cannot alter confidence without validation.
>
> Deliver: simulator UI, physical-driver profiles, projection snapshot tests, touch debounce/anchor flow, peripheral fault tests, accessibility review, and HIL fixtures.
>
> Exit gate: simulator and hardware satisfy identical contracts; peripheral failure cannot freeze a healthy display or stop core supervision; every rendered label is policy-approved and evidence-age aware.

## Phase 15 — Implement secure CLI, commissioning, and site qualification

> Implement the supported operator surface and installation workflow. Read ADR-016, ADR-025, ADR-026, ADR-027, ADR-028, ADR-029, ADR-039, ADR-040, ADR-043, ADR-044, ADR-049, and ADR-051. Read Radio Acquisition, Consent and Governance, and Device Interaction DDDs.
>
> Implement authenticated Unix-domain IPC and `ste-cli` commands for status, doctor, authorization, capture test, hardware probe, calibration, replay, validation export, models, capability policy, support bundle, updates, data lifecycle, recovery, and reset. Use stable JSON plus human output, typed exits, idempotency keys, authorization, redacted input, and confirmation/dry-run for destructive operations.
>
> Build guided commissioning that verifies hardware/firmware, peripherals, power/thermal, AP/link, packet rate, geometry, consent, storage, clocks, calibration, interference, and capability-specific coverage. Produce signed site-acceptance and requalification records; block unsupported capabilities rather than weakening thresholds.
>
> Deliver: stable CLI/API schemas, authorization tests, commissioning workflow, recovery mode, acceptance report, and installation/support runbooks.
>
> Exit gate: a fresh reference device can be securely provisioned, qualified, operated, diagnosed, requalified, and reset offline; an unqualified site cannot enable out-of-envelope capabilities.

## Phase 16 — Add optional TypeScript sidecars without weakening the core

> Add TypeScript integrations only for a measured missing capability. Read ADR-001, ADR-007, ADR-012, ADR-014, ADR-019, ADR-023, ADR-024, ADR-027, and ADR-042. Read the DDD modeling rules and context-map dependency rule for TypeScript adapters.
>
> First document the capability gap and demonstrate why Rust is not applicable. Implement generated contract bindings and an authenticated, allowlisted local IPC adapter. Run the sidecar unprivileged, offline by default, with CPU/memory/storage limits, health checks, rate limits, sandboxing, pinned Node/npm versions, and no hardware or authoritative-store access.
>
> If integrating AgentDB, keep RuVector/domain journals authoritative. If integrating DSPy.ts, restrict it to offline copy optimization whose reviewed output becomes Rust templates. If integrating Ruflo, restrict it to development/experiment orchestration rather than live inference. Ensure the Rust runtime remains fully functional with the sidecar absent, hung, corrupt, or killed.
>
> Deliver: written justification, adapter threat-model update, generated bindings, contract/failure/resource tests, SBOM/license update, and removal/fallback plan.
>
> Exit gate: sidecar failure cannot affect capture, authorization, inference validity, persistence authority, or safe UI; it cannot widen purpose, claim level, or capabilities.

## Phase 17 — Complete reliability, HIL, security, and optimization hardening

> Harden the integrated system on reference hardware. Read ADR-015, ADR-020, ADR-021, ADR-022, ADR-023, ADR-024, ADR-025, ADR-026, ADR-038, ADR-039, ADR-041, ADR-045, ADR-046, ADR-049, ADR-050, and ADR-052. Read the full DDD validation report and production-readiness map.
>
> Complete parser/model/IPC fuzzing; penetration testing; journal corruption/migration tests; key compromise/rotation tests; update A/B activation and downgrade controls; rollback; backup/restore/reset; low-voltage, thermal, bus, storage, AP, and power fault injection; watchdog validation; and multi-day soak tests. Exercise incident response and preserve signed evidence.
>
> Profile the complete pipeline on the reference Pi. Optimize measured bottlenecks only, then rerun numerical replay, calibration, selective risk, SLO, thermal, power, HIL, and operating-envelope gates. Establish release-blocking regression thresholds and supported media/endurance limits.
>
> Deliver: HIL/soak evidence, penetration and fuzz reports, incident exercise, update/recovery evidence, final benchmark distributions, resource/thermal profiles, hazard-control traceability, and residual-risk review.
>
> Exit gate: all release SLOs and safety/security gates pass under nominal and defined fault loads; no optimization regresses scientific fidelity, privacy, abstention, or recoverability.

## Phase 18 — Build release, commercial operations, and post-market readiness

> Prepare the exact production candidate and organization around it. Read ADR-026, ADR-028, ADR-033, ADR-041, ADR-043, ADR-044, ADR-045, ADR-046, ADR-047, ADR-048, ADR-049, ADR-052, ADR-053, ADR-054, ADR-055, and ADR-056. Read the complete ADR index, production-readiness map, research conclusions, and all DDD documents.
>
> Build hermetic reproducible ARM releases with signed manifests, software/firmware/model/data SBOMs, model/dataset cards, compatibility matrix, migrations, update/rollback, support matrices, versioned user/installer/operator/security/privacy/API documentation, and immutable verification evidence. Promote identical artifacts through candidate, pilot, and production channels.
>
> Finalize intended use and claims, jurisdictional review, quality records, warranty, repair/replace, support/update windows, installation qualification, incident and recall processes, end-of-life, complaint handling, privacy-safe optional telemetry, and post-market risk review. Run a commercial pilot that measures installation success, coverage/abstention, reliability, support cost, returns, usability, and willingness to pay without changing scientific gates post hoc.
>
> Convene ADR-055 production-readiness review for the exact hardware, software, model, market, operating envelope, and support organization. Link every criterion to immutable evidence and record residual risks/exceptions. Create corrective-action and capability-suspension procedures for post-market findings.
>
> Deliver: signed release evidence bundle, pilot report, claim-evidence matrix, regulatory/legal approvals, quality and support records, readiness decision, incident contacts, and post-market plan.
>
> Exit gate: production approval is explicit, scoped, evidence-backed, and time-bound; all enabled claims are promoted and supported; unacceptable hazards or unsupported constructs remain disabled; the organization can update, support, recover, suspend, notify, and decommission deployed devices.

## Sequence completion rule

Finishing Phase 18 does not freeze the architecture. Any implementation discovery that materially changes the defined hardware, intended use, cloud boundary, regulated status, multi-person scope, model class, data flow, or manufacturing/support model requires a new sequential ADR and corresponding updates to the DDD model, this implementation sequence, and the production-readiness evidence map.
