# Bounded Context: State Inference

## Purpose

Estimate operationalized latent constructs from valid physiological evidence, contextual features, and an approved personalized model. This context owns scientific claim boundaries and must prefer abstention over unsupported labels.

## Aggregate: `StateAssessment`

**Identity:** `StateAssessmentId`

**Entities:** `ConstructEstimate`, `EvidenceBundle`, `InferenceDecision`, `AbstentionRecord`

**Value objects:** `ConstructDefinition`, `ClaimLevel`, `EvidenceHorizon`, `CalibratedProbability`, `ModelScope`, `ModelVersion`, `CalibrationId`, `Provenance`, `AbstentionReason`

### Invariants

1. Observation, physiology, and latent-state types remain distinct at compile time.
2. A construct must have a versioned operational definition and an Experiment Validation promotion before inference is enabled.
3. Initial production claim levels exclude valence and decision phase; workload is task-specific, never universal.
4. Every result includes evidence IDs, horizon, model scope, calibration, probability, and claim level.
5. Out-of-scope user, room, posture, hardware, or task conditions cause abstention unless the model card explicitly covers them.
6. Raw model output is never a user-facing projection.
7. A state transition is debounced under a versioned temporal policy.

### Commands

- `AssessLatentState`
- `EnableValidatedConstruct`
- `DisableConstruct`
- `ApplyTemporalUpdate`

### Events

- `LatentStateEstimated`
- `LatentStateInferenceAbstained`
- `StateTransitionDetected`
- `ConstructEnabled`
- `ConstructDisabled`

### Repository port

`StateAssessmentRepository` stores immutable assessments. `ModelRegistry`, `PatternProfileReader`, and `ValidationRegistry` are explicit ports; they cannot mutate the aggregate during evaluation.

## Rust and TypeScript boundaries

The live inference and temporal logic are Rust. Optional TypeScript prompt tooling may consume a policy-approved projection offline, but it cannot create or change a `StateAssessment`.
