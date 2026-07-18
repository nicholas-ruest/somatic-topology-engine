# ADR-032: Standardize Edge Model Packages and the Rust Inference Port

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: inference, ruv-fann, models, arm

## Context

ruv-FANN is a plausible Rust inference engine, but STE must not bind its domain to one model runtime or load weights without preprocessing, scope, and integrity metadata.

## Decision

Define a Rust `InferenceModel` port and signed `ModelPackage` containing model format, weights, feature schema, preprocessing digest, outputs, calibration, operating scope, training/evaluation lineage, resource profile, license, and compatibility constraints.

Use ruv-FANN first when it satisfies numerical parity, ARM performance, and required operators. Permit another Rust-native runtime through the same port when justified. Load packages in a quarantined validation step, verify bounds and digests, run known-answer tests, then atomically activate. No Python runtime or network model call is permitted on the edge path.

## Consequences

### Positive
- Model engine is replaceable without changing domain semantics.
- Corrupt or incompatible models fail before serving.

### Negative
- Package tooling and engine parity tests are required.
- Some research architectures may need simplification for edge support.

### Neutral
- Model training may use other languages when reproducible and isolated from the runtime.

## Links

**Depends on**: [ADR-021](ADR-021-enforce-slos-and-resource-budgets-with-benchmarks.md), [ADR-023](ADR-023-govern-dependencies-licenses-and-software-supply-chain.md), [ADR-031](ADR-031-use-immutable-feature-and-evidence-artifacts.md)
