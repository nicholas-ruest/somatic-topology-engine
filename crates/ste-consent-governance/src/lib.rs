//! Consent and governance bounded context.
//!
//! Domain code is dependency-inward; infrastructure is intentionally private.

pub mod application;
pub mod domain;
mod infrastructure;

/// Stable name used in diagnostics and architecture tests.
pub const CONTEXT_NAME: &str = "consent-governance";
