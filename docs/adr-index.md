# Architecture Decision Record Index

The ADR series turns the [research synthesis](research.md) and [DDD model](ddd/README.md) into implementation and production obligations. All decisions are initially **Proposed** and require named deciders before acceptance.

## Foundation and domain boundaries

| ADR | Decision |
|---|---|
| [ADR-001](adr/ADR-001-adopt-a-rust-first-modular-monolith.md) | Adopt a Rust-First Modular Monolith |
| [ADR-002](adr/ADR-002-keep-the-edge-runtime-local-and-deterministic.md) | Keep the Edge Runtime Local and Deterministic |
| [ADR-003](adr/ADR-003-separate-observations-physiology-and-latent-state.md) | Separate Observations, Physiology, and Latent State |
| [ADR-004](adr/ADR-004-use-rvcsi-and-nexmon-behind-a-versioned-capture-port.md) | Use rvCSI and Nexmon Behind a Versioned Capture Port |
| [ADR-005](adr/ADR-005-make-quality-gating-and-abstention-mandatory.md) | Make Quality Gating and Abstention Mandatory |
| [ADR-006](adr/ADR-006-use-multi-timescale-event-time-processing.md) | Use Multi-Timescale Event-Time Processing |
| [ADR-007](adr/ADR-007-use-ruvector-as-primary-personalization-memory.md) | Use RuVector as Primary Personalization Memory |
| [ADR-008](adr/ADR-008-promote-capabilities-through-preregistered-validation-gates.md) | Promote Capabilities Through Preregistered Validation Gates |
| [ADR-009](adr/ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md) | Enforce Consent, Privacy, and Prohibited Use in the Domain |
| [ADR-010](adr/ADR-010-use-deterministic-policy-approved-ui-projections.md) | Use Deterministic, Policy-Approved UI Projections |
| [ADR-011](adr/ADR-011-abstract-crowpi-hardware-behind-rust-ports.md) | Abstract CrowPi Hardware Behind Rust Ports |
| [ADR-012](adr/ADR-012-use-versioned-contracts-and-an-anti-corruption-layer.md) | Use Versioned Contracts and an Anti-Corruption Layer |

## Runtime, persistence, quality, and engineering controls

| ADR | Decision |
|---|---|
| [ADR-013](adr/ADR-013-define-the-cargo-workspace-and-crate-dependency-policy.md) | Define the Cargo Workspace and Crate Dependency Policy |
| [ADR-014](adr/ADR-014-use-a-bounded-asynchronous-runtime-with-backpressure.md) | Use a Bounded Asynchronous Runtime with Backpressure |
| [ADR-015](adr/ADR-015-supervise-failures-and-degrade-capabilities-independently.md) | Supervise Failures and Degrade Capabilities Independently |
| [ADR-016](adr/ADR-016-use-validated-layered-configuration-and-secret-isolation.md) | Use Validated Layered Configuration and Secret Isolation |
| [ADR-017](adr/ADR-017-standardize-identifiers-time-units-and-numeric-semantics.md) | Standardize Identifiers, Time, Units, and Numeric Semantics |
| [ADR-018](adr/ADR-018-use-an-append-only-journal-with-versioned-projections.md) | Use an Append-Only Journal with Versioned Projections |
| [ADR-019](adr/ADR-019-evolve-contracts-with-explicit-compatibility-rules.md) | Evolve Contracts with Explicit Compatibility Rules |
| [ADR-020](adr/ADR-020-build-local-first-observability-and-support-bundles.md) | Build Local-First Observability and Support Bundles |
| [ADR-021](adr/ADR-021-enforce-slos-and-resource-budgets-with-benchmarks.md) | Enforce SLOs and Resource Budgets with Benchmarks |
| [ADR-022](adr/ADR-022-adopt-a-replay-first-multi-layer-test-strategy.md) | Adopt a Replay-First Multi-Layer Test Strategy |
| [ADR-023](adr/ADR-023-govern-dependencies-licenses-and-software-supply-chain.md) | Govern Dependencies, Licenses, and the Software Supply Chain |
| [ADR-024](adr/ADR-024-maintain-a-living-threat-model-and-trust-boundaries.md) | Maintain a Living Threat Model and Trust Boundaries |

## Security, RF/DSP, data, and model lifecycle

