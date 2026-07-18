//! Signal observation bounded context.
//!
//! Domain code is dependency-inward; infrastructure is intentionally private.

pub mod application;
pub mod domain;
/// Deterministic signal-processing graph and golden-vector primitives.
pub mod dsp;
mod infrastructure;
pub use infrastructure::{
    ContentAddressedEvidenceRepository, ContentAddressedStore, ObservationReplay, PutOutcome,
    ReplayEvidenceFrame, ReplayRepositoryError, RepositoryError,
};

pub use application::{FeatureArtifactStore, ObservationWindowRepository};
pub use domain::{
    AbstentionReason, AlgorithmVersion, BaselineDrift, DspVersion, FeatureEvidenceArtifact,
    FrameEvidence, MotionEnergy, ObservationCommand, ObservationError, ObservationEvent,
    ObservationWindow, ObservationWindowId, PUBLIC_OBSERVATION_TYPES, PartitionRole,
    PeriodicityCandidate, PresenceScore, QualityAssessment, QualityDisposition, WindowBounds,
    WindowPolicy,
};

/// Stable name used in diagnostics and architecture tests.
pub const CONTEXT_NAME: &str = "signal-observation";
