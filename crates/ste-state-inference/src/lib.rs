//! State inference bounded context.
//!
//! Domain code is dependency-inward; infrastructure is intentionally private.

pub mod application;
pub mod domain;
mod infrastructure;
pub mod temporal;
pub mod validation;

pub use infrastructure::{AtomicStateRepository, SafeDisplayProjection, StateRepositoryError};

/// Stable name used in diagnostics and architecture tests.
pub const CONTEXT_NAME: &str = "state-inference";
