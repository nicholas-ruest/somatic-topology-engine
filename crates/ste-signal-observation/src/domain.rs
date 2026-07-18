//! Aggregate roots, value objects, domain events, and repository ports.

/// Marker for domain types owned exclusively by this bounded context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainBoundary;
