//! Fail-closed non-medical respiration CLI tests.

use std::{cell::Cell, process::Command};
use ste_cli::{
    RespirationCommand, RespirationCommandError, RespirationOperations, execute_respiration_command,
};
use ste_consent_governance::domain::{
    AuthorizationRequest, DenialReason, ParticipantPseudonym, PolicyDecision, PolicyVersion,
    Purpose, SpaceId,
};
use ste_runtime::{GovernanceGate, RequestOrigin};

fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("lab").unwrap(),
        participants: [ParticipantPseudonym::new("operator").unwrap()]
            .into_iter()
            .collect(),
        purpose: Purpose::Wellness,
        policy_version: PolicyVersion::new(1).unwrap(),
        evaluated_at: 1,
    }
}
struct Ops(Cell<u8>);
impl RespirationOperations for Ops {
    fn status(&self, _: &str) -> Result<String, RespirationCommandError> {
        self.0.set(self.0.get() + 1);
        Ok(
            "{\"capability\":\"respiration-v1\",\"enabled\":false,\"claim\":\"non-medical\"}"
                .into(),
        )
    }
    fn validate(&self, _: &str) -> Result<String, RespirationCommandError> {
        self.0.set(self.0.get() + 1);
        Ok("{\"gate\":\"not-promoted\",\"reference_agreement\":null}".into())
    }
}

#[test]
fn parses_only_exact_status_and_validation_queries() {
    assert_eq!(
        RespirationCommand::parse(["status", "resp-baseline-v1"]).unwrap(),
        RespirationCommand::Status {
            model_id: "resp-baseline-v1".into()
        }
    );
    assert!(RespirationCommand::parse(["status"]).is_err());
    assert!(RespirationCommand::parse(["enable", "resp-baseline-v1"]).is_err());
}

#[test]
fn policy_denial_prevents_report_access() {
    let ops = Ops(Cell::new(0));
    let gate = GovernanceGate::new(|_| PolicyDecision::Denied(DenialReason::NotGranted));
    assert_eq!(
        execute_respiration_command(
            &RespirationCommand::Status {
                model_id: "resp-baseline-v1".into()
            },
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &ops
        ),
        Err(RespirationCommandError::AuthorizationRequired)
    );
    assert_eq!(ops.0.get(), 0);
}

#[test]
fn status_is_explicitly_disabled_and_non_medical() {
    let ops = Ops(Cell::new(0));
    let gate = GovernanceGate::new(|_| PolicyDecision::Authorized);
    let output = execute_respiration_command(
        &RespirationCommand::Status {
            model_id: "resp-baseline-v1".into(),
        },
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &ops,
    )
    .unwrap();
    assert!(output.contains("\"enabled\":false"));
    assert!(output.contains("non-medical"));
}

#[test]
fn direct_cli_has_no_unauthenticated_status_bypass() {
    let status = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["respiration", "status", "resp-baseline-v1"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(77));
}
