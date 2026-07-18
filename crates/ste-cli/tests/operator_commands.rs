//! Broad authenticated idempotent operator-contract tests.
use std::{cell::Cell, process::Command};
use ste_cli::{
    IdempotentOperatorService, OperatorAction, OperatorBackend, OperatorCommand,
    OperatorCommandError, OperatorResponse, execute_operator_command,
};
use ste_consent_governance::domain::{
    AuthorizationRequest, DenialReason, ParticipantPseudonym, PolicyDecision, PolicyVersion,
    Purpose, SpaceId,
};
use ste_runtime::{GovernanceGate, RequestOrigin};
fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("site").unwrap(),
        participants: [ParticipantPseudonym::new("operator").unwrap()]
            .into_iter()
            .collect(),
        purpose: Purpose::Wellness,
        policy_version: PolicyVersion::new(1).unwrap(),
        evaluated_at: 1,
    }
}
struct Backend(Cell<u8>);
impl OperatorBackend for Backend {
    fn run(&self, c: &OperatorCommand) -> Result<OperatorResponse, OperatorCommandError> {
        self.0.set(self.0.get() + 1);
        Ok(OperatorResponse {
            schema_major: 1,
            action: format!("{:?}", c.action),
            status: "ok".into(),
            idempotency_key: c.idempotency_key.clone(),
            changed: !c.dry_run,
            message: "completed offline".into(),
        })
    }
}
#[test]
fn parses_every_supported_operator_action_with_stable_schema() {
    for action in [
        "status",
        "doctor",
        "authorization",
        "capture-test",
        "hardware-probe",
        "calibration",
        "replay",
        "validation-export",
        "models",
        "capability-policy",
        "support-bundle",
        "updates",
        "data-lifecycle",
        "recovery",
        "reset",
        "commission",
        "requalify",
    ] {
        assert!(
            OperatorCommand::parse([action, "--idempotency", "key", "--json"]).is_ok(),
            "{action}"
        );
    }
}
#[test]
fn exact_retry_returns_same_receipt_and_key_reuse_with_different_request_fails() {
    let service = IdempotentOperatorService::new(Backend(Cell::new(0)));
    let gate = GovernanceGate::new(|_| PolicyDecision::Authorized);
    let first = OperatorCommand::parse(["doctor", "--idempotency", "k", "--json"]).unwrap();
    let a = execute_operator_command(
        &first,
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &service,
    )
    .unwrap();
    let b = execute_operator_command(
        &first,
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &service,
    )
    .unwrap();
    assert_eq!(a, b);
    let conflicting = OperatorCommand::parse(["status", "--idempotency", "k", "--json"]).unwrap();
    assert_eq!(
        execute_operator_command(
            &conflicting,
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &service
        ),
        Err(OperatorCommandError::OperationFailed)
    );
}
#[test]
fn reset_requires_confirmation_and_denial_prevents_dispatch() {
    let service = IdempotentOperatorService::new(Backend(Cell::new(0)));
    let denied = GovernanceGate::new(|_| PolicyDecision::Denied(DenialReason::NotGranted));
    let reset = OperatorCommand {
        action: OperatorAction::Reset,
        idempotency_key: "r".into(),
        json: true,
        dry_run: false,
        confirmed: false,
    };
    assert_eq!(
        execute_operator_command(
            &reset,
            &request(),
            RequestOrigin::LocalOperator,
            &denied,
            &service
        ),
        Err(OperatorCommandError::ConfirmationRequired)
    );
    let doctor = OperatorCommand::parse(["doctor", "--idempotency", "d"]).unwrap();
    assert_eq!(
        execute_operator_command(
            &doctor,
            &request(),
            RequestOrigin::LocalOperator,
            &denied,
            &service
        ),
        Err(OperatorCommandError::AuthorizationRequired)
    );
}
#[test]
fn direct_process_has_no_authenticated_ipc_bypass() {
    let status = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["operator", "commission", "--idempotency", "k", "--json"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(77));
}
