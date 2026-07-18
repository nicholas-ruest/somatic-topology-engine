//! Recoverable session snapshots, append-only audit, and simulator ports.
use crate::{
    application::{DisplayPort, InteractionAuditJournal, InteractionSessionRepository, LedPort},
    domain::{InteractionEvent, InteractionSession, RenderedProjection, RgbColor},
};
use std::{collections::BTreeMap, error::Error, fmt};
/// Recoverable latest snapshots; audit remains the authority for history.
#[derive(Default)]
pub struct SnapshotSessionRepository {
    sessions: BTreeMap<String, InteractionSession>,
}
impl SnapshotSessionRepository {
    /// Latest snapshot.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&InteractionSession> {
        self.sessions.get(id)
    }
}
impl InteractionSessionRepository for SnapshotSessionRepository {
    type Error = InteractionAdapterError;
    fn save(&mut self, session: &InteractionSession) -> Result<(), Self::Error> {
        if self.sessions.get(session.id()) == Some(session) {
            return Ok(());
        }
        self.sessions.insert(session.id().into(), session.clone());
        Ok(())
    }
}
/// Immutable event journal.
#[derive(Default)]
pub struct AppendOnlyInteractionAudit {
    events: Vec<InteractionEvent>,
}
impl AppendOnlyInteractionAudit {
    /// Full ordered audit evidence.
    #[must_use]
    pub fn events(&self) -> &[InteractionEvent] {
        &self.events
    }
}
impl InteractionAuditJournal for AppendOnlyInteractionAudit {
    type Error = InteractionAdapterError;
    fn append(&mut self, event: &InteractionEvent) -> Result<(), Self::Error> {
        self.events.push(event.clone());
        Ok(())
    }
}
/// Deterministic OLED simulator retaining accessible snapshots.
#[derive(Default)]
pub struct SimulatorDisplay {
    snapshots: Vec<(String, RgbColor)>,
    fail_next: bool,
}
impl SimulatorDisplay {
    /// Injects one recoverable peripheral failure.
    pub fn fail_next(&mut self) {
        self.fail_next = true
    }
    /// Rendered snapshots.
    #[must_use]
    pub fn snapshots(&self) -> &[(String, RgbColor)] {
        &self.snapshots
    }
}
impl DisplayPort for SimulatorDisplay {
    type Error = InteractionAdapterError;
    fn display(&mut self, p: &RenderedProjection) -> Result<(), Self::Error> {
        if self.fail_next {
            self.fail_next = false;
            return Err(InteractionAdapterError::PeripheralFault);
        }
        self.snapshots.push((p.text.into(), p.color));
        Ok(())
    }
}
/// Deterministic RGB simulator.
#[derive(Default)]
pub struct SimulatorLed {
    colors: Vec<RgbColor>,
    fail_next: bool,
}
impl SimulatorLed {
    /// Injects one recoverable peripheral failure.
    pub fn fail_next(&mut self) {
        self.fail_next = true
    }
    /// Ordered palette snapshots.
    #[must_use]
    pub fn colors(&self) -> &[RgbColor] {
        &self.colors
    }
}
impl LedPort for SimulatorLed {
    type Error = InteractionAdapterError;
    fn set_color(&mut self, color: RgbColor) -> Result<(), Self::Error> {
        if self.fail_next {
            self.fail_next = false;
            return Err(InteractionAdapterError::PeripheralFault);
        }
        self.colors.push(color);
        Ok(())
    }
}
/// Unqualified physical metadata; construction never opens GPIO/I2C hardware.
#[derive(Default)]
pub struct CrowPiPhysicalProfile {
    /// Exact revision, absent until probed/qualified.
    pub revision: Option<String>,
    /// HIL qualification evidence digest, absent until run.
    pub hil_evidence: Option<[u8; 32]>,
}
impl CrowPiPhysicalProfile {
    /// Only exact revision plus HIL evidence is qualified.
    #[must_use]
    pub fn is_qualified(&self) -> bool {
        self.revision.as_ref().is_some_and(|v| !v.trim().is_empty()) && self.hil_evidence.is_some()
    }
}
/// Payload-free adapter failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractionAdapterError {
    /// Isolated peripheral operation failed.
    PeripheralFault,
}
impl fmt::Display for InteractionAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("isolated interaction peripheral fault")
    }
}
impl Error for InteractionAdapterError {}
