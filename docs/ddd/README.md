# Somatic Topology Engine Domain Model

This model translates the [project brief](../../.plans/description.md) and [research synthesis](../research.md) into bounded contexts and aggregates. It is an architectural model, not a claim that the proposed cognitive outputs have been validated.

## Modeling rules

- Rust owns acquisition, signal processing, inference, memory, device I/O, policy enforcement, and experiment metrics whenever a suitable crate or direct implementation is practical.
- TypeScript is permitted only behind explicit ports for tools whose maintained API is TypeScript-first, such as optional AgentDB, DSPy.ts, or Ruflo integrations.
- The core is a modular monolith deployed as one edge process initially. Context boundaries are Rust crate/module and API boundaries, not network services.
- Domain objects never import hardware drivers, databases, neural runtimes, clocks, or TypeScript packages. Those are infrastructure adapters.
- Cross-context communication uses public application commands, queries, or versioned integration events.
- Observation, physiology estimate, and latent-state estimate are distinct concepts and must never share a type.
- Every uncertain estimate can abstain. Confidence without provenance, evidence horizon, and calibration identity is invalid.

## Documents

- [Context map](context-map.md)
- [Ubiquitous language](ubiquitous-language.md)
- [Radio Acquisition](contexts/radio-acquisition.md)
- [Signal Observation](contexts/signal-observation.md)
- [Physiology Estimation](contexts/physiology-estimation.md)
- [State Inference](contexts/state-inference.md)
- [Personalization Memory](contexts/personalization-memory.md)
- [Experiment Validation](contexts/experiment-validation.md)
- [Device Interaction](contexts/device-interaction.md)
- [Consent and Governance](contexts/consent-governance.md)

## Aggregate inventory

| Context | Aggregate root | Primary invariant |
|---|---|---|
| Radio Acquisition | `CaptureSession` | Accepted frames share a pinned capture profile and monotonic session timeline. |
| Signal Observation | `ObservationWindow` | Derived observations retain source-frame evidence and quality state. |
| Physiology Estimation | `PhysiologyAssessment` | An estimate is emitted only for a validated modality/window combination or as an abstention. |
| State Inference | `StateAssessment` | Latent state never masquerades as direct observation and cannot bypass an approved evidence gate. |
| Personalization Memory | `PatternProfile` | Feedback and anchors cannot mutate historical evidence or contaminate held-out evaluation data. |
| Experiment Validation | `ValidationStudy` | Protocol, splits, metrics, and gates are frozen before evaluation results are admitted. |
| Device Interaction | `InteractionSession` | Displays render only policy-approved projections; anchors require an explicit touch event. |
| Consent and Governance | `SensingAuthorization` | Capture is prohibited unless authorization is active for the space and participants. |

## Production-supporting boundaries

The following concerns are deliberately not modeled as additional business aggregates. They are platform, assurance, or operating capabilities surrounding the eight domain contexts:

- runtime supervision, backpressure, configuration, persistence, and schema evolution;
- device identity, cryptography, updates, administrative authorization, and incident response;
- model registry, dataset governance, validation laboratories, and release evidence;
- hardware qualification, site commissioning, support, warranty, and post-market review.

Their decisions are indexed in the [complete ADR catalog](../adr-index.md). When implementation reveals a distinct ubiquitous language, lifecycle, and invariant-owning entity in one of these areas, add a bounded context through a new ADR rather than turning infrastructure records into domain aggregates prematurely.
