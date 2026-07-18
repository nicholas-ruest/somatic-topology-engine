//! Public commands, queries, and ports exposed by this context.

use crate::domain::DomainBoundary;

/// Returns a domain marker without exposing infrastructure implementation.
#[must_use]
pub const fn boundary() -> DomainBoundary {
    DomainBoundary
}
