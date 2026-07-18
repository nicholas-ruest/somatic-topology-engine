//! Site qualification, requalification, signature, and recovery acceptance tests.
use ed25519_dalek::SigningKey;
use ste_commissioning::*;

fn complete(session: &mut CommissioningSession) {
    for kind in CheckKind::ALL {
        session
            .record_check(
                kind,
                CheckOutcome::passed(format!("evidence-{kind:?}")).unwrap(),
            )
            .unwrap();
    }
}

#[test]
fn missing_or_failed_mandatory_check_cannot_issue_acceptance() {
    let key = SigningKey::from_bytes(&[1; 32]);
    let mut missing = CommissioningSession::start("c1", "site", "profile", Vec::new()).unwrap();
    assert_eq!(
        missing.qualify(1, "key", &key),
        Err(CommissioningError::MandatoryCheckMissingOrFailed)
    );
    let mut failed = CommissioningSession::start("c2", "site", "profile", Vec::new()).unwrap();
    failed
        .record_check(CheckKind::Thermal, CheckOutcome::failed("hot").unwrap())
        .unwrap();
    assert_eq!(failed.state(), &CommissioningState::Rejected);
    assert_eq!(
        failed.qualify(1, "key", &key),
        Err(CommissioningError::TerminalState)
    );
}

#[test]
fn signed_acceptance_enables_only_capabilities_with_passing_coverage() {
    let key = SigningKey::from_bytes(&[2; 32]);
    let mut session = CommissioningSession::start(
        "c",
        "site",
        "profile",
        vec!["respiration".into(), "workload".into()],
    )
    .unwrap();
    complete(&mut session);
    session
        .record_coverage(CapabilityCoverage::new("respiration", true, "coverage-1").unwrap())
        .unwrap();
    session
        .record_coverage(CapabilityCoverage::new("workload", false, "coverage-2").unwrap())
        .unwrap();
    let signed = session.qualify(10, "site-key", &key).unwrap();
    assert_eq!(signed.verify(&key.verifying_key()), Ok(()));
    assert!(signed.enables("respiration"));
    assert!(!signed.enables("workload"));
}

#[test]
fn tampering_invalidates_acceptance_and_requalification_links_previous_record() {
    let key = SigningKey::from_bytes(&[3; 32]);
    let mut session =
        CommissioningSession::requalify("c", "site", "profile-v2", Vec::new(), "acceptance:old")
            .unwrap();
    complete(&mut session);
    let mut signed = session.qualify(20, "key", &key).unwrap();
    assert_eq!(
        signed.record.previous_record.as_deref(),
        Some("acceptance:old")
    );
    signed.record.site_id = "attacker-site".into();
    assert_eq!(
        signed.verify(&key.verifying_key()),
        Err(CommissioningError::InvalidSignature)
    );
}

#[test]
fn recovery_mode_cannot_manufacture_a_qualified_record() {
    let key = SigningKey::from_bytes(&[4; 32]);
    let mut session = CommissioningSession::start("c", "site", "profile", Vec::new()).unwrap();
    session.enter_recovery("storage repair").unwrap();
    assert_eq!(
        session.qualify(1, "key", &key),
        Err(CommissioningError::TerminalState)
    );
}
