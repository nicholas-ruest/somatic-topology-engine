# Bounded Context: Personalization Memory

## Purpose

Own versioned user anchors, pattern retrieval, explicit feedback, adaptation state, and provenance without corrupting historical evidence or scientific evaluation.

## Aggregate: `PatternProfile`

**Identity:** `PatternProfileId`

**Entities:** `Anchor`, `PatternEmbedding`, `FeedbackRecord`, `AdaptationVersion`, `DataPartition`

**Value objects:** `ParticipantPseudonym`, `AnchorLabel`, `EmbeddingVector`, `Reward`, `ModelVersion`, `CalibrationId`, `SessionId`, `PartitionRole`, `Provenance`, `RetentionClass`

### Invariants

1. An anchor is append-only and preserves its source assessment and observation provenance.
2. Feedback never rewrites an earlier label, estimate, or reward; correction is a new linked record.
3. Evaluation/test partitions cannot update retrieval weights, embeddings, or models.
4. Every adaptation has a parent version and the exact feedback set used to derive it.
5. Retrieval is scoped by participant and policy; cross-user retrieval is prohibited by default.
6. Deletion policy can tombstone and cryptographically erase user-associated payloads without falsifying audit history.
7. “Improved” may be claimed only after prospective held-out evaluation.

### Commands

- `CreatePatternProfile`
- `RecordAnchor`
- `RecordFeedback`
- `BuildAdaptationVersion`
- `FreezeEvaluationPartition`
- `ForgetParticipantData`

### Events

- `PatternProfileCreated`
- `AnchorRecorded`
- `FeedbackRecorded`
- `AdaptationVersionBuilt`
- `EvaluationPartitionFrozen`
- `ParticipantDataForgotten`

### Repository port

`PatternProfileRepository` stores aggregate metadata. `VectorMemory` is a separate port implemented first with Rust RuVector/RVF. An optional AgentDB adapter may be added through a TypeScript sidecar only if it demonstrates a needed capability and equivalent local/privacy guarantees.
