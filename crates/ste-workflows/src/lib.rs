//! Durable finite-state workflows for privileged STE operations.
#![forbid(unsafe_code)]
#![allow(missing_docs)]

mod catalog;
mod engine;
mod journal;
mod model;
mod ports;

pub use catalog::WorkflowType;
pub use engine::{EngineError, WorkflowEngine};
pub use journal::{InMemoryJournal, Journal, JournalError, StoredEvent};
pub use model::{
    Action, Challenge, ChallengeRequirement, Command, EvidenceRef, Outcome, Progress, Receipt,
    Role, Snapshot, StepState, WorkflowEvent, WorkflowRequest,
};
pub use ports::{Authorization, AuthorizationDecision, EffectError, EffectPort, PreparedEffect};
