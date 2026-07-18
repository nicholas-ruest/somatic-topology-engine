use crate::{
    Action, Authorization, AuthorizationDecision, Challenge, ChallengeRequirement, Command,
    EffectPort, Journal, JournalError, Outcome, Progress, Receipt, Snapshot, StepState,
    WorkflowEvent, WorkflowRequest,
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

/// Workflow orchestration failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EngineError {
    InvalidRequest,
    NotFound,
    IdempotencyConflict,
    VersionConflict,
    InvalidTransition,
    Unauthorized(String),
    Expired,
    ChallengeInvalid,
    ResourceLocked(String),
    Journal(JournalError),
}
impl From<JournalError> for EngineError {
    fn from(value: JournalError) -> Self {
        Self::Journal(value)
    }
}

/// Durable engine. Locks and idempotency indexes are rebuilt from the journal by `recover`.
pub struct WorkflowEngine<J, A> {
    journal: J,
    authorization: A,
    idempotency: Mutex<HashMap<String, (String, String)>>,
    locks: Mutex<HashMap<String, String>>,
}

impl<J: Journal, A: Authorization> WorkflowEngine<J, A> {
    /// Construct an engine around durable ports.
    pub fn new(journal: J, authorization: A) -> Self {
        Self {
            journal,
            authorization,
            idempotency: Mutex::new(HashMap::new()),
            locks: Mutex::new(HashMap::new()),
        }
    }

    /// Creates once. An exact retry returns the existing projection.
    pub fn create(&self, request: WorkflowRequest, now_ms: u64) -> Result<Snapshot, EngineError> {
        validate_request(&request, now_ms)?;
        let digest = request_digest(&request)?;
        let mut keys = self
            .idempotency
            .lock()
            .map_err(|_| EngineError::InvalidRequest)?;
        if let Some((existing_digest, id)) = keys.get(&request.idempotency_key) {
            if existing_digest != &digest {
                return Err(EngineError::IdempotencyConflict);
            }
            return self.snapshot(id);
        }
        match self.authorization.reauthorize(&request, now_ms) {
            AuthorizationDecision::Allow => {}
            AuthorizationDecision::Pause(reason) | AuthorizationDecision::Preempt(reason) => {
                return Err(EngineError::Unauthorized(reason));
            }
        }
        let id = Uuid::now_v7().to_string();
        if let Some(class) = request.workflow_type.lock_class() {
            let key = format!("{class}:{}", request.scope);
            let mut locks = self.locks.lock().map_err(|_| EngineError::InvalidRequest)?;
            if let Some(owner) = locks.get(&key) {
                return Err(EngineError::ResourceLocked(owner.clone()));
            }
            locks.insert(key, id.clone());
        }
        let mut events = vec![WorkflowEvent::Created {
            request: request.clone(),
            request_digest: digest.clone(),
        }];
        if request.workflow_type.destructive() {
            events.push(WorkflowEvent::ChallengeIssued(Challenge {
                nonce: Uuid::now_v7().to_string(),
                workflow_id: id.clone(),
                scope: request.scope.clone(),
                consequence: format!(
                    "authorize irreversible {:?} operation",
                    request.workflow_type
                ),
                state_version: 1,
                expires_at_ms: now_ms.saturating_add(300_000),
                requirement: ChallengeRequirement::TypedScope,
            }));
        } else {
            events.push(WorkflowEvent::StateChanged {
                state: StepState::Ready,
                progress: Progress::Percent(0),
            });
        }
        self.journal.append(&id, 0, &events)?;
        keys.insert(request.idempotency_key.clone(), (digest, id.clone()));
        self.snapshot(&id)
    }

    /// Rebuild and integrity-check a projection from its event stream.
    pub fn snapshot(&self, id: &str) -> Result<Snapshot, EngineError> {
        let events = self.journal.load(id)?;
        if events.is_empty() {
            return Err(EngineError::NotFound);
        }
        project(id, &events)
    }

