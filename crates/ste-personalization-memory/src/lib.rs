//! Personalization memory bounded context.
//!
//! Domain code is dependency-inward; infrastructure is intentionally private.

pub mod adaptation;
pub mod application;
pub mod domain;
mod infrastructure;

pub use infrastructure::{
    AppendOnlyPatternRepository, ReferenceVectorMatch, ReferenceVectorMemory, VectorAdapterError,
};
pub mod retrieval;

/// Stable name used in diagnostics and architecture tests.
pub const CONTEXT_NAME: &str = "personalization-memory";
