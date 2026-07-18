use crate::WorkflowType;
use serde::{Deserialize, Serialize};

/// Coarse authoritative role; concrete policy remains an authorization port concern.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Viewer,
    Operator,
    Researcher,
    Administrator,
    SafetyOfficer,
}

/// Creation request whose canonical digest enforces idempotency conflicts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowRequest {
    pub workflow_type: WorkflowType,
    pub scope: String,
    pub requester_role: Role,
    pub requester_session: String,
    pub authorization_ref: String,
    pub purpose_ref: String,
    pub idempotency_key: String,
    pub correlation_id: Option<String>,
    pub expires_at_ms: u64,
}

/// Semantic progress; unknown duration is never represented as a fake percentage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Progress {
    Indeterminate,
    Percent(u8),
}

/// Workflow step state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    Pending,
    Ready,
    Running,
    AwaitingHuman,
    AwaitingDevice,
    RetryWait,
    Compensating,
    Blocked,
    Cancelled,
    Succeeded,
    Failed,
}

/// Evidence is referenced, never embedded in receipts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub kind: String,
    pub digest: String,
}

/// Server-bound confirmation requirement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeRequirement {
    TypedScope,
    StepUp,
    NamedApproval,
    DualControl,
    PhysicalAction,
}

/// Short-lived server challenge bound to exact projection state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Challenge {
    pub nonce: String,
    pub workflow_id: String,
    pub scope: String,
    pub consequence: String,
    pub state_version: u64,
    pub expires_at_ms: u64,
    pub requirement: ChallengeRequirement,
}

/// Actions accepted by the state machine.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Confirm { nonce: String, typed_scope: String },
    Start,
    Complete,
    Fail { reason: String },
    Retry,
    Cancel,
    Compensate,
    AddEvidence(EvidenceRef),
}

/// Effect outcome supplied only after an idempotent application command returns.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    pub affected_resources: Vec<String>,
    pub before_ref: Option<String>,
    pub after_ref: Option<String>,
    pub audit_digest: String,
    pub warnings: Vec<String>,
}

/// Terminal, secret-free result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub workflow_id: String,
    pub workflow_type: WorkflowType,
    pub outcome: String,
    pub affected_resources: Vec<String>,
    pub before_ref: Option<String>,
    pub after_ref: Option<String>,
    pub evidence_digests: Vec<String>,
    pub audit_digest: String,
    pub warnings: Vec<String>,
    pub recovery_guidance: Option<String>,
    pub compensated: bool,
}

/// Durable commands. `Preempt` is reserved for revocation and safety paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Apply(Action),
    CommitEffect(Outcome),
    Preempt { reason: String },
}

/// Events are versioned and append-only.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum WorkflowEvent {
    Created {
        request: WorkflowRequest,
        request_digest: String,
    },
    ChallengeIssued(Challenge),
    Confirmed,
    StateChanged {
        state: StepState,
        progress: Progress,
    },
    EvidenceAdded(EvidenceRef),
    EffectPrepared {
        token: String,
    },
    EffectCommitted(Outcome),
    Blocked {
        reason: String,
    },
    Preempted {
        reason: String,
    },
    CompensationCompleted,
    ReceiptIssued(Receipt),
}

/// Materialized projection rebuilt solely from verified events.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub schema_version: u16,
    pub version: u64,
    pub request: WorkflowRequest,
    pub state: StepState,
    pub progress: Progress,
    pub permitted_actions: Vec<String>,
    pub blocking_reasons: Vec<String>,
    pub evidence: Vec<EvidenceRef>,
    pub challenge: Option<Challenge>,
    pub prepared_token: Option<String>,
    pub receipt: Option<Receipt>,
}
