# Implementation Status

This checklist records independently gated implementation phases from
[`implementation-prompts.md`](implementation-prompts.md). A phase is checked
only after its required tests and validation pass.

- [x] Phase 01 — Bootstrap the Rust architecture and dependency boundaries
- [x] Phase 02 — Establish threat, safety, supply-chain, and quality baselines
- [x] Phase 03 — Build configuration, runtime supervision, and deterministic execution
- [x] Phase 04 — Implement Consent and Governance as the capture gate
- [x] Phase 05 — Implement encrypted journals, projections, and data lifecycle
- [x] Phase 06 — Implement Radio Acquisition and replay before live hardware
- [x] Phase 07 — Implement Signal Observation, DSP, quality, and evidence artifacts
- [x] Phase 08 — Add local observability, SLOs, benchmarks, and fault harnesses
- [x] Phase 09 — Implement Experiment Validation and dataset governance
- [x] Phase 10 — Implement and validate respiration-first Physiology Estimation
- [x] Phase 11 — Build model packaging, registry, uncertainty, and capability policy
- [x] Phase 12 — Implement State Inference without unsupported claims
- [x] Phase 13 — Implement Personalization Memory and constrained adaptation
- [x] Phase 14 — Implement CrowPi Device Interaction and deterministic projections
- [x] Phase 15 — Implement secure CLI, commissioning, and site qualification
- [x] Phase 16 — Add optional TypeScript sidecars without weakening the core
- [x] Phase 17 — Complete reliability, HIL, security, and optimization hardening
- [x] Phase 18 — Build release, commercial operations, and post-market readiness controls

## Phase 01 evidence

- Workspace compilation, formatting, Clippy, tests, and rustdoc pass with warnings denied.
- Domain boundary, unsafe-code, dependency advisory, license, and source-policy gates pass.
- Versioned contracts and JSON Schema generation are covered by automated tests.

## Phase 02 evidence

- Threat, privacy, abuse, trust-boundary, data-classification, and hazard baselines are versioned with owners and verification references.
- Intended-use, prohibited-use, claim-evidence, risk-acceptance, vulnerability-disclosure, and quality-record controls are defined.
- CI enforces dependency, license, advisory, secret, unsafe-code, SBOM, reproducibility, fuzz-build, and provenance gates.

## Phase 03 evidence

- Layered, signed, versioned configuration fails closed, isolates secrets, and produces deterministic non-secret digests.
- Bounded queues, restart budgets, circuit breaking, cancellation, health degradation, and safe shutdown are covered by deterministic tests.
- Synthetic replay is deterministic, critical events are never shed, and development-host runtime budgets pass; ARM qualification remains explicitly pending reference hardware.

## Phase 04 evidence

- The authorization aggregate enforces exact participant, space, purpose, policy-version, expiry, retention, revocation, and prohibited-use invariants.
- Fail-closed policy evaluation, in-memory and atomic durable repository contracts, offline deletion propagation, and separated governance records are tested.
- Runtime capture publication and privileged commands reevaluate authorization and block immediately on revocation or attempted purpose widening.

## Phase 05 evidence

- Checksummed partitioned journals, bounded chunks, upcasters, idempotent projections, atomic checkpoints, compaction, and recovery pass fault-injection tests.
- Authenticated per-class envelope encryption, managed key rotation/erasure, portable export, restore, deletion propagation, reset, and decommissioning fail closed.
- Authorized storage CLI operations, destructive confirmation, corruption corpora, and recovery fault matrices are executable; unauthenticated direct invocation is denied.

## Phase 06 evidence

- Radio domain and contract tests enforce plausible finite frames, exact provenance/calibration, sequence gaps, health statistics, policy gating, and bounded backpressure.
- Bounded deterministic RVCSI/PCAP replay, hostile malformed inputs, anti-corruption conversion, and parser fuzz builds pass without panics.
- Replay CLI and acquisition budgets pass on the development host; the pinned live adapter and compatibility procedure are ready, while physical Pi 4 qualification remains explicitly pending hardware.

## Phase 07 evidence

- Observation-only domain types enforce monotonic contamination, quality/abstention, exact provenance, bounded windows, and immutable content-addressed artifacts.
- Versioned finite DSP passes golden, tolerance, phase-wrap, hostile-graph, and replay tests without downstream claim vocabulary.
- Governed observation replay preserves source references and development numerical/performance budgets; physical ARM profiling remains pending hardware.

## Phase 08 evidence

- Local bounded metrics, traces, separated records, health snapshots, redaction, drop counters, and preview-bound checksummed support bundles pass canary tests.
- Eleven typed, owned SLO budgets and regression comparisons cover continuity, coverage, latency, freshness, recovery, compute, storage, temperature, and power.
- Cross-layer synthetic fault and benchmark suites pass on the development host; Pi thermal, power, and soak evidence remains an explicit release blocker.

## Phase 09 evidence

