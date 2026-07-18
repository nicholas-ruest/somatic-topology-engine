use crate::{Snapshot, WorkflowRequest};

/// Result of a fresh authorization check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorizationDecision {
    Allow,
    Pause(String),
    Preempt(String),
}

/// Rechecked before every delayed or privileged transition.
pub trait Authorization {
    fn reauthorize(&self, request: &WorkflowRequest, now_ms: u64) -> AuthorizationDecision;
}

/// Opaque prepared external effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedEffect {
    pub token: String,
}

/// External effects are ports; implementations must prepare/commit or be idempotent.
pub trait EffectPort {
    fn prepare(&self, snapshot: &Snapshot) -> Result<PreparedEffect, EffectError>;
    fn compensate(&self, snapshot: &Snapshot) -> Result<(), EffectError>;
}

/// Sanitized effect failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectError(pub String);
