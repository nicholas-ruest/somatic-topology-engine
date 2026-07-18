//! Experiment validation bounded context.
//!
//! Domain code is dependency-inward; infrastructure is intentionally private.

pub mod application;
pub mod domain;
mod infrastructure;
pub use infrastructure::{
    AtomicValidationRepository, DeidentifiedEvidenceExport, RepositoryError,
    ReproducibleValidationReport,
};
pub mod metrics;
pub mod reference;

/// Stable name used in diagnostics and architecture tests.
pub const CONTEXT_NAME: &str = "experiment-validation";