    /// Rebuild volatile idempotency and lock indexes after process restart.
    pub fn recover(&self, ids: &[&str]) -> Result<Vec<Snapshot>, EngineError> {
        let mut projections = Vec::with_capacity(ids.len());
        let mut keys = self
            .idempotency
            .lock()
            .map_err(|_| EngineError::InvalidRequest)?;
        let mut locks = self.locks.lock().map_err(|_| EngineError::InvalidRequest)?;
        for id in ids {
            let events = self.journal.load(id)?;
            if events.is_empty() {
                continue;
            }
            let snapshot = project(id, &events)?;
            let digest = request_digest(&snapshot.request)?;
            if let Some((existing, _)) = keys.insert(
                snapshot.request.idempotency_key.clone(),
                (digest.clone(), snapshot.id.clone()),
            ) {
                if existing != digest {
                    return Err(EngineError::IdempotencyConflict);
                }
            }
            if snapshot.receipt.is_none() {
                if let Some(class) = snapshot.request.workflow_type.lock_class() {
                    let key = format!("{class}:{}", snapshot.request.scope);
                    if let Some(owner) = locks.insert(key, snapshot.id.clone()) {
                        if owner != snapshot.id {
                            return Err(EngineError::ResourceLocked(owner));
                        }
                    }
                }
            }
            projections.push(snapshot);
        }
        Ok(projections)
    }

    /// Prepare an external effect without committing it. Restart leaves an observable token.
    pub fn prepare_effect<E: EffectPort>(
        &self,
        id: &str,
        expected_version: u64,
        effects: &E,
        now_ms: u64,
    ) -> Result<Snapshot, EngineError> {
        let snapshot = self.snapshot(id)?;
        if snapshot.version != expected_version {
            return Err(EngineError::VersionConflict);
        }
        if snapshot.state != StepState::Running || snapshot.prepared_token.is_some() {
            return Err(EngineError::InvalidTransition);
        }
        match self.authorization.reauthorize(&snapshot.request, now_ms) {
            AuthorizationDecision::Allow => {}
            AuthorizationDecision::Pause(reason) | AuthorizationDecision::Preempt(reason) => {
                return Err(EngineError::Unauthorized(reason));
            }
        }
        let prepared = effects
            .prepare(&snapshot)
            .map_err(|error| EngineError::Unauthorized(error.0))?;
        self.append(
            id,
            &snapshot,
            vec![WorkflowEvent::EffectPrepared {
                token: prepared.token,
            }],
        )
    }

    /// Applies a transition with optimistic concurrency and fresh authorization.
    pub fn execute(
        &self,
        id: &str,
        expected_version: u64,
        command: Command,
        now_ms: u64,
    ) -> Result<Snapshot, EngineError> {
        let snapshot = self.snapshot(id)?;
        if snapshot.version != expected_version {
            return Err(EngineError::VersionConflict);
        }
        if now_ms >= snapshot.request.expires_at_ms && snapshot.receipt.is_none() {
            return Err(EngineError::Expired);
        }
        let decision = self.authorization.reauthorize(&snapshot.request, now_ms);
        if let AuthorizationDecision::Preempt(reason) = decision {
            return self.append(
                id,
                &snapshot,
                vec![
                    WorkflowEvent::Preempted {
                        reason: reason.clone(),
                    },
                    terminal_receipt(&snapshot, "preempted", vec![reason], false),
                ],
            );
        }
        if let Command::Preempt { reason } = command {
            return self.append(
                id,
                &snapshot,
                vec![
                    WorkflowEvent::Preempted {
                        reason: reason.clone(),
                    },
                    terminal_receipt(&snapshot, "preempted", vec![reason], false),
                ],
            );
        }
        if let AuthorizationDecision::Pause(reason) = decision {
            return self.append(id, &snapshot, vec![WorkflowEvent::Blocked { reason }]);
        }
        let events = transition(&snapshot, command, now_ms)?;
        self.append(id, &snapshot, events)
    }

    fn append(
        &self,
        id: &str,
        old: &Snapshot,
        events: Vec<WorkflowEvent>,
    ) -> Result<Snapshot, EngineError> {
        self.journal.append(id, old.version, &events)?;
        let result = self.snapshot(id)?;
        if result.receipt.is_some() {
            self.release_lock(&result);
        }
        Ok(result)
    }
    fn release_lock(&self, snapshot: &Snapshot) {
        if let Some(class) = snapshot.request.workflow_type.lock_class() {
            let key = format!("{class}:{}", snapshot.request.scope);
            if let Ok(mut locks) = self.locks.lock() {
                if locks.get(&key).is_some_and(|owner| owner == &snapshot.id) {
                    locks.remove(&key);
                }
            }
        }
    }
}

