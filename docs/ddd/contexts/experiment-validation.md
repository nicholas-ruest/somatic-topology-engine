# Bounded Context: Experiment Validation

## Purpose

Own preregistered protocols, reference comparisons, dataset partitions, metrics, capability promotion, and reproducible evidence. It prevents implementation convenience from becoming a scientific claim.

## Aggregate: `ValidationStudy`

**Identity:** `ValidationStudyId`

**Entities:** `Protocol`, `Cohort`, `DatasetPartition`, `MetricDefinition`, `Baseline`, `Gate`, `StudyRun`, `ResultSet`

**Value objects:** `ProtocolVersion`, `TargetConstruct`, `ReferenceModality`, `SplitStrategy`, `MetricThreshold`, `ConfidenceInterval`, `GateDecision`, `EthicsApproval`, `ArtifactDigest`

### Invariants

1. Protocol, targets, splits, baselines, metrics, and gates are frozen before a confirmatory run starts.
2. Windows from one session cannot cross train/test boundaries.
3. Reference alignment error and missingness are recorded.
4. Results include coverage and abstention, not accuracy on emitted estimates alone.
5. A promotion requires all mandatory gates; failure and negative results remain immutable.
6. Human-subject collection cannot begin without recorded ethics/consent authorization appropriate to the jurisdiction.
7. Every run links code, data, firmware, model, calibration, and environment digests.

### Commands

- `RegisterProtocol`
- `FreezeProtocol`
- `StartStudyRun`
- `RecordStudyResult`
- `EvaluateGate`
- `PromoteCapability`
- `RejectCapability`

### Events

- `ValidationProtocolRegistered`
- `ValidationProtocolFrozen`
- `StudyRunCompleted`
- `ValidationGatePassed`
- `ValidationGateFailed`
- `CapabilityPromoted`
- `CapabilityRejected`

### Repository port

`ValidationStudyRepository` stores protocols and results. `EvidenceExportReader` reads immutable, de-identified exports; it has no command access to operational aggregates.

## Rust boundaries

Metrics, splits, artifact verification, and reports are Rust where practical. Python is allowed only for a scientifically necessary library without a suitable Rust implementation, behind a reproducible batch adapter—not in the edge runtime.
