//! Shared simulator/physical hardware ports and interaction persistence.
use crate::domain::{
    AnchorRequest, DomainBoundary, InteractionEvent, InteractionSession, RenderedProjection,
    RgbColor, TouchGesture,
};
/// Returns a boundary marker without exposing infrastructure.
#[must_use]
pub const fn boundary() -> DomainBoundary {
    DomainBoundary
}
/// Recoverable current-session store.
pub trait InteractionSessionRepository {
    /// Store failure.
    type Error;
    /// Saves current state.
    fn save(&mut self, session: &InteractionSession) -> Result<(), Self::Error>;
}
/// Append-only policy-permitted interaction journal.
pub trait InteractionAuditJournal {
    /// Journal failure.
    type Error;
    /// Appends one immutable event.
    fn append(&mut self, event: &InteractionEvent) -> Result<(), Self::Error>;
}
/// OLED adapter shared by simulator and physical hardware.
pub trait DisplayPort {
    /// Adapter failure.
    type Error;
    /// Displays text and redundant accessible color cue.
    fn display(&mut self, projection: &RenderedProjection) -> Result<(), Self::Error>;
}
/// RGB adapter restricted to already-approved colors from projections.
pub trait LedPort {
    /// Adapter failure.
    type Error;
    /// Sets a reviewed palette color.
    fn set_color(&mut self, color: RgbColor) -> Result<(), Self::Error>;
}
/// Physical touch adapter.
pub trait TouchPort {
    /// Adapter failure.
    type Error;
    /// Polls timestamped physical gesture evidence.
    fn poll(&mut self) -> Result<Option<TouchGesture>, Self::Error>;
}
/// Anchor command boundary to Personalization Memory.
pub trait AnchorRequestPort {
    /// Adapter failure.
    type Error;
    /// Sends an authorized request.
    fn request_anchor(&mut self, request: &AnchorRequest) -> Result<(), Self::Error>;
}