fn validate_request(request: &WorkflowRequest, now: u64) -> Result<(), EngineError> {
    if request.scope.trim().is_empty()
        || request.requester_session.trim().is_empty()
        || request.authorization_ref.trim().is_empty()
        || request.purpose_ref.trim().is_empty()
        || request.idempotency_key.trim().is_empty()
        || request.expires_at_ms <= now
    {
        return Err(EngineError::InvalidRequest);
    }
    Ok(())
}
fn request_digest(request: &WorkflowRequest) -> Result<String, EngineError> {
    let bytes = serde_json::to_vec(request).map_err(|_| EngineError::InvalidRequest)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn transition(s: &Snapshot, command: Command, now: u64) -> Result<Vec<WorkflowEvent>, EngineError> {
    if s.receipt.is_some() {
        return Err(EngineError::InvalidTransition);
    }
    match (s.state.clone(), command) {
        (StepState::AwaitingHuman, Command::Apply(Action::Confirm { nonce, typed_scope })) => {
            let challenge = s.challenge.as_ref().ok_or(EngineError::ChallengeInvalid)?;
            if challenge.nonce != nonce
                || challenge.scope != typed_scope
                || challenge.expires_at_ms <= now
                || challenge.state_version + 1 != s.version
            {
                return Err(EngineError::ChallengeInvalid);
            }
            Ok(vec![
                WorkflowEvent::Confirmed,
                WorkflowEvent::StateChanged {
                    state: StepState::Ready,
                    progress: Progress::Percent(0),
                },
            ])
        }
        (StepState::Ready, Command::Apply(Action::Start)) => {
            Ok(vec![WorkflowEvent::StateChanged {
                state: StepState::Running,
                progress: Progress::Indeterminate,
            }])
        }
        (StepState::Running, Command::Apply(Action::Complete)) => Ok(vec![
            WorkflowEvent::StateChanged {
                state: StepState::Succeeded,
                progress: Progress::Percent(100),
            },
            terminal_receipt(s, "succeeded", vec![], false),
        ]),
        (StepState::Running, Command::CommitEffect(outcome)) => Ok(vec![
            WorkflowEvent::EffectCommitted(outcome.clone()),
            WorkflowEvent::StateChanged {
                state: StepState::Succeeded,
                progress: Progress::Percent(100),
            },
            receipt_for(s, outcome, false),
        ]),
        (StepState::Running, Command::Apply(Action::Fail { reason })) => Ok(vec![
            WorkflowEvent::Blocked { reason },
            WorkflowEvent::StateChanged {
                state: StepState::RetryWait,
                progress: Progress::Indeterminate,
            },
        ]),
        (StepState::RetryWait, Command::Apply(Action::Retry)) => {
            Ok(vec![WorkflowEvent::StateChanged {
                state: StepState::Ready,
                progress: Progress::Indeterminate,
            }])
        }
        (_, Command::Apply(Action::AddEvidence(e))) => Ok(vec![WorkflowEvent::EvidenceAdded(e)]),
        (state, Command::Apply(Action::Cancel))
            if s.request.workflow_type.cancellable()
                && matches!(
                    state,
                    StepState::Pending
                        | StepState::Ready
                        | StepState::AwaitingHuman
                        | StepState::AwaitingDevice
                        | StepState::RetryWait
                        | StepState::Blocked
                ) =>
        {
            Ok(vec![
                WorkflowEvent::StateChanged {
                    state: StepState::Cancelled,
                    progress: s.progress.clone(),
                },
                terminal_receipt(s, "cancelled", vec![], false),
            ])
        }
        (StepState::RetryWait | StepState::Blocked, Command::Apply(Action::Compensate)) => {
            Ok(vec![
                WorkflowEvent::StateChanged {
                    state: StepState::Compensating,
                    progress: Progress::Indeterminate,
                },
                WorkflowEvent::CompensationCompleted,
                WorkflowEvent::StateChanged {
                    state: StepState::Failed,
                    progress: Progress::Indeterminate,
                },
                terminal_receipt(
                    s,
                    "failed",
                    vec!["external effect compensated".into()],
                    true,
                ),
            ])
        }
        _ => Err(EngineError::InvalidTransition),
    }
}
fn terminal_receipt(
    s: &Snapshot,
    outcome: &str,
    warnings: Vec<String>,
    compensated: bool,
) -> WorkflowEvent {
    WorkflowEvent::ReceiptIssued(Receipt {
        workflow_id: s.id.clone(),
        workflow_type: s.request.workflow_type,
        outcome: outcome.into(),
        affected_resources: vec![],
        before_ref: None,
        after_ref: None,
        evidence_digests: s.evidence.iter().map(|e| e.digest.clone()).collect(),
        audit_digest: digest_text(&format!("{}:{outcome}:{}", s.id, s.version)),
        warnings,
        recovery_guidance: (outcome != "succeeded")
            .then(|| "inspect the audit trail before retrying".into()),
        compensated,
    })
}
fn receipt_for(s: &Snapshot, o: Outcome, compensated: bool) -> WorkflowEvent {
    WorkflowEvent::ReceiptIssued(Receipt {
        workflow_id: s.id.clone(),
        workflow_type: s.request.workflow_type,
        outcome: "succeeded".into(),
        affected_resources: o.affected_resources,
        before_ref: o.before_ref,
        after_ref: o.after_ref,
        evidence_digests: s.evidence.iter().map(|e| e.digest.clone()).collect(),
        audit_digest: o.audit_digest,
        warnings: o.warnings,
        recovery_guidance: None,
        compensated,
    })
}
fn digest_text(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn project(id: &str, stored: &[crate::StoredEvent]) -> Result<Snapshot, EngineError> {
    let (request, _) = match &stored[0].event {
        WorkflowEvent::Created {
            request,
            request_digest,
        } => (request.clone(), request_digest),
        _ => return Err(EngineError::Journal(JournalError::Corrupt)),
    };
    let mut s = Snapshot {
        id: id.into(),
        schema_version: 1,
        version: 0,
        request,
        state: StepState::Pending,
        progress: Progress::Indeterminate,
        permitted_actions: vec![],
        blocking_reasons: vec![],
        evidence: vec![],
        challenge: None,
        prepared_token: None,
        receipt: None,
    };
    for envelope in stored {
        s.version = envelope.sequence;
        match &envelope.event {
            WorkflowEvent::Created { .. } => {}
            WorkflowEvent::ChallengeIssued(c) => {
                s.challenge = Some(c.clone());
                s.state = StepState::AwaitingHuman;
            }
            WorkflowEvent::Confirmed => s.challenge = None,
            WorkflowEvent::StateChanged { state, progress } => {
                s.state = state.clone();
                s.progress = progress.clone();
            }
            WorkflowEvent::EvidenceAdded(e) => s.evidence.push(e.clone()),
            WorkflowEvent::EffectPrepared { token } => s.prepared_token = Some(token.clone()),
            WorkflowEvent::EffectCommitted(_) => s.prepared_token = None,
            WorkflowEvent::Blocked { reason } => {
                s.state = StepState::Blocked;
                s.blocking_reasons.push(reason.clone());
            }
            WorkflowEvent::Preempted { reason } => {
                s.state = StepState::Blocked;
                s.blocking_reasons.push(reason.clone());
            }
            WorkflowEvent::CompensationCompleted => {}
            WorkflowEvent::ReceiptIssued(r) => s.receipt = Some(r.clone()),
        }
    }
    s.permitted_actions = permitted(&s);
    Ok(s)
}
fn permitted(s: &Snapshot) -> Vec<String> {
    if s.receipt.is_some() {
        return vec![];
    }
    match s.state {
        StepState::AwaitingHuman => vec!["confirm"],
        StepState::Ready => vec!["start", "cancel"],
        StepState::Running => vec!["complete", "fail"],
        StepState::RetryWait => vec!["retry", "cancel", "compensate"],
        StepState::Blocked => vec!["compensate"],
        _ => vec![],
    }
    .into_iter()
    .map(str::to_owned)
    .collect()
}
