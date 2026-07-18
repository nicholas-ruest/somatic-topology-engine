//! Outside-in storage command authorization and safety tests.

use std::cell::RefCell;
use std::process::Command;
use std::sync::Arc;

use ste_cli::{
    EncryptedExportOperations, SteStorageOperations, StorageCommand, StorageCommandError,
    StorageOperations, execute_storage_command,
};
use ste_consent_governance::domain::{
    AuthorizationRequest, DenialReason, ParticipantPseudonym, PolicyDecision, PolicyVersion,
    Purpose, SpaceId,
};
use ste_runtime::{GovernanceGate, RequestOrigin};
use ste_storage::crypto::DevelopmentKeyProvider;
use ste_storage::lifecycle::LifecycleManager;
use ste_storage::{
    DataClass, EventUpcaster, InMemoryJournalIo, Journal, JournalError, UpcastEvent,
};

#[derive(Default)]
struct RecordingStorage {
    calls: RefCell<Vec<&'static str>>,
}

impl StorageOperations for RecordingStorage {
    fn inspect_journal(&self) -> Result<String, StorageCommandError> {
        self.calls.borrow_mut().push("inspect");
        Ok("journal verified".into())
    }
    fn rebuild_projections(&self, _: bool) -> Result<String, StorageCommandError> {
        self.calls.borrow_mut().push("rebuild");
        Ok("projection rebuild planned".into())
    }
    fn export_manifest(&self, _: &str) -> Result<String, StorageCommandError> {
        self.calls.borrow_mut().push("export");
        Ok("encrypted export manifest written".into())
    }
    fn recover(&self, _: bool) -> Result<String, StorageCommandError> {
        self.calls.borrow_mut().push("recover");
        Ok("recovery planned".into())
    }
    fn delete_participant(&self, _: &str, _: bool) -> Result<String, StorageCommandError> {
        self.calls.borrow_mut().push("delete");
        Ok("deletion planned".into())
    }
    fn factory_reset(&self) -> Result<String, StorageCommandError> {
        self.calls.borrow_mut().push("reset");
        Ok("factory reset complete; capture disabled".into())
    }
    fn decommission(&self) -> Result<String, StorageCommandError> {
        self.calls.borrow_mut().push("decommission");
        Ok("device decommissioned".into())
    }
}

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
fn denied_policy_prevents_every_storage_backend_call() {
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| {
        PolicyDecision::Denied(DenialReason::NotGranted)
    });
    let storage = RecordingStorage::default();

    let result = execute_storage_command(
        StorageCommand::InspectJournal,
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &storage,
    );

    assert_eq!(result, Err(StorageCommandError::AuthorizationRequired));
    assert!(storage.calls.borrow().is_empty());
}

#[test]
fn every_operation_requires_a_fresh_policy_decision() {
    let evaluations = RefCell::new(0_u8);
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| {
        *evaluations.borrow_mut() += 1;
        PolicyDecision::Authorized
    });
    let storage = RecordingStorage::default();
    for command in [
        StorageCommand::InspectJournal,
        StorageCommand::RebuildProjections { dry_run: true },
        StorageCommand::ExportManifest {
            output: "out".into(),
        },
        StorageCommand::Recover { dry_run: true },
        StorageCommand::DeleteParticipant {
            participant: "participant-a".into(),
            dry_run: true,
        },
        StorageCommand::FactoryReset { confirmed: true },
        StorageCommand::Decommission { confirmed: true },
    ] {
        execute_storage_command(
            command,
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &storage,
        )
        .unwrap();
    }
    assert_eq!(*evaluations.borrow(), 7);
    assert_eq!(storage.calls.borrow().len(), 7);
}

#[test]
fn destructive_commands_require_confirmation_before_backend_execution() {
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| PolicyDecision::Authorized);
    let storage = RecordingStorage::default();
    for command in [
        StorageCommand::FactoryReset { confirmed: false },
        StorageCommand::Decommission { confirmed: false },
    ] {
        assert_eq!(
            execute_storage_command(
                command,
                &request(),
                RequestOrigin::Administrator,
                &gate,
                &storage,
            ),
            Err(StorageCommandError::ConfirmationRequired)
        );
    }
    assert!(storage.calls.borrow().is_empty());
}

#[test]
fn rebuild_recovery_and_deletion_default_to_dry_run() {
    assert_eq!(
        StorageCommand::parse(["journal", "rebuild"]).unwrap(),
        StorageCommand::RebuildProjections { dry_run: true }
    );
    assert_eq!(
        StorageCommand::parse(["recover"]).unwrap(),
        StorageCommand::Recover { dry_run: true }
    );
    assert_eq!(
        StorageCommand::parse(["delete", "participant-a"]).unwrap(),
        StorageCommand::DeleteParticipant {
            participant: "participant-a".into(),
            dry_run: true,
        }
    );
}

#[test]
fn direct_cli_invocation_fails_closed_without_authenticated_ipc() {
    let output = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["storage", "journal", "inspect"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(77));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "active authorization required"
    );
}

struct PassThrough;

impl EventUpcaster for PassThrough {
    fn upcast(&self, version: u16, payload: &[u8]) -> Result<UpcastEvent, JournalError> {
        Ok(UpcastEvent {
            schema_version: version,
            payload: payload.to_vec(),
        })
    }
}

struct RecordingExporter(RefCell<Vec<String>>);

impl EncryptedExportOperations for RecordingExporter {
    fn export_encrypted(&self, output: &str) -> Result<String, StorageCommandError> {
        self.0.borrow_mut().push(output.into());
        Ok("encrypted portable export written".into())
    }
}

#[test]
fn concrete_adapter_calls_rust_journal_lifecycle_and_export_ports() {
    let mut journal = Journal::new(InMemoryJournalIo::default(), 1024);
    journal.append(DataClass::Audit, 1, b"payload").unwrap();
    let lifecycle = LifecycleManager::new(vec![], Arc::new(DevelopmentKeyProvider::new()));
    let exporter = RecordingExporter(RefCell::new(Vec::new()));
    let adapter = SteStorageOperations::new(
        &journal,
        &PassThrough,
        DataClass::Audit,
        &lifecycle,
        &exporter,
        &[DataClass::Audit],
        100,
    );

    assert!(adapter.inspect_journal().unwrap().contains("records=1"));
    assert_eq!(
        adapter.export_manifest("approved.rvf").unwrap(),
        "encrypted portable export written"
    );
    assert_eq!(exporter.0.borrow().as_slice(), ["approved.rvf"]);
    assert!(
        adapter
            .factory_reset()
            .unwrap()
            .contains("capture disabled")
    );
}
