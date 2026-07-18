# Implementation Status

This checklist records independently gated implementation phases from
[`implementation-prompts.md`](implementation-prompts.md). A phase is checked
only after its required tests and validation pass.

- [x] Phase 01 — Bootstrap the Rust architecture and dependency boundaries
- [x] Phase 02 — Establish threat, safety, supply-chain, and quality baselines
- [ ] Phase 03 — Build configuration, runtime supervision, and deterministic execution
- [ ] Phase 04 — Implement Consent and Governance as the capture gate
- [ ] Phase 05 — Implement encrypted journals, projections, and data lifecycle
- [ ] Phase 06 — Implement Radio Acquisition and replay before live hardware
- [ ] Phase 07 — Implement Signal Observation, DSP, quality, and evidence artifacts
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
