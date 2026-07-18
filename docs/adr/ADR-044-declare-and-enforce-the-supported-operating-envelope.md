# ADR-044: Declare and Enforce the Supported Operating Envelope

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: operating-envelope, multiperson, interference, scope

## Context

Multiple people, pets, fans, speech, typing, posture, walls, motion, AP traffic, and room changes can confound CSI and physiology. Commercial claims require explicit conditions under which outputs are supported.

## Decision

Define a versioned operating envelope per capability covering participant count, identity assumptions, position/orientation, distance, motion, activities, room geometry, interference, AP/link profile, environment, calibration age, packet quality, and participant/model scope.

Detect observable envelope violations and abstain. Initial physiology and latent-state capabilities are single-consenting-participant, stationary, qualified-site features unless validation promotes a broader scope. Multi-person source separation is a separate capability requiring its own datasets, references, gates, hazards, and claim language.

## Consequences

### Positive
- Marketing, UI, models, and field support share enforceable limits.
- Confounders become test cases rather than disclaimers.

### Negative
- Initial product scope is narrower than the concept vision.
- Some violations cannot be detected perfectly.

### Neutral
- The envelope may expand only through versioned validation evidence.

## Links

**Depends on**: [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md), [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md), [ADR-029](ADR-029-version-csi-calibration-baselines-and-operating-geometry.md), [ADR-034](ADR-034-calibrate-uncertainty-detect-out-of-distribution-inputs-and-abstain.md), [ADR-043](ADR-043-use-guided-installation-site-qualification-and-acceptance.md)
