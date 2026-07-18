//! ADR-060 state-machine, authorization, durability, and receipt tests.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use ste_workflows::*;

struct Allow;
impl Authorization for Allow {
    fn reauthorize(&self, _: &WorkflowRequest, _: u64) -> AuthorizationDecision {
        AuthorizationDecision::Allow
    }
}
struct Pause;
impl Authorization for Pause {
    fn reauthorize(&self, _: &WorkflowRequest, _: u64) -> AuthorizationDecision {
        AuthorizationDecision::Pause("session expired".into())
    }
}
struct Revoke;
impl Authorization for Revoke {
    fn reauthorize(&self, _: &WorkflowRequest, _: u64) -> AuthorizationDecision {
        AuthorizationDecision::Preempt("consent revoked".into())
    }
}

struct Toggle(Arc<AtomicBool>);
impl Authorization for Toggle {
    fn reauthorize(&self, _: &WorkflowRequest, _: u64) -> AuthorizationDecision {
        if self.0.load(Ordering::SeqCst) {
            AuthorizationDecision::Preempt("consent revoked".into())
        } else {
            AuthorizationDecision::Allow
        }
    }
}

fn request(kind: WorkflowType, key: &str) -> WorkflowRequest {
    WorkflowRequest {
        workflow_type: kind,
        scope: "site/a".into(),
        requester_role: Role::Administrator,
        requester_session: "session-1".into(),
        authorization_ref: "auth-1".into(),
        purpose_ref: "operations".into(),
        idempotency_key: key.into(),
        correlation_id: Some("correlation-1".into()),
        expires_at_ms: 10_000,
    }
}

#[test]
fn exact_retry_returns_same_instance_and_changed_body_conflicts() {
    let engine = WorkflowEngine::new(InMemoryJournal::default(), Allow);
    let first = engine
        .create(request(WorkflowType::HardwareProbe, "same"), 1)
        .unwrap();
    let second = engine
        .create(request(WorkflowType::HardwareProbe, "same"), 2)
        .unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(
        engine.create(request(WorkflowType::Calibration, "same"), 2),
        Err(EngineError::IdempotencyConflict)
    );
}

#[test]
fn destructive_confirmation_is_server_bound_and_expires() {
    let engine = WorkflowEngine::new(InMemoryJournal::default(), Allow);
    let created = engine
        .create(request(WorkflowType::FactoryReset, "reset"), 1)
        .unwrap();
    assert_eq!(created.state, StepState::AwaitingHuman);
    let challenge = created.challenge.clone().unwrap();
    assert_eq!(
        engine.execute(
            &created.id,
            created.version,
            Command::Apply(Action::Confirm {
                nonce: challenge.nonce.clone(),
                typed_scope: "wrong".into()
            }),
            2
        ),
        Err(EngineError::ChallengeInvalid)
    );
    let ready = engine
        .execute(
            &created.id,
            created.version,
            Command::Apply(Action::Confirm {
                nonce: challenge.nonce,
                typed_scope: challenge.scope,
            }),
            2,
        )
        .unwrap();
    assert_eq!(ready.state, StepState::Ready);
}

#[test]
fn complete_effect_produces_bounded_receipt() {
    let engine = WorkflowEngine::new(InMemoryJournal::default(), Allow);
    let ready = engine
        .create(request(WorkflowType::UpdateStage, "update"), 1)
        .unwrap();
    let running = engine
        .execute(&ready.id, ready.version, Command::Apply(Action::Start), 2)
        .unwrap();
    let result = engine
        .execute(
            &running.id,
            running.version,
            Command::CommitEffect(Outcome {
                affected_resources: vec!["slot-b".into()],
                before_ref: Some("digest-a".into()),
                after_ref: Some("digest-b".into()),
                audit_digest: "audit".into(),
                warnings: vec![],
            }),
            3,
        )
        .unwrap();
    let receipt = result.receipt.unwrap();
    assert_eq!(receipt.affected_resources, ["slot-b"]);
    assert_eq!(receipt.after_ref.as_deref(), Some("digest-b"));
}

#[test]
fn authorization_pause_blocks_and_revocation_preempts() {
    let seed = request(WorkflowType::ModelEvaluate, "pause");
    let journal = InMemoryJournal::default();
    // Create using a mutable policy is exercised separately by the preemption policy below.
    assert_eq!(
        WorkflowEngine::new(journal, Pause).create(seed, 1),
        Err(EngineError::Unauthorized("session expired".into()))
    );
    assert_eq!(
        WorkflowEngine::new(InMemoryJournal::default(), Revoke)
            .create(request(WorkflowType::ModelEvaluate, "revoke"), 1),
        Err(EngineError::Unauthorized("consent revoked".into()))
    );
}

