//! Governed validation command contract tests.

use std::{cell::Cell, path::Path, process::Command};

use ste_cli::{
    ValidationCommand, ValidationCommandError, ValidationOperations, execute_validation_command,
};
use ste_consent_governance::domain::{
    AuthorizationRequest, DenialReason, ParticipantPseudonym, PolicyDecision, PolicyVersion,
    Purpose, SpaceId,
};
use ste_runtime::{GovernanceGate, RequestOrigin};

fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("validation-lab").unwrap(),
        participants: [ParticipantPseudonym::new("operator").unwrap()]
            .into_iter()
            .collect(),
        purpose: Purpose::Wellness,
        policy_version: PolicyVersion::new(1).unwrap(),
        evaluated_at: 10,
    }
}

struct Operations(Cell<u8>);
impl ValidationOperations for Operations {
    fn validate_dataset(&self, _: &Path) -> Result<String, ValidationCommandError> {
        self.0.set(self.0.get() + 1);
        Ok("valid".into())
    }
    fn export(&self, _: &str) -> Result<String, ValidationCommandError> {
        self.0.set(self.0.get() + 1);
        Ok("export".into())
    }
    fn promote(&self, _: &str, _: &str) -> Result<String, ValidationCommandError> {
        self.0.set(self.0.get() + 1);
        Ok("promoted".into())
    }
    fn reject(&self, _: &str, _: &str, _: &str) -> Result<String, ValidationCommandError> {
        self.0.set(self.0.get() + 1);
        Ok("rejected".into())
    }
}

#[test]
fn parses_only_complete_bounded_commands() {
    assert_eq!(
        ValidationCommand::parse(["export", "study-1"]).unwrap(),
        ValidationCommand::Export {
            study_id: "study-1".into()
        }
    );
    assert!(ValidationCommand::parse(["promote", "study-1"]).is_err());
    assert!(ValidationCommand::parse(["reject", "study-1", "cap", ""]).is_err());
}

#[test]
fn denial_prevents_any_validation_side_effect() {
    let operations = Operations(Cell::new(0));
    let gate = GovernanceGate::new(|_| PolicyDecision::Denied(DenialReason::NotGranted));
    let result = execute_validation_command(
        &ValidationCommand::Promote {
            study_id: "study-1".into(),
            capability: "occupancy-v1".into(),
        },
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &operations,
    );
    assert_eq!(result, Err(ValidationCommandError::AuthorizationRequired));
    assert_eq!(operations.0.get(), 0);
}

#[test]
fn each_decision_gets_a_fresh_policy_evaluation() {
    let evaluations = Cell::new(0_u8);
    let gate = GovernanceGate::new(|_| {
        evaluations.set(evaluations.get() + 1);
        PolicyDecision::Authorized
    });
    let operations = Operations(Cell::new(0));
    for command in [
        ValidationCommand::Promote {
            study_id: "study-1".into(),
            capability: "occupancy-v1".into(),
        },
        ValidationCommand::Reject {
            study_id: "study-1".into(),
            capability: "presence-v1".into(),
            reason: "gate failed".into(),
        },
    ] {
        execute_validation_command(
            &command,
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &operations,
        )
        .unwrap();
    }
    assert_eq!(evaluations.get(), 2);
    assert_eq!(operations.0.get(), 2);
}

#[test]
fn direct_process_is_fail_closed_without_authenticated_ipc() {
    let status = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["validation", "promote", "study-1", "occupancy-v1"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(77));
}
