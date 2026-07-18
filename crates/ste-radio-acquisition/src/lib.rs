//! Radio acquisition bounded context.
//!
//! Domain code is dependency-inward; infrastructure is intentionally private.

pub mod application;
pub mod domain;
mod infrastructure;
/// Pinned, policy-gated live rvCSI process adapter.
pub mod live_adapter;
/// Deterministic replay format and source adapter.
pub mod replay;

pub use application::{
    AcquisitionService, CaptureAuthorizationPort, CsiCaptureSource, FrameJournal, FramePublisher,
    PublicationOutcome, SessionRepository,
};
pub use domain::{
    AcquisitionError, CalibrationMetadata, CaptureCommand, CaptureEvent, CaptureHealth,
    CaptureLink, CaptureProfile, CaptureSession, CsiFrameInput, FrameSequence, HardwareProvenance,
    ValidatedFrame,
};
pub use infrastructure::{
    InMemoryFrameJournal, InMemorySessionRepository, ReplayCaptureSource,
    RvCsiAntiCorruptionAdapter,
};

/// Stable name used in diagnostics and architecture tests.
pub const CONTEXT_NAME: &str = "radio-acquisition";
