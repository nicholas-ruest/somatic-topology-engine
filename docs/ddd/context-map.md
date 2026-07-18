# DDD Context Map

## Relationship map

```text
Consent & Governance ──policy──> Radio Acquisition
        │                           │
        │ policy                    │ Published Language: validated frames
        v                           v
Device Interaction <──projection── State Inference
        ^                           ^
        │                           │ Customer/Supplier: physiology evidence
        │                           │
        │                    Physiology Estimation
        │                           ^
        │                           │ Customer/Supplier: observations
        │                           │
        └──── status ───── Signal Observation
                                    ^
                                    │
                              Radio Acquisition

Personalization Memory <── explicit ports ──> State Inference
Experiment Validation <── replay/read models ── all evidence-producing contexts
Consent & Governance ── retention/redaction policy ──> all persistent stores
```

## Context relationships

| Upstream | Downstream | Pattern | Contract |
|---|---|---|---|
| Consent and Governance | Radio Acquisition | Conformist policy gate | `CapturePermission` query; fail closed |
| Radio Acquisition | Signal Observation | Published language | `ValidatedCsiFrameV1`, `CaptureHealthV1` |
| Signal Observation | Physiology Estimation | Customer/supplier | `ObservationWindowClosedV1` |
| Physiology Estimation | State Inference | Customer/supplier | `PhysiologyEvidenceUpdatedV1` |
| Personalization Memory | State Inference | Open host service | profile query and explicit reward command |
| State Inference | Device Interaction | Published language | `DisplayProjectionV1`; never raw model output |
| Device Interaction | Personalization Memory | Anticorruption layer | touch becomes an `AnchorRequestedV1`, not direct storage |
| Experiment Validation | Evidence contexts | Separate ways | immutable replay and exported read models |
| Consent and Governance | Persistence adapters | Policy | retention, deletion, encryption, audit requirements |

## Dependency rules

1. Dependency direction follows the evidence pipeline; upstream contexts know nothing about downstream interpretations.
2. Governance policy may gate any context but does not own its scientific or signal-processing rules.
3. Experiment Validation reads immutable exports and issues promotion decisions; it never edits production evidence.
4. Device Interaction cannot invoke model internals. It sends commands and renders approved projections.
5. TypeScript adapters communicate through serialized contracts at the application/infrastructure edge. Core Rust domain crates do not link to Node.js.

## Suggested Rust workspace

```text
crates/
  ste-kernel/                 # IDs, time, confidence, provenance, domain-event traits
  ste-radio-acquisition/
  ste-signal-observation/
  ste-physiology-estimation/
  ste-state-inference/
  ste-personalization-memory/
  ste-experiment-validation/
  ste-device-interaction/
  ste-consent-governance/
  ste-contracts/              # serde integration-event DTOs only
  ste-runtime/                # composition root and async supervision
  ste-cli/
adapters/
  typescript/                 # optional AgentDB/DSPy.ts/Ruflo bridges
```

This is a target layout. Crates should be extracted only when implementation begins; the initial modular monolith may keep multiple modules in fewer crates while enforcing the same dependency rules.
