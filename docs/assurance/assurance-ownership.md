# Assurance Ownership and Risk Acceptance

**Record ID:** STE-QMS-GOV-001
**Process owner:** Quality Lead

Role names below are accountable functions, not committees. Before pilot or commercial distribution, the Executive Release Authority must assign a named, trained person and deputy to every role. A vacant or conflicted required role blocks the applicable approval.

| Role | Accountable decisions | Cannot self-approve |
|---|---|---|
| Executive Release Authority | Final release and residual-risk acceptance within delegated authority | Evidence they produced; prohibited-use exceptions |
| Product & Regulatory Lead | Intended use, claim language, markets, regulatory/legal review | Scientific validity or final release |
| Quality Lead | Quality system, records, deviations, CAPA, verification completeness, readiness review | A verification they executed |
| Safety Lead | Hazard analysis, safe states, control adequacy, safety residual risk | Release alone |
| Security & Privacy Lead | Threat/privacy model, vulnerability triage, penetration evidence, disclosure/incident response | Business acceptance of critical security/privacy risk |
| Scientific Validation Lead | Protocols, datasets, statistical gates, model/capability promotion | Product claims or release alone |
| Supply-Chain & Licensing Lead | Dependency intake, SBOM, provenance, license compatibility | Legal conclusions without qualified counsel |
| Runtime Engineering Lead | Rust architecture, implementation, tests, benchmarks, operations | Their own assurance review |
| Hardware & Field Lead | Firmware/hardware qualification, HIL, commissioning, operating envelope | Scientific model promotion |
| Support & Incident Lead | Support window, complaint handling, exercises, field response | Closing safety/security CAPA alone |

## Decision rules

- Prohibited uses are not risk-acceptable exceptions. Changing one requires a new architectural decision, intended-use revision, qualified legal/regulatory and ethics review, and explicit product redesign.
- No individual may produce evidence and independently approve the same evidence. Electronic signatures must bind identity, role, record digest, decision, and UTC time.
- Critical hazards and critical security/privacy risks require Safety Lead or Security & Privacy Lead recommendation, Quality Lead concurrence, and Executive Release Authority acceptance. An unknown or unbounded impact blocks release.
- Commercial-license blockers require written resolution from the Supply-Chain & Licensing Lead and qualified counsel; business risk acceptance cannot override a license obligation.
- Time-limited acceptance expires automatically and cannot survive a release, scope, or evidence change unless re-approved.

## Risk acceptance record

Use one immutable record per risk:

```yaml
risk_acceptance_id: RA-YYYY-NNN
risk_or_hazard_id: ""
decision: accept | reject | remediate | defer
scope:
  release_digest: ""
  hardware: []
  models: []
  configurations: []
  jurisdictions: []
description: ""
foreseeable_harm: ""
initial_severity: ""
initial_likelihood: ""
controls: []
verification_record_ids: []
residual_severity: ""
residual_likelihood: ""
rationale: ""
alternatives_considered: []
user_disclosure_or_warning: ""
monitoring_and_trigger: ""
expires_at_utc: ""
accepted_by: [{name: "", role: "", signature: "", signed_at_utc: ""}]
independent_concurrence: [{name: "", role: "", signature: "", signed_at_utc: ""}]
record_digest: ""
```

Acceptance is invalid if fields are blank, controls are not verified, approvers lack authority, evidence is mutable/unresolved, or scope exceeds the evaluated configuration.

## Escalation

Safety, consent, covert-sensing, exploitable critical vulnerability, key compromise, material privacy breach, invalid scientific claim, or license-rights concerns immediately suspend the affected capability/release. The relevant lead opens an incident/CAPA record and escalates to the Executive Release Authority; schedule pressure is not an acceptance rationale.
