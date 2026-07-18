# Phase 12 state inference evidence

Status: baseline policy mechanics only; no trained cognitive model or production construct promoted.

## Exact capability status

- Generic mental-state inference: prohibited and unrepresentable.
- Valence and decision phase: disabled.
- Stress, emotion, intent, and universal cognitive workload: disabled.
- Task-specific workload: disabled absent an exact frozen Experiment Validation promotion and signed capability policy.

No output in this phase is a reading of thoughts or feelings. Physiological evidence and signal observations remain distinct types and cannot be relabeled as latent constructs.

## Integration controls

The immutable assessment repository preserves estimates and abstentions, accepts exact retries idempotently, and rejects conflicting replacement. The only UI/API boundary is `DisplayProjectionV1`, built from approved domain output through fixed templates. It contains an enumerated availability/claim state and bounded provenance references; it contains no raw score, feature vector, tensor, physiology value, participant identity, or generative text.

Every assessment must trace to physiology and observation evidence and pass exact construct promotion, active model, calibration, signed policy, participant purpose, model scope, operating envelope, confidence, and temporal debounce gates. Any failed or missing gate emits a typed unavailable/abstention projection. Configuration cannot enable a failed scientific gate.

## Evidence interpretation

Deterministic no-op and synthetic baseline tests validate software policy, replay, and projection isolation only. They provide no human-validity evidence. A trained task-specific model would require a separately authorized, frozen protocol with strong time/task/motion baselines and held-out participants/sessions/days before promotion.

Run `scripts/validate-state-inference.sh` for executable evidence. Raspberry Pi, human-reference, HIL, thermal, soak, and promoted construct benchmarks remain pending unless separately attached.

The executable suite currently passes seven state-inference tests, two governed CLI tests, and strict Clippy. It verifies every mandatory gate, baseline no-claim behavior, immutable conflict rejection, abstention projection, and explicit leakage canaries for raw probability, observation/physiology/model/calibration/profile references, participant/room scope, feature vectors, and tensors.
