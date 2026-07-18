//! Policy-gated acquisition application service and adapter ports.

use std::sync::Arc;

use crate::domain::{AcquisitionError, CaptureSession, CsiFrameInput, ValidatedFrame};

/// Current local consent-policy decision port.
pub trait CaptureAuthorizationPort: Send + Sync {
    /// Checks the exact session immediately before publication.
    fn authorize_capture(&self, session: &CaptureSession) -> bool;
}
/// Durable session repository port.
pub trait SessionRepository: Send + Sync {
    /// Saves aggregate state.
    fn save(&self, session: &CaptureSession) -> Result<(), AcquisitionError>;
}
/// Accepted-frame journal port.
pub trait FrameJournal: Send + Sync {
    /// Appends an accepted frame before publication.
    fn append(&self, frame: &ValidatedFrame) -> Result<(), AcquisitionError>;
}
/// Bounded frame publisher port.
pub trait FramePublisher: Send + Sync {
    /// Attempts bounded publication.
    fn publish(&self, frame: &ValidatedFrame) -> PublicationOutcome;
}
/// Replay/live source anti-corruption port.
pub trait CsiCaptureSource {
    /// Returns the next untrusted frame or end of stream.
    fn next_frame(&mut self) -> Result<Option<CsiFrameInput>, AcquisitionError>;
}

/// Explicit bounded publication result.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PublicationOutcome {
    /// Published within capacity.
    #[default]
    Published,
    /// Retained/journaled but bounded channel has no capacity.
    Backpressured,
}

/// Policy-gated application service.
pub struct AcquisitionService {
    authorization: Arc<dyn CaptureAuthorizationPort>,
    publisher: Arc<dyn FramePublisher>,
    journal: Arc<dyn FrameJournal>,
    repository: Arc<dyn SessionRepository>,
}

impl AcquisitionService {
    /// Composes required local ports.
    #[must_use]
    pub fn new(
        authorization: Arc<dyn CaptureAuthorizationPort>,
        publisher: Arc<dyn FramePublisher>,
        journal: Arc<dyn FrameJournal>,
        repository: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            authorization,
            publisher,
            journal,
            repository,
        }
    }

    /// Rechecks policy, validates, journals, publishes, and persists in that order.
    pub fn process(
        &mut self,
        session: &mut CaptureSession,
        input: CsiFrameInput,
    ) -> Result<PublicationOutcome, AcquisitionError> {
        if !self.authorization.authorize_capture(session) {
            return Err(AcquisitionError::Unauthorized);
        }
        let frame = session.accept(input)?;
        self.journal.append(&frame)?;
        let outcome = self.publisher.publish(&frame);
        if outcome == PublicationOutcome::Backpressured {
            session.record_backpressure();
        }
        self.repository.save(session)?;
        Ok(outcome)
    }
}

use crate::domain::DomainBoundary;
/// Returns a domain marker without exposing infrastructure implementation.
#[must_use]
pub const fn boundary() -> DomainBoundary {
    DomainBoundary
}
