//! Exact-scope approval, blocking, expiry, and suspension acceptance tests.
use ed25519_dalek::SigningKey;
use std::collections::BTreeSet;
use ste_commercial_readiness::*;
fn set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|v| (*v).into()).collect()
}
fn scope() -> ReleaseScope {
    ReleaseScope::new(
        "pi5-rev1",
        "ste-1.0-sha",
        set(&["resp-v1"]),
        set(&["CA"]),
        "envelope-v1",
        "support-v1",
    )
    .unwrap()
}
fn review() -> ReadinessReview {
    let binding = "scope-sha256".to_string();
    let evidence = Criterion::ALL
        .into_iter()
        .map(|c| {
            (
                c,
                CriterionEvidence::new(c, format!("digest-{c:?}"), true, 1_000, &binding).unwrap(),
            )
        })
        .collect();
    ReadinessReview {
        scope: scope(),
        scope_binding: binding,
        evidence,
        residual_risks: vec![ResidualRisk::new("r1", 2, true, "risk-evidence").unwrap()],
        exceptions: Vec::new(),
    }
}

#[test]
fn complete_exact_scope_review_produces_time_bound_signed_approval() {
    let key = SigningKey::from_bytes(&[12; 32]);
    let signed = ReadinessEngine::decide(review(), 100, 500, "board", &key).unwrap();
    assert_eq!(signed.decision.status, DecisionStatus::Approved);
    assert!(signed.authorizes_release(499));
    assert!(!signed.authorizes_release(500));
    assert_eq!(
        signed.verify(&key.verifying_key(), &scope(), "scope-sha256", 200),
        Ok(())
    );
}

#[test]
fn missing_legal_pilot_hil_or_claim_evidence_always_blocks_with_correction() {
    for criterion in [
        Criterion::LegalRegulatory,
        Criterion::PilotAcceptance,
        Criterion::HardwareInLoop,
        Criterion::ClaimEvidence,
    ] {
        let key = SigningKey::from_bytes(&[13; 32]);
        let mut review = review();
        review.evidence.remove(&criterion);
        review.exceptions.push(ReadinessException {
            id: "exception".into(),
            condition: "ship anyway".into(),
            approved_by: "operator".into(),
            expires_at: 999,
        });
        let signed = ReadinessEngine::decide(review, 100, 500, "board", &key).unwrap();
        assert_eq!(signed.decision.status, DecisionStatus::Blocked);
        assert!(
            signed
                .decision
                .corrective_actions
                .iter()
                .any(|a| a.reason == BlockReason::MissingCriterion(criterion))
        );
    }
}

#[test]
fn expired_mismatched_evidence_and_unaccepted_or_critical_risk_block() {
    let key = SigningKey::from_bytes(&[14; 32]);
    let mut review = review();
    review
        .evidence
        .get_mut(&Criterion::SafetyCase)
        .unwrap()
        .expires_at = 50;
    review
        .evidence
        .get_mut(&Criterion::ClaimEvidence)
        .unwrap()
        .scope_binding = "other".into();
    review
        .residual_risks
        .push(ResidualRisk::new("r2", 4, false, "open").unwrap());
    review
        .residual_risks
        .push(ResidualRisk::new("r3", 5, true, "claimed accepted").unwrap());
    let signed = ReadinessEngine::decide(review, 100, 500, "board", &key).unwrap();
    assert_eq!(signed.decision.status, DecisionStatus::Blocked);
    assert_eq!(signed.decision.corrective_actions.len(), 4);
}

#[test]
fn incident_suspends_prior_scope_and_tampering_or_scope_drift_fails_verification() {
    let key = SigningKey::from_bytes(&[15; 32]);
    let approved = ReadinessEngine::decide(review(), 100, 500, "board", &key).unwrap();
    let mut suspended = ReadinessEngine::suspend(
        &approved,
        "incident-1",
        200,
        300,
        "incident-commander",
        &key,
    )
    .unwrap();
    assert_eq!(suspended.decision.status, DecisionStatus::Suspended);
    assert!(!suspended.authorizes_release(250));
    suspended.decision.scope.software = "other".into();
    assert_eq!(
        suspended.verify(&key.verifying_key(), &scope(), "scope-sha256", 250),
        Err(ReadinessError::InvalidSignature)
    );
}
