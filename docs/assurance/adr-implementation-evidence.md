# ADR Accepted-to-Implemented Evidence Gate

**Record ID:** STE-QMS-ADR-001
**Owner:** Architecture Owner (Runtime Engineering Lead)
**Gatekeeper:** Quality Lead

An ADR status describes evidence, not intent or code volume. `Accepted` means authorized direction. `Implemented` means every applicable normative obligation is demonstrably implemented for the declared scope. Scaffolding, merged code, unchecked boxes, a successful build, or author assertion is insufficient.

## Mandatory evidence package

Before `Accepted` may become `Implemented`, the ADR must have:

1. A decision owner and independent reviewer, explicit implementation scope, and exact release/code commit.
2. An extracted obligation list covering every “must,” “require,” “forbid,” invariant, negative consequence mitigation, and linked DDD invariant.
3. Trace rows linking each obligation to production code/config/schema and objective verification records.
4. Passing applicable unit/property, contract, boundary, fuzz, replay, integration, migration, security, HIL, soak, benchmark, and validation gates; omitted tiers have approved non-applicability rationale.
5. Negative tests for forbidden paths and safe-state/failure tests for critical controls.
6. Updated threat, privacy, hazard, claim, data-class, dependency/SBOM, operating-envelope, runbook, and support records where impacted.
7. No unresolved critical defect, critical hazard, prohibited-use bypass, forbidden/unknown commercial license, exposed secret, unreviewed unsafe code, or incompatible contract/migration.
8. Documented residual risks using the [risk acceptance format](assurance-ownership.md), with unexpired approvals.
9. Reproducible evidence artifacts with digests, tools/environment, thresholds, complete results, negative results/deviations, and independent review.
10. A Quality Lead gate decision binding the ADR revision, evidence manifest digest, and UTC time.

## Evidence manifest

```yaml
adr: ADR-NNN
adr_revision_digest: ""
scope: {release: "", commits: [], hardware: [], models: [], configurations: [], markets: []}
owner: ""
independent_reviewer: ""
obligations:
  - id: REQ-ADR-NNN-01
    statement: ""
    implementation_refs: []
    verification_record_ids: []
    result: pass
impact_record_ids: {threats: [], hazards: [], claims: [], dependencies: [], documentation: []}
exceptions_and_risk_acceptance_ids: []
evidence_artifact_digests: []
quality_decision: {decision: implemented | reject, name: "", signature: "", signed_at_utc: ""}
```

## Status changes

- `Proposed -> Accepted`: deciders approve the direction and known consequences; implementation evidence is not implied.
- `Accepted -> Implemented`: the complete package above passes.
- `Implemented -> Superseded`: the replacement links and migration/decommission evidence exist.
- Any evidence invalidation, scope expansion, failed regression, vulnerability, hazard, or changed claim returns affected scope to `Accepted` or `Suspended` operationally until reassessed; history is preserved.

## Review test

A reviewer must be able to choose any ADR obligation and navigate to exact implementation, test procedure, raw result, environment, review, residual-risk decision, and release scope—and navigate back from the evidence to that obligation. If this cannot be done, the ADR is not `Implemented`.
