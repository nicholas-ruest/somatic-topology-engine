# Bounded Context: Physiology Estimation

## Purpose

Interpret clean observation windows as uncertain estimates of explicitly defined physiological variables. It enforces modality-specific validation status, evidence horizons, and abstention.

## Aggregate: `PhysiologyAssessment`

**Identity:** `PhysiologyAssessmentId`

**Entities:** `ModalityEstimate`, `ReferenceValidation`, `AbstentionRecord`

**Value objects:** `PhysiologyModality`, `EstimateValue`, `EvidenceHorizon`, `CalibratedConfidence`, `ValidationStatus`, `ErrorBounds`, `StillnessRequirement`, `AbstentionReason`, `ModelVersion`

### Invariants

1. Allowed modalities are versioned and defined; initial promotion order is respiration, then cardiac rate, then interval-derived metrics.
2. Every estimate records source observation IDs, evidence horizon, model/algorithm, calibration, confidence, and validation status.
3. The output is an abstention when quality, stillness, horizon, or validation rules fail.
4. Cardiac interval or HRV-like output is prohibited until the relevant reference-validation gate is passed.
5. A 500 ms presentation cadence cannot shorten a modality's evidence horizon.
6. Environmental measurements are covariates only unless a preregistered model validates a causal correction.
7. No estimate is marked medical-grade.

### Commands

- `AssessPhysiology`
- `RecordReferenceComparison`
- `PromoteModality`
- `WithdrawModality`

### Events

- `PhysiologyEstimated`
- `PhysiologyEstimationAbstained`
- `ReferenceValidationRecorded`
- `PhysiologyModalityPromoted`
- `PhysiologyModalityWithdrawn`

### Repository port

`PhysiologyAssessmentRepository` stores immutable assessments. Promotion policy is queried through a separate `ValidationRegistry` port controlled by Experiment Validation.

## Rust boundaries

Estimators and calibration logic are Rust. ruv-FANN may implement a small model behind `PhysiologyModel`; deterministic preprocessing and feature parity are tested independently of the model runtime.
