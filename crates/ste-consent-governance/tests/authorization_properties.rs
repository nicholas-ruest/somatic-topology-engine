//! Adversarial properties for the sensing-authorization aggregate.

use std::collections::BTreeSet;

use proptest::prelude::*;
use ste_consent_governance::domain::{
    AuthorizationRequest, AuthorizationState, ConsentVersion, DataClass, DenialReason, DomainError,
    ParticipantPseudonym, PolicyDecision, PolicyVersion, Purpose, RetentionPeriod, RetentionRule,
    RevocationReason, SensingAuthorization, SensingAuthorizationId, SpaceId,
};

fn participant(value: &str) -> ParticipantPseudonym {
    ParticipantPseudonym::new(value).expect("valid participant")
}

fn aggregate(required: &[ParticipantPseudonym], expires_at: u64) -> SensingAuthorization {
    let mut authorization = SensingAuthorization::new(
        SensingAuthorizationId::new("auth-1").unwrap(),
        PolicyVersion::new(3).unwrap(),
    );
    let space = SpaceId::new("room-a").unwrap();
    authorization.authorize_space(space.clone()).unwrap();
    for data_class in DataClass::ALL {
        authorization
            .apply_retention_policy(RetentionRule::new(
                data_class,
                RetentionPeriod::new(86_400).unwrap(),
            ))
            .unwrap();
    }
    for person in required {
        authorization
            .record_participant_consent(
                person.clone(),
                Purpose::Wellness,
                ConsentVersion::new(2).unwrap(),
                expires_at,
                10,
            )
            .unwrap();
    }
    authorization
        .grant(
            space,
            Purpose::Wellness,
            required.iter().cloned().collect(),
            expires_at,
            10,
        )
        .unwrap();
    authorization
}

fn request(
    participants: impl IntoIterator<Item = ParticipantPseudonym>,
    purpose: Purpose,
    now: u64,
) -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("room-a").unwrap(),
        participants: participants.into_iter().collect(),
        purpose,
        policy_version: PolicyVersion::new(3).unwrap(),
        evaluated_at: now,
    }
}

proptest! {
    #[test]
    fn removing_any_required_participant_always_denies(
        count in 1usize..24,
        removed in 0usize..24,
    ) {
        let people: Vec<_> = (0..count).map(|index| participant(&format!("p-{index}"))).collect();
        let authorization = aggregate(&people, 1_000);
        let removed = removed % count;
        let presented = people.iter().enumerate()
            .filter(|(index, _)| *index != removed)
            .map(|(_, person)| person.clone());

        prop_assert_eq!(
            authorization.evaluate(&request(presented, Purpose::Wellness, 100)),
            PolicyDecision::Denied(DenialReason::ParticipantSetMismatch),
        );
    }

    #[test]
    fn expiry_is_fail_closed_at_and_after_the_exact_boundary(
        lifetime in 1u64..1_000_000,
        delay in 0u64..1_000_000,
    ) {
        let expires_at = 11 + lifetime;
        let people = [participant("p-1")];
        let authorization = aggregate(&people, expires_at);
        let now = expires_at.saturating_add(delay);
        prop_assert_eq!(
            authorization.evaluate(&request(people, Purpose::Wellness, now)),
            PolicyDecision::Denied(DenialReason::Expired),
        );
    }

    #[test]
    fn prohibited_purposes_can_never_be_granted(index in 0usize..6) {
        let prohibited = [
            Purpose::IdentityInference,
            Purpose::ClinicalDiagnosis,
            Purpose::EmploymentScoring,
            Purpose::DeceptionDetection,
            Purpose::CovertSensing,
            Purpose::UnrelatedSecondaryUse,
        ][index];
        let mut authorization = aggregate(&[participant("p-1")], 1_000);
        let result = authorization.grant(
            SpaceId::new("room-a").unwrap(),
            prohibited,
            BTreeSet::from([participant("p-1")]),
            2_000,
            20,
        );
        prop_assert_eq!(result, Err(DomainError::ProhibitedPurpose(prohibited)));
    }
}

#[test]
fn exact_space_purpose_and_policy_version_are_required() {
    let people = [participant("p-1")];
    let authorization = aggregate(&people, 1_000);

    let mut wrong_space = request(people.clone(), Purpose::Wellness, 100);
    wrong_space.space = SpaceId::new("other").unwrap();
    assert_eq!(
        authorization.evaluate(&wrong_space),
        PolicyDecision::Denied(DenialReason::SpaceMismatch)
    );

    assert_eq!(
        authorization.evaluate(&request(people.clone(), Purpose::Research, 100)),
        PolicyDecision::Denied(DenialReason::PurposeMismatch)
    );

    let mut wrong_policy = request(people, Purpose::Wellness, 100);
    wrong_policy.policy_version = PolicyVersion::new(4).unwrap();
    assert_eq!(
        authorization.evaluate(&wrong_policy),
        PolicyDecision::Denied(DenialReason::PolicyVersionMismatch)
    );
}

#[test]
fn revocation_is_immediate_monotonic_and_schedules_local_deletion() {
    let person = participant("p-1");
    let mut authorization = aggregate(std::slice::from_ref(&person), 1_000);
    let events = authorization
        .revoke_consent(person.clone(), RevocationReason::ParticipantRequest, 200)
        .expect("active consent revokes");

    assert_eq!(authorization.state(), AuthorizationState::Revoked);
    assert!(events.iter().any(|event| matches!(
        event,
        ste_consent_governance::domain::GovernanceEvent::ParticipantDeletionRequested { participant, .. }
            if participant == &person
    )));
    assert_eq!(
        authorization.evaluate(&request([person.clone()], Purpose::Wellness, 200)),
        PolicyDecision::Denied(DenialReason::Revoked)
    );
    assert_eq!(
        authorization.grant(
            SpaceId::new("room-a").unwrap(),
            Purpose::Wellness,
            BTreeSet::from([person]),
            2_000,
            201,
        ),
        Err(DomainError::AuthorizationTerminal)
    );
}

#[test]
fn adapter_cannot_widen_the_original_participant_or_purpose_scope() {
    let p1 = participant("p-1");
    let p2 = participant("p-2");
    let authorization = aggregate(std::slice::from_ref(&p1), 1_000);

    assert_eq!(
        authorization.evaluate(&request([p1.clone(), p2], Purpose::Wellness, 100)),
        PolicyDecision::Denied(DenialReason::ParticipantSetMismatch)
    );
    assert_eq!(
        authorization.evaluate(&request([p1], Purpose::Calibration, 100)),
        PolicyDecision::Denied(DenialReason::PurposeMismatch)
    );
}
