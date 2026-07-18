//! Outside-in tests for the policy enforcement point around capture and privileged commands.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use ste_consent_governance::domain::{
    AuthorizationRequest, ConsentVersion, DataClass, DenialReason, ParticipantPseudonym,
    PolicyDecision, PolicyVersion, Purpose, RetentionPeriod, RetentionRule, RevocationReason,
    SensingAuthorization, SensingAuthorizationId, SpaceId,
};
use ste_runtime::{
    GateError, GovernanceGate, PrivilegedCommand, RequestOrigin, SafeGovernanceState,
};

fn participant(value: &str) -> ParticipantPseudonym {
    ParticipantPseudonym::new(value).unwrap()
}

fn authorized_policy() -> Rc<RefCell<SensingAuthorization>> {
    let person = participant("participant-a");
    let mut policy = SensingAuthorization::new(
        SensingAuthorizationId::new("authorization-a").unwrap(),
        PolicyVersion::new(1).unwrap(),
    );
    policy
        .authorize_space(SpaceId::new("room-a").unwrap())
        .unwrap();
    for data_class in DataClass::ALL {
        policy
            .apply_retention_policy(RetentionRule::new(
                data_class,
                RetentionPeriod::new(3_600).unwrap(),
            ))
            .unwrap();
    }
    policy
        .record_participant_consent(
            person.clone(),
            Purpose::Wellness,
            ConsentVersion::new(1).unwrap(),
            1_000,
            1,
        )
        .unwrap();
    policy
        .grant(
            SpaceId::new("room-a").unwrap(),
            Purpose::Wellness,
            BTreeSet::from([person]),
            1_000,
            1,
        )
        .unwrap();
    Rc::new(RefCell::new(policy))
}

fn request(purpose: Purpose, now: u64) -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("room-a").unwrap(),
        participants: BTreeSet::from([participant("participant-a")]),
        purpose,
        policy_version: PolicyVersion::new(1).unwrap(),
        evaluated_at: now,
    }
}

#[test]
fn no_frame_reaches_the_sink_without_an_active_exact_authorization() {
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| {
        PolicyDecision::Denied(DenialReason::NotGranted)
    });
    let mut published = Vec::new();

    let result = gate.publish_capture(
        &request(Purpose::Wellness, 10),
        RequestOrigin::CaptureAdapter,
        vec![1_u8, 2, 3],
        |frame| published.push(frame),
    );

    assert_eq!(result, Err(GateError::Denied(DenialReason::NotGranted)));
    assert!(published.is_empty());
    assert_eq!(
        gate.safe_state(),
        SafeGovernanceState::Unauthorized(DenialReason::NotGranted)
    );
}

#[test]
fn revocation_blocks_the_very_next_publication_without_restarting_runtime() {
    let policy = authorized_policy();
    let evaluated = Rc::clone(&policy);
    let gate = GovernanceGate::new(move |request: &AuthorizationRequest| {
        evaluated.borrow().evaluate(request)
    });
    let mut published = Vec::new();
    gate.publish_capture(
        &request(Purpose::Wellness, 10),
        RequestOrigin::CaptureAdapter,
        1_u8,
        |frame| published.push(frame),
    )
    .unwrap();

    policy
        .borrow_mut()
        .revoke_consent(
            participant("participant-a"),
            RevocationReason::ParticipantRequest,
            11,
        )
        .unwrap();
    let result = gate.publish_capture(
        &request(Purpose::Wellness, 11),
        RequestOrigin::CaptureAdapter,
        2_u8,
        |frame| published.push(frame),
    );

    assert_eq!(result, Err(GateError::Denied(DenialReason::Revoked)));
    assert_eq!(published, [1]);
}

#[test]
fn config_feature_admin_adapter_and_sidecar_cannot_widen_purpose() {
    let policy = authorized_policy();
    let evaluated = Rc::clone(&policy);
    let gate = GovernanceGate::new(move |request: &AuthorizationRequest| {
        evaluated.borrow().evaluate(request)
    });

    for origin in [
        RequestOrigin::Configuration,
        RequestOrigin::FeaturePolicy,
        RequestOrigin::Administrator,
        RequestOrigin::CaptureAdapter,
        RequestOrigin::Sidecar,
    ] {
        let result = gate.authorize_command(
            &request(Purpose::Research, 10),
            origin,
            PrivilegedCommand::StartCapture,
        );
        assert_eq!(
            result,
            Err(GateError::Denied(DenialReason::PurposeMismatch)),
            "origin {origin:?} widened purpose"
        );
    }
}

#[test]
fn prohibited_purpose_is_denied_even_if_a_defective_evaluator_authorizes_it() {
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| PolicyDecision::Authorized);
    let mut published = false;

    let result = gate.publish_capture(
        &request(Purpose::CovertSensing, 10),
        RequestOrigin::Sidecar,
        (),
        |_| published = true,
    );

    assert_eq!(
        result,
        Err(GateError::Denied(DenialReason::ProhibitedPurpose))
    );
    assert!(!published);
}

#[test]
fn privileged_command_returns_an_unforgeable_short_lived_grant() {
    let policy = authorized_policy();
    let evaluated = Rc::clone(&policy);
    let gate = GovernanceGate::new(move |request: &AuthorizationRequest| {
        evaluated.borrow().evaluate(request)
    });

    let grant = gate
        .authorize_command(
            &request(Purpose::Wellness, 10),
            RequestOrigin::LocalOperator,
            PrivilegedCommand::StartCapture,
        )
        .unwrap();

    assert_eq!(grant.command(), PrivilegedCommand::StartCapture);
    assert_eq!(grant.origin(), RequestOrigin::LocalOperator);
}
