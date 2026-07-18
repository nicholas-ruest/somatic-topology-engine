//! Governed simulator command tests.
use std::{cell::Cell, process::Command};
use ste_cli::{
    DeviceSimulatorCommand, DeviceSimulatorError, DeviceSimulatorOperations,
    execute_device_simulator,
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
impl DeviceSimulatorOperations for Ops {
    fn render(&self, _: &str) -> Result<String, DeviceSimulatorError> {
        self.0.set(self.0.get() + 1);
        Ok("{\"text\":\"Calibrating\",\"color\":\"Blue\"}".into())
    }
}
#[test]
fn arbitrary_or_valence_labels_cannot_be_rendered() {
    assert!(DeviceSimulatorCommand::parse(["render", "inferred-valence"]).is_err());
}
#[test]
fn denial_prevents_peripheral_dispatch() {
    let ops = Ops(Cell::new(0));
    let gate = GovernanceGate::new(|_| PolicyDecision::Denied(DenialReason::NotGranted));
    assert_eq!(
        execute_device_simulator(
            &DeviceSimulatorCommand {
                projection: "calibrating".into()
            },
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &ops
        ),
        Err(DeviceSimulatorError::AuthorizationRequired)
    );
    assert_eq!(ops.0.get(), 0);
}
#[test]
fn direct_process_is_fail_closed() {
    let status = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["device-sim", "render", "calibrating"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(77));
}
