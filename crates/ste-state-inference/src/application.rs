//! State-assessment persistence and external policy query ports.
use crate::domain::{AssessmentOutcome, DomainBoundary};

/// Returns a domain marker without exposing infrastructure.
#[must_use]
pub const fn boundary() -> DomainBoundary {
    DomainBoundary
}

/// Append-only assessment persistence.
pub trait StateAssessmentRepository {
    /// Persistence failure.
    type Error;
    /// Appends an immutable estimate or abstention.
    fn append(
        &mut self,
        assessment_id: &str,
        outcome: &AssessmentOutcome,
    ) -> Result<(), Self::Error>;
}

/// Read-only construct-promotion query owned by Experiment Validation.
pub trait ValidationRegistry {
    /// Query failure.
    type Error;
    /// Exact construct promotion status.
    fn construct_is_promoted(&self, construct_reference: &str) -> Result<bool, Self::Error>;
}

/// Read-only active-model query.
pub trait ModelRegistry {
    /// Query failure.
    type Error;
    /// Whether an exact verified package is active.
    fn model_is_active(&self, model_id: &str) -> Result<bool, Self::Error>;
}

/// Read-only participant-profile scope query.
pub trait PatternProfileReader {
    /// Query failure.
    type Error;
    /// Whether a profile is valid for a participant.
    fn profile_is_valid(
        &self,
        participant: &str,
        profile_version: &str,
    ) -> Result<bool, Self::Error>;
}