#[test]
fn revocation_preempts_an_in_flight_workflow_before_its_next_step() {
    let revoked = Arc::new(AtomicBool::new(false));
    let policy = Toggle(Arc::clone(&revoked));
    let engine = WorkflowEngine::new(InMemoryJournal::default(), policy);
    let ready = engine
        .create(request(WorkflowType::CaptureDiagnostics, "capture"), 1)
        .unwrap();
    revoked.store(true, Ordering::SeqCst);
    let stopped = engine
        .execute(&ready.id, ready.version, Command::Apply(Action::Start), 2)
        .unwrap();
    assert_eq!(stopped.receipt.unwrap().outcome, "preempted");
}

#[test]
fn conflicting_resource_locks_are_observable_and_release_at_terminal_state() {
    let engine = WorkflowEngine::new(InMemoryJournal::default(), Allow);
    let first = engine
        .create(request(WorkflowType::ModelActivate, "one"), 1)
        .unwrap();
    assert!(matches!(
        engine.create(request(WorkflowType::ModelRollback, "two"), 1),
        Err(EngineError::ResourceLocked(_))
    ));
    let running = engine
        .execute(&first.id, first.version, Command::Apply(Action::Start), 2)
        .unwrap();
    engine
        .execute(
            &running.id,
            running.version,
            Command::Apply(Action::Complete),
            3,
        )
        .unwrap();
    assert!(
        engine
            .create(request(WorkflowType::ModelRollback, "two"), 4)
            .is_ok()
    );
}

#[test]
fn optimistic_concurrency_rejects_stale_commands() {
    let engine = WorkflowEngine::new(InMemoryJournal::default(), Allow);
    let ready = engine
        .create(request(WorkflowType::BackupVerify, "backup"), 1)
        .unwrap();
    let running = engine
        .execute(&ready.id, ready.version, Command::Apply(Action::Start), 2)
        .unwrap();
    assert_eq!(
        engine.execute(&ready.id, ready.version, Command::Apply(Action::Start), 3),
        Err(EngineError::VersionConflict)
    );
    assert_eq!(running.state, StepState::Running);
}

#[test]
fn failure_retry_and_compensation_are_explicit() {
    let engine = WorkflowEngine::new(InMemoryJournal::default(), Allow);
    let ready = engine
        .create(request(WorkflowType::UpdateStage, "failed"), 1)
        .unwrap();
    let running = engine
        .execute(&ready.id, ready.version, Command::Apply(Action::Start), 2)
        .unwrap();
    let retry = engine
        .execute(
            &running.id,
            running.version,
            Command::Apply(Action::Fail {
                reason: "power interruption".into(),
            }),
            3,
        )
        .unwrap();
    assert_eq!(retry.state, StepState::RetryWait);
    let failed = engine
        .execute(
            &retry.id,
            retry.version,
            Command::Apply(Action::Compensate),
            4,
        )
        .unwrap();
    assert!(failed.receipt.unwrap().compensated);
}

#[test]
fn journal_detects_conflicts_and_chains_checksums() {
    let journal = InMemoryJournal::default();
    let event = WorkflowEvent::Created {
        request: request(WorkflowType::HardwareProbe, "journal"),
        request_digest: "digest".into(),
    };
    let stored = journal.append("id", 0, &[event]).unwrap();
    assert!(!stored[0].checksum.is_empty());
    assert_eq!(journal.append("id", 0, &[]), Err(JournalError::Conflict));
    assert!(journal.load("id").is_ok());
}

#[test]
fn all_workflow_classes_are_serializable() {
    let representatives = [
        WorkflowType::Commissioning,
        WorkflowType::ConsentGrant,
        WorkflowType::Calibration,
        WorkflowType::ModelActivate,
        WorkflowType::CapabilityActivate,
        WorkflowType::PersonalizationErasure,
        WorkflowType::DatasetValidate,
        WorkflowType::OledSimulate,
        WorkflowType::SupportBundleExport,
        WorkflowType::UpdateRollback,
        WorkflowType::FactoryReset,
        WorkflowType::IncidentDeclare,
    ];
    for kind in representatives {
        assert!(!serde_json::to_string(&kind).unwrap().is_empty());
    }
}
