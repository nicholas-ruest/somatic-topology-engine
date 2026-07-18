//! Domain event behavior shared without transport concerns.

use crate::EventId;

/// Behavior common to domain events.
///
/// Serialization and integration metadata deliberately belong in
/// `ste-contracts`, not this domain-facing trait.
pub trait DomainEvent: Send + Sync {
    /// Stable identity of this event occurrence.
    fn event_id(&self) -> EventId;
    /// Stable, past-tense event type name.
    fn event_name(&self) -> &'static str;
}
