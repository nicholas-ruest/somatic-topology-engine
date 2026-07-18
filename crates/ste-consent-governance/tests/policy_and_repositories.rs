//! Policy and repository adapter contracts.

use std::collections::BTreeSet;
use std::sync::Arc;
use ste_consent_governance::application::{
    DiagnosticRecord, GovernanceApplicationService, GovernanceRecords, PolicyDecisionPoint,
    SecurityRecord, SensingAuthorizationRepository,
};
use ste_consent_governance::domain::*;
use ste_consent_governance::{
    FileAuthorizationRepository, InMemoryAuthorizationRepository, SeparatedInMemoryRecords,
};

fn id() -> SensingAuthorizationId {
    SensingAuthorizationId::new("auth-1").unwrap()
}

#[test]
fn diagnostics_are_redacted_and_record_categories_remain_separate() {
    let records = SeparatedInMemoryRecords::default();
    let diagnostic = DiagnosticRecord::redacted("config_error", "password=do-not-log");
    assert_eq!(diagnostic.detail, "[REDACTED]");
    records.diagnostic(diagnostic);
    records.security(SecurityRecord {
        code: "auth_failure",
    });
    assert_eq!(records.counts(), (0, 0, 1, 1));
}
fn participant() -> ParticipantPseudonym {
    ParticipantPseudonym::new("person-a").unwrap()
}
fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("lab").unwrap(),
        participants: BTreeSet::from([participant()]),
        purpose: Purpose::Research,
        policy_version: PolicyVersion::new(1).unwrap(),
        evaluated_at: 10,
    }
}

fn active_authorization() -> SensingAuthorization {
    let mut aggregate = SensingAuthorization::new(id(), PolicyVersion::new(1).unwrap());
    aggregate
        .authorize_space(SpaceId::new("lab").unwrap())
        .unwrap();
    for class in DataClass::ALL {
        aggregate
            .apply_retention_policy(RetentionRule::new(
                class,
                RetentionPeriod::new(1_000).unwrap(),
            ))
            .unwrap();
    }
    aggregate
        .record_participant_consent(
            participant(),
            Purpose::Research,
            ConsentVersion::new(1).unwrap(),
            100,
            0,
        )
        .unwrap();
    aggregate
        .grant(
            SpaceId::new("lab").unwrap(),
            Purpose::Research,
            BTreeSet::from([participant()]),
            100,
            0,
        )
        .unwrap();
    aggregate
}

#[test]
fn policy_decision_point_fails_closed_for_missing_state() {
    let repository = Arc::new(InMemoryAuthorizationRepository::default());
    let pdp = PolicyDecisionPoint::new(repository);
    assert_eq!(
        pdp.evaluate(&id(), &request()),
        PolicyDecision::Denied(DenialReason::NotGranted)
    );
}

#[test]
fn in_memory_repository_round_trips_without_aliasing() {
    let repository = InMemoryAuthorizationRepository::default();
    let aggregate = SensingAuthorization::new(id(), PolicyVersion::new(1).unwrap());
    repository.save(&aggregate).unwrap();
    assert_eq!(repository.load(&id()).unwrap(), Some(aggregate));
}

#[test]
fn durable_repository_survives_adapter_restart_and_rejects_path_traversal() {
    let root = std::env::temp_dir().join(format!("ste-governance-{}", std::process::id()));
    let traversal = SensingAuthorizationId::new("../../outside").unwrap();
    let aggregate = SensingAuthorization::new(traversal.clone(), PolicyVersion::new(1).unwrap());
    FileAuthorizationRepository::new(&root)
        .unwrap()
        .save(&aggregate)
        .unwrap();
    let reopened = FileAuthorizationRepository::new(&root).unwrap();
    assert_eq!(reopened.load(&traversal).unwrap(), Some(aggregate));
    assert!(!root.parent().unwrap().join("outside.json").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn durable_repository_reports_corrupt_state_instead_of_authorizing() {
    let root = std::env::temp_dir().join(format!("ste-governance-corrupt-{}", std::process::id()));
    let repository = FileAuthorizationRepository::new(&root).unwrap();
    std::fs::write(root.join("617574682d31.json"), b"not json").unwrap();
    assert!(repository.load(&id()).is_err());
    let pdp = PolicyDecisionPoint::new(Arc::new(repository));
    assert!(matches!(
        pdp.evaluate(&id(), &request()),
        PolicyDecision::Denied(_)
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn application_revocation_is_immediately_observed_and_requests_offline_deletion() {
    let repository = Arc::new(InMemoryAuthorizationRepository::default());
    let records = Arc::new(SeparatedInMemoryRecords::default());
    repository.save(&active_authorization()).unwrap();
    let pdp = PolicyDecisionPoint::new(repository.clone());
    assert_eq!(pdp.evaluate(&id(), &request()), PolicyDecision::Authorized);
    let service = GovernanceApplicationService::new(repository, records.clone());
    let events = service
        .execute(
            &id(),
            GovernanceCommand::RevokeConsent {
                participant: participant(),
                reason: RevocationReason::ParticipantRequest,
                revoked_at: 11,
            },
            11,
        )
        .unwrap();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, GovernanceEvent::ParticipantDeletionRequested { .. }))
    );
    assert_eq!(
        pdp.evaluate(&id(), &request()),
        PolicyDecision::Denied(DenialReason::Revoked)
    );
    assert_eq!(records.counts(), (2, 2, 0, 0));
}
