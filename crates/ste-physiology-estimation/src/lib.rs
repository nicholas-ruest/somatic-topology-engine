//! Physiology estimation bounded context.
//!
//! Domain code is dependency-inward; infrastructure is intentionally private.

pub mod application;
pub mod domain;
pub mod estimator;
pub mod evaluation;
mod infrastructure;

pub use infrastructure::{
    AtomicPhysiologyRepository, ExperimentValidationRegistry, PhysiologyRepositoryError,
};

/// Stable name used in diagnostics and architecture tests.
pub const CONTEXT_NAME: &str = "physiology-estimation";
