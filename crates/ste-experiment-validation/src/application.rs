//! Application ports for validation persistence and evidence export.
use crate::domain::{DomainBoundary, FrozenStudy, PromotionDecision, StudyRun, ValidationStudy};

/// Returns a boundary marker without exposing infrastructure.
#[must_use]
pub const fn boundary() -> DomainBoundary {
    DomainBoundary
}

/// Persistence boundary for aggregates and immutable runs.
pub trait ValidationStudyRepository {
    /// Repository-specific failure.
    type Error;
    /// Stores an editable draft.
    fn save_draft(&mut self, study: &ValidationStudy) -> Result<(), Self::Error>;
    /// Stores an immutable definition.
    fn save_frozen(&mut self, study: &FrozenStudy) -> Result<(), Self::Error>;
    /// Appends a run, rejecting replacement of completed results.
    fn append_run(&mut self, run: &StudyRun) -> Result<(), Self::Error>;
    /// Appends a promotion decision.
    fn append_promotion(&mut self, decision: &PromotionDecision) -> Result<(), Self::Error>;
}

/// Read-only boundary for de-identified immutable evidence.
pub trait EvidenceExportReader {
    /// Reader-specific failure.
    type Error;
    /// Signed, manifest-driven export representation.
    type Export;
    /// Reads evidence without command access to aggregates.
    fn deidentified_export(&self, study_id: &str) -> Result<Self::Export, Self::Error>;
}
