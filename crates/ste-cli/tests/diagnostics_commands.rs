//! Authorized local diagnostics and support-preview tests.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::process::Command;

use ste_cli::{
    DiagnosticsCommand, DiagnosticsError, DiagnosticsOperations, LocalDiagnostics,
    execute_diagnostics,
};
use ste_consent_governance::domain::{
    AuthorizationRequest, DenialReason, ParticipantPseudonym, PolicyDecision, PolicyVersion,
    Purpose, SpaceId,
};
use ste_observability::{
    HealthSnapshot, Record, RecordClass, RedactionSchema, SupportBundleBuilder,
};
use ste_runtime::{GovernanceGate, RequestOrigin};

fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("room-a").unwrap(),
        participants: [ParticipantPseudonym::new("operator-a").unwrap()]
            .into_iter()
            .collect(),
        purpose: Purpose::Wellness,
        policy_version: PolicyVersion::new(1).unwrap(),
        evaluated_at: 10,
    }
}

#[test]
fn concrete_diagnostics_exposes_health_and_manifest_without_canary_value() {
    let health = HealthSnapshot {
        state: "degraded".into(),
        saturated_queues: 2,
        dropped_records: 3,
    };
    let mut schema = RedactionSchema::default();
    schema.allow(RecordClass::Diagnostic, "health", ["state".into()]);
    let mut support = SupportBundleBuilder::new(&schema);
    support.add(Record {
        class: RecordClass::Diagnostic,
        code: "health".into(),
        time_ns: 1,
        fields: BTreeMap::from([
            ("state".into(), "degraded".into()),
            ("raw_csi".into(), "REDACTION_CANARY_SECRET".into()),
        ]),
    });
    let diagnostics = LocalDiagnostics::new(&health, &support);
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| PolicyDecision::Authorized);

    let health_json = execute_diagnostics(
        DiagnosticsCommand::Health,
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &diagnostics,
    )
    .unwrap();
    let preview_json = execute_diagnostics(
        DiagnosticsCommand::SupportPreview,
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &diagnostics,
    )
    .unwrap();
    assert!(health_json.contains("dropped_records"));
    assert!(preview_json.contains("checksum"));
    assert!(!preview_json.contains("REDACTION_CANARY_SECRET"));
    assert!(!preview_json.contains("raw_csi"));
}

struct CountingDiagnostics(Cell<u8>);
impl DiagnosticsOperations for CountingDiagnostics {
    fn health_json(&self) -> Result<String, DiagnosticsError> {
        self.0.set(self.0.get() + 1);
        Ok("{}".into())
    }
    fn support_preview_json(&self) -> Result<String, DiagnosticsError> {
        self.0.set(self.0.get() + 1);
        Ok("{}".into())
    }
}

#[test]
fn policy_denial_prevents_diagnostics_access() {
    let diagnostics = CountingDiagnostics(Cell::new(0));
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| {
        PolicyDecision::Denied(DenialReason::NotGranted)
    });
    assert_eq!(
        execute_diagnostics(
            DiagnosticsCommand::SupportPreview,
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &diagnostics,
        ),
        Err(DiagnosticsError::AuthorizationRequired)
    );
    assert_eq!(diagnostics.0.get(), 0);
}

#[test]
fn direct_cli_is_fail_closed_without_authenticated_ipc() {
    let output = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["diagnostics", "support", "preview", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(77));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "active authorization required"
    );
}

#[test]
fn unsupported_diagnostics_arguments_are_rejected() {
    assert_eq!(
        DiagnosticsCommand::parse(["dump", "all"]),
        Err(DiagnosticsError::InvalidArguments)
    );
}