- Frozen study protocols, complete dataset cards, four-way split manifests, and property tests reject participant/session/room/day leakage and block unauthorized human collection.
- Agreement, calibration, selective-risk, interval, failure-rate, baseline, and reference synchronization artifacts are deterministic and finite checked.
- Atomic repositories, append-only promotion decisions, de-identified export, negative-result preservation, governed CLI tooling, and byte-reproducible reports pass validation.

## Phase 10 evidence

- The physiology domain exposes respiration only and enforces validation promotion plus motion, quality, OOD, calibration, duration, confidence, and operating-envelope abstention.
- A deterministic content-digested baseline passes exact known-answer/parity and held-out evaluation mechanics without granting promotion.
- Repository, registry, governed CLI, model card, negative report, and host benchmark pass; real belt-reference and Pi gates remain pending, so respiration stays disabled and non-medical.

## Phase 11 evidence

- Ed25519-signed, content-addressed packages bind weights, feature/preprocessing/calibration digests, model card, lineage, licensing, and exact software/hardware compatibility; adversarial fixtures fail closed.
- The registry preserves immutable quarantine-to-revocation decisions, permits serving only from an active promoted package, and atomically activates, suspends, and rolls back with known-answer verification.
- Frozen calibration, OOD/selective-risk evaluation, signed capability policy, governed CLI operations, and experimental isolation pass workspace gates; Pi HIL, thermal, and soak qualification remain pending reference hardware.

## Phase 12 evidence

- State assessments support only a no-claim baseline and specifically promoted task-workload constructs; valence and decision-phase production variants do not exist.
- Sixteen domain, temporal replay, validation, repository, and projection tests enforce evidence lineage, debounce/staleness, OOD abstention, capability policy, and experimental isolation.
- The sole UI projection exposes approved bands or typed unavailability without raw scores or evidence payloads; no trained cognitive model is claimed and Pi/human-validity gates remain pending.

## Phase 13 evidence

- Participant-scoped types, encrypted append-only vector records, and deterministic retrieval prevent cross-participant queries and evaluation-partition updates by construction and tests.
- Adaptation candidates require quality-qualified evidence, exact lineage, rate limits, sandboxing, and prospective held-out improvement before promotion, with rollback preserved.
- Governed view/correct/delete flows destroy participant key material and rebuild derived indexes; the Rust reference adapter is qualified, while RuVector/RVF and Pi benchmarks remain explicitly pending.

## Phase 14 evidence

- Simulator and versioned CrowPi profiles implement the same safe display, RGB, touch, DHT covariate, visible-sensing, and physical-off ports with deterministic snapshots and fault injection.
- Approved accessible projections include explicit authorization, calibration, contamination, insufficiency, staleness, and fault states; RGB cannot encode inferred valence.
- Touch debounce, anchor authorization, append-only audit, and peripheral isolation pass workspace gates; exact board-revision HIL, accessibility review, and Pi qualification remain pending hardware and human review.

## Phase 15 evidence

- Authenticated UID-bound Unix IPC enforces stable bounded JSON, role authorization, nonce replay protection, idempotent retries, secret redaction, and typed failure exits.
- Guided commissioning requires all mandatory checks, signs exact enabled/blocked capability sets, preserves requalification lineage, and cannot qualify from recovery mode or incomplete evidence.
- The operator CLI covers the supported lifecycle with confirmation/dry-run for reset and passes hostile-input gates; physical reference-device site acceptance remains pending and cannot be enabled by synthetic evidence.

## Phase 16 evidence

- No measured Rust capability gap exists, so no production TypeScript, Node, AgentDB, DSPy, or Ruflo dependency was added.
- Signed sidecar manifests and the Rust supervisor bind executable/contract digests, advisory-only allowlists, purpose/claim/capability ceilings, offline sandboxing, resource limits, authentication, and deadlines.
- Absent, hung, corrupt, killed, or disabled sidecars leave authorization, capture, inference validity, persistence authority, hardware, and the Rust core unaffected; removal and fallback are documented.

## Phase 17 evidence

- Signed A/B updates, downgrade controls, health rollback, authenticated backup/restore, journal migration/corruption detection, key rotation/compromise, and zeroizing reset pass adversarial tests.
- Deterministic hostile parser/model/IPC corpora, authorization/redaction probes, watchdog checks, and simulated voltage/thermal/bus/storage/AP/power faults produce signed development-host evidence.
- Release regression gates pass on the current host with no speculative optimization; external penetration, continuous fuzzing, exact-Pi HIL, thermal/power, and multi-day soak evidence remain explicit production blockers.

## Phase 18 evidence

- Canonical signed release manifests require exact source, lockfile, toolchain, software/firmware, SBOM, model/data, compatibility, migration, update, rollback, and support evidence; channel promotion forbids rebuilding artifacts.
- The signed readiness engine binds the exact release scope, requires twelve current evidence criteria, records residual risks and corrective actions, and cannot bypass legal, claim, pilot, HIL, or safety gates.
- Commercial operations, release, pilot, claims, incident/recall, support, warranty, complaint, EOL, privacy, post-market, CAPA, and suspension procedures are documented; the current verified decision is `NOT_APPROVED`, so sale and production deployment remain blocked.
