//! Observation persistence and artifact application ports.
use crate::domain::{FeatureEvidenceArtifact, ObservationError, ObservationWindow};
/// Closed-window repository.
pub trait ObservationWindowRepository {
    /// Saves closed metadata.
    fn save(&self, window: &ObservationWindow) -> Result<(), ObservationError>;
}
/// Immutable artifact store.
pub trait FeatureArtifactStore {
    /// Stores by content digest idempotently.
    fn put(&self, artifact: &FeatureEvidenceArtifact) -> Result<(), ObservationError>;
}
use crate::domain::DomainBoundary;
/// Returns the context boundary marker.
#[must_use]
pub const fn boundary() -> DomainBoundary {
    DomainBoundary
}
