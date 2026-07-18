//! Device interaction bounded context.
//!
//! Domain code is dependency-inward; infrastructure is intentionally private.

pub mod application;
pub mod domain;
pub mod hardware;
mod infrastructure;

pub use infrastructure::{
    AppendOnlyInteractionAudit, CrowPiPhysicalProfile, InteractionAdapterError, SimulatorDisplay,
    SimulatorLed, SnapshotSessionRepository,
};

/// Stable name used in diagnostics and architecture tests.
pub const CONTEXT_NAME: &str = "device-interaction";