| ADR | Decision |
|---|---|
| [ADR-025](adr/ADR-025-use-device-identity-encryption-and-managed-keys.md) | Use Device Identity, Encryption, and Managed Keys |
| [ADR-026](adr/ADR-026-use-signed-reproducible-updates-with-atomic-rollback.md) | Use Signed Reproducible Updates with Atomic Rollback |
| [ADR-027](adr/ADR-027-authenticate-and-authorize-local-administration.md) | Authenticate and Authorize Local Administration |
| [ADR-028](adr/ADR-028-govern-retention-export-backup-recovery-and-deletion.md) | Govern Retention, Export, Backup, Recovery, and Deletion |
| [ADR-029](adr/ADR-029-version-csi-calibration-baselines-and-operating-geometry.md) | Version CSI Calibration, Baselines, and Operating Geometry |
| [ADR-030](adr/ADR-030-version-the-dsp-pipeline-and-require-numerical-replay.md) | Version the DSP Pipeline and Require Numerical Replay |
| [ADR-031](adr/ADR-031-use-immutable-feature-and-evidence-artifacts.md) | Use Immutable Feature and Evidence Artifacts |
| [ADR-032](adr/ADR-032-standardize-edge-model-packages-and-the-rust-inference-port.md) | Standardize Edge Model Packages and the Rust Inference Port |
| [ADR-033](adr/ADR-033-govern-model-registration-promotion-activation-and-rollback.md) | Govern Model Registration, Promotion, Activation, and Rollback |
| [ADR-034](adr/ADR-034-calibrate-uncertainty-detect-out-of-distribution-inputs-and-abstain.md) | Calibrate Uncertainty, Detect OOD Inputs, and Abstain |
| [ADR-035](adr/ADR-035-constrain-personalization-and-online-adaptation.md) | Constrain Personalization and Online Adaptation |
| [ADR-036](adr/ADR-036-govern-datasets-partitions-lineage-and-reproducibility.md) | Govern Datasets, Partitions, Lineage, and Reproducibility |

## Validation, hardware, delivery, safety, and commercial operation

| ADR | Decision |
|---|---|
| [ADR-037](adr/ADR-037-synchronize-reference-sensors-and-quantify-measurement-error.md) | Synchronize Reference Sensors and Quantify Measurement Error |
| [ADR-038](adr/ADR-038-require-hardware-in-loop-fault-injection-and-soak-testing.md) | Require HIL, Fault Injection, and Soak Testing |
| [ADR-039](adr/ADR-039-handle-power-thermal-storage-and-watchdog-failures.md) | Handle Power, Thermal, Storage, and Watchdog Failures |
| [ADR-040](adr/ADR-040-version-peripheral-drivers-and-design-accessible-interactions.md) | Version Peripheral Drivers and Design Accessible Interactions |
| [ADR-041](adr/ADR-041-use-reproducible-ci-cd-and-evidence-bearing-releases.md) | Use Reproducible CI/CD and Evidence-Bearing Releases |
| [ADR-042](adr/ADR-042-isolate-and-supervise-typescript-sidecars.md) | Isolate and Supervise TypeScript Sidecars |
| [ADR-043](adr/ADR-043-use-guided-installation-site-qualification-and-acceptance.md) | Use Guided Installation, Site Qualification, and Acceptance |
| [ADR-044](adr/ADR-044-declare-and-enforce-the-supported-operating-envelope.md) | Declare and Enforce the Supported Operating Envelope |
| [ADR-045](adr/ADR-045-maintain-a-safety-case-and-hazard-control-traceability.md) | Maintain a Safety Case and Hazard-Control Traceability |
| [ADR-046](adr/ADR-046-establish-security-privacy-and-model-incident-response.md) | Establish Security, Privacy, and Model Incident Response |
| [ADR-047](adr/ADR-047-control-product-claims-regulatory-classification-and-quality-records.md) | Control Product Claims, Regulatory Classification, and Quality Records |
| [ADR-048](adr/ADR-048-design-commercial-support-warranty-and-field-operations.md) | Design Commercial Support, Warranty, and Field Operations |
| [ADR-049](adr/ADR-049-define-disaster-recovery-factory-reset-and-decommissioning.md) | Define Disaster Recovery, Factory Reset, and Decommissioning |
| [ADR-050](adr/ADR-050-optimize-only-against-profiled-budgets-and-fidelity-gates.md) | Optimize Only Against Profiled Budgets and Fidelity Gates |
| [ADR-051](adr/ADR-051-provide-a-stable-cli-and-minimal-local-control-api.md) | Provide a Stable CLI and Minimal Local Control API |
| [ADR-052](adr/ADR-052-separate-audit-security-and-diagnostic-records.md) | Separate Audit, Security, and Diagnostic Records |
| [ADR-053](adr/ADR-053-gate-capabilities-with-signed-feature-policy.md) | Gate Capabilities with Signed Feature Policy |
| [ADR-054](adr/ADR-054-version-documentation-runbooks-and-support-matrices.md) | Version Documentation, Runbooks, and Support Matrices |
| [ADR-055](adr/ADR-055-require-a-production-readiness-review-and-acceptance-evidence.md) | Require a Production Readiness Review and Acceptance Evidence |
| [ADR-056](adr/ADR-056-operate-post-market-surveillance-and-continuous-risk-review.md) | Operate Post-Market Surveillance and Continuous Risk Review |

## Lifecycle

1. A decision begins as `Proposed` and receives named technical, scientific, security, privacy, product, or regulatory deciders as applicable.
2. Acceptance means the decision and obligations are approved; it does not mean implementation is complete.
3. Implementation evidence is linked before status becomes `Implemented`.
4. Materially changed decisions receive a new ADR that amends or supersedes the old one.
5. The ADR graph is rebuilt after status or relationship changes and must remain free of dangling references and cycles.
6. ADR-055 is the integrated release gate; no subset of ADR count alone establishes production readiness.
