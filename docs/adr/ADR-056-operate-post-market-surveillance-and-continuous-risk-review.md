# ADR-056: Operate Post-Market Surveillance and Continuous Risk Review

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: post-market, monitoring, complaints, continuous-improvement

## Context

Laboratory and pilot evidence cannot cover every room, participant, interference pattern, misuse, accessibility issue, or long-term failure. Commercial production creates new evidence and obligations.

## Decision

Maintain privacy-preserving post-market processes for complaints, returns, installation failures, abstention/coverage issues, safety events, security reports, model incidents, support trends, and regulatory changes. Collect telemetry only with opt-in and data minimization; accept participant/operator reports without requiring telemetry.

Periodically review aggregate field evidence against SLOs, hazards, operating envelope, claims, subgroup performance, and residual risk. Trigger investigation, corrective action, capability suspension, model rollback, field notice, update, recall, or ADR revision using defined thresholds. Feed outcomes into validation protocols and release review without silently training on field data.

## Consequences

### Positive
- Production evidence improves safety and commercial sustainability.
- Emerging model and hardware problems can trigger controlled action.

### Negative
- Requires long-term staffing, governance, and customer communication.
- Privacy limits may reduce available diagnostic detail.

### Neutral
- Field observations generate hypotheses; controlled validation remains required for new claims.

## Links

**Depends on**: [ADR-033](ADR-033-govern-model-registration-promotion-activation-and-rollback.md), [ADR-046](ADR-046-establish-security-privacy-and-model-incident-response.md), [ADR-048](ADR-048-design-commercial-support-warranty-and-field-operations.md), [ADR-055](ADR-055-require-a-production-readiness-review-and-acceptance-evidence.md)
