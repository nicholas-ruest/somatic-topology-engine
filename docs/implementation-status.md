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
- [ ] Phase 08 — Add local observability, SLOs, benchmarks, and fault harnesses
- [ ] Phase 09 — Implement Experiment Validation and dataset governance
- [ ] Phase 10 — Implement and validate respiration-first Physiology Estimation
- [ ] Phase 11 — Build model packaging, registry, uncertainty, and capability policy
- [ ] Phase 12 — Implement State Inference without unsupported claims
- [ ] Phase 13 — Implement Personalization Memory and constrained adaptation
- [ ] Phase 14 — Implement CrowPi Device Interaction and deterministic projections
- [ ] Phase 15 — Implement secure CLI, commissioning, and site qualification
- [ ] Phase 16 — Add optional TypeScript sidecars without weakening the core
- [ ] Phase 17 — Complete reliability, HIL, security, and optimization hardening
- [ ] Phase 18 — Build release, commercial operations, and post-market readiness

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
