//! Governed projection query tests.
use std::{cell::Cell, process::Command};
use ste_cli::{
    StateProjectionCommand, StateProjectionCommandError, StateProjectionOperations,
    execute_state_projection,
};
use ste_consent_governance::domain::{
    AuthorizationRequest, DenialReason, ParticipantPseudonym, PolicyDecision, PolicyVersion,
    Purpose, SpaceId,
};
use ste_runtime::{GovernanceGate, RequestOrigin};
fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("room").unwrap(),
        participants: [ParticipantPseudonym::new("operator").unwrap()]
            .into_iter()
            .collect(),
        purpose: Purpose::Wellness,
        policy_version: PolicyVersion::new(1).unwrap(),
        evaluated_at: 1,
    }
}
struct Ops(Cell<u8>);
impl StateProjectionOperations for Ops {
    fn projection(&self, _: &str) -> Result<String, StateProjectionCommandError> {
        self.0.set(self.0.get() + 1);
        Ok("{\"availability\":\"unavailable\",\"reason\":\"ConstructNotPromoted\"}".into())
    }
}
#[test]
fn denial_prevents_projection_read() {
    let ops = Ops(Cell::new(0));
    let gate = GovernanceGate::new(|_| PolicyDecision::Denied(DenialReason::NotGranted));
    assert_eq!(
        execute_state_projection(
            &StateProjectionCommand {
                assessment_id: "a1".into()
            },
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &ops
        ),
        Err(StateProjectionCommandError::AuthorizationRequired)
    );
    assert_eq!(ops.0.get(), 0);
}
#[test]
fn direct_process_is_fail_closed() {
    let status = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["state", "projection", "a1"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(77));
}
