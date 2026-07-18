//! Application ports for respiration estimation and validation policy.
use crate::domain::{AssessmentOutcome, DomainBoundary, ModelEstimate};

/// Returns a boundary marker without exposing infrastructure.
#[must_use]
pub const fn boundary() -> DomainBoundary {
    DomainBoundary
}

/// Experiment-validation query owned outside this bounded context.
pub trait ValidationRegistry {
    /// Query failure.
    type Error;
    /// Returns only immutable promotion state; no config override is accepted.
    fn respiration_is_promoted(&self, model_id: &str) -> Result<bool, Self::Error>;
}

/// Rust inference boundary for deterministic respiration models.
pub trait PhysiologyModel {
    /// Model failure.
    type Error;
    /// Estimates respiration from finite ordered features.
    fn estimate_respiration(&self, features: &[f64]) -> Result<ModelEstimate, Self::Error>;
}

/// Append-only assessment repository.
pub trait PhysiologyAssessmentRepository {
    /// Persistence failure.
    type Error;
    /// Appends an immutable estimate or abstention.
    fn append(
        &mut self,
        assessment_id: &str,
        outcome: &AssessmentOutcome,
    ) -> Result<(), Self::Error>;
}
