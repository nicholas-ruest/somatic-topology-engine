# Intended Use and Use Restrictions

**Record ID:** STE-QMS-IU-001
**Baseline:** 0.1
**Status:** Draft — not approved for commercial claims
**Effective date:** 2026-07-18
**Accountable owner:** Product & Regulatory Lead
**Required approvers:** Safety Lead, Security & Privacy Lead, Scientific Validation Lead, Quality Lead, Executive Release Authority

## Intended use

The Somatic Topology Engine (STE) is a local-first, non-medical research and wellness platform for consenting adults in a qualified indoor space. It processes compatible Wi-Fi channel-state information on supported edge hardware to produce quality-gated observations. A separately validated and policy-enabled release may present narrowly defined wellness or research estimates within its documented operating envelope.

STE is operated by a trained installer or administrator. Every person whose data may be captured must be covered by an active, purpose-specific, time-bounded authorization. Sensing state must be visible, and a physical means to stop sensing must remain available. Local operation does not relax consent, privacy, validation, or security controls.

No physiological or latent-state capability is part of this baseline merely because the software architecture can represent it. Each capability is disabled until its preregistered validation gate, claim review, operating-envelope qualification, and signed capability policy are complete.

## Intended users and environment

- Consenting adults participating in approved research or choosing a supported wellness feature.
- Trained installers and local administrators operating supported Raspberry Pi/CrowPi and Wi-Fi capture hardware.
- Qualified researchers using de-identified exports under a recorded protocol and data-use authority.
- Indoor, non-emergency, non-clinical spaces that pass site qualification for geometry, occupants, radio conditions, hardware, thermal/power conditions, and physical sensing indication.

Children, adults unable to give valid consent, custodial populations, workplaces, schools, healthcare decision settings, and public or covertly monitored spaces are outside the baseline unless a new approved regulatory, ethics, safety, and consent program explicitly brings them into scope.

## Excluded and prohibited uses

The following uses are prohibited and must be unrepresentable in production policy, configuration, adapters, UI, API, documentation, and sales material:

1. Covert sensing or capture without active space, purpose, and participant authorization.
2. Identity, re-identification, face-equivalent recognition, or tracking a person across spaces.
3. Medical diagnosis, screening, triage, treatment, alarm, vital-sign monitoring, or replacement for a clinician or approved medical device.
4. Emergency, life-support, public-safety, or safety-critical control decisions.
5. Employment, education, credit, insurance, housing, policing, immigration, benefits, or other high-impact eligibility/scoring decisions.
6. Deception detection, interrogation, surveillance, productivity scoring, disciplinary monitoring, or inference of protected/sensitive traits.
7. Inferring emotion, valence, intention, stress, workload, cardiac coherence, HRV, or decision phase without a separately promoted construct and approved claim; generic arousal or motion is not a substitute.
8. Secondary use, model training, personalization, data sale, advertising, or sharing beyond the authorized purpose.
9. Circumventing abstention, quality gates, retention, deletion, physical-off, revocation, operating-envelope, or capability-policy controls.
10. Use of experimental output as production evidence or as the sole basis for consequential action.

## Baseline claims

At this stage the only permitted claim is: “STE is software under development for consent-gated, local-first processing and replay of supported RF sensing data.” This is an engineering statement, not a performance, health, privacy-certification, or commercial-readiness claim. Permitted future claims must appear in [the claim-evidence matrix](claim-evidence-matrix.md) before publication.

## Change control

A change to capability, target user, market, model, sensor, deployment setting, UI label, API semantic, sales statement, or data use requires Product & Regulatory Lead triage. Material changes require updated hazard/threat analysis, qualified jurisdictional review, validation, and approval of this record. “Research,” “wellness,” or “local” language cannot be used to evade obligations arising from actual functionality.

## References

- [ADR-009](../adr/ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md)
- [ADR-045](../adr/ADR-045-maintain-a-safety-case-and-hazard-control-traceability.md)
- [ADR-047](../adr/ADR-047-control-product-claims-regulatory-classification-and-quality-records.md)
- [Consent and Governance context](../ddd/contexts/consent-governance.md)
