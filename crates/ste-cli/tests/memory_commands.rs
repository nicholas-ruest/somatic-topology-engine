//! Governed participant-memory command tests.
use std::{cell::Cell, process::Command};
use ste_cli::{MemoryCommand, MemoryCommandError, MemoryOperations, execute_memory_command};
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
impl MemoryOperations for Ops {
    fn view(&self, _: &str) -> Result<String, MemoryCommandError> {
        self.0.set(self.0.get() + 1);
        Ok("[]".into())
    }
    fn correct(&self, _: &str, _: &str, _: f32) -> Result<String, MemoryCommandError> {
        self.0.set(self.0.get() + 1);
        Ok("appended".into())
    }
    fn delete(&self, _: &str) -> Result<String, MemoryCommandError> {
        self.0.set(self.0.get() + 1);
        Ok("cryptographic_erasure=true;index_rebuilt=true".into())
    }
}
#[test]
fn delete_requires_confirmation_before_authorization_or_dispatch() {
    let ops = Ops(Cell::new(0));
    let gate = GovernanceGate::new(|_| PolicyDecision::Authorized);
    assert_eq!(
        execute_memory_command(
            &MemoryCommand::Delete {
                participant: "p1".into(),
                confirmed: false
            },
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &ops
        ),
        Err(MemoryCommandError::ConfirmationRequired)
    );
    assert_eq!(ops.0.get(), 0);
}
#[test]
fn denial_prevents_scoped_memory_access() {
    let ops = Ops(Cell::new(0));
    let gate = GovernanceGate::new(|_| PolicyDecision::Denied(DenialReason::NotGranted));
    assert_eq!(
        execute_memory_command(
            &MemoryCommand::View {
                participant: "p1".into()
            },
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &ops
        ),
        Err(MemoryCommandError::AuthorizationRequired)
    );
    assert_eq!(ops.0.get(), 0);
}
#[test]
fn direct_process_delete_is_fail_closed() {
    let status = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["memory", "delete", "p1", "--confirm"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(77));
}
