//! Opaque identifier types and generation port.

use core::fmt;
use core::num::NonZeroU128;

/// Returned when an identifier is constructed from the reserved zero value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdError;

impl fmt::Display for IdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an identifier must be nonzero")
    }
}

impl std::error::Error for IdError {}

macro_rules! opaque_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU128);

        impl $name {
            /// Constructs the identifier, rejecting the reserved zero value.
            ///
            /// # Errors
            ///
            /// Returns [`IdError`] when `value` is zero.
            pub fn from_u128(value: u128) -> Result<Self, IdError> {
                NonZeroU128::new(value).map(Self).ok_or(IdError)
            }

            /// Returns the stable 128-bit representation.
            #[must_use]
            pub const fn as_u128(self) -> u128 {
                self.0.get()
            }
        }
    };
}

opaque_id!(EventId, "Opaque identity of a domain or integration event.");
opaque_id!(AggregateId, "Opaque identity of an aggregate root.");
opaque_id!(
    CorrelationId,
    "Identity shared by events in one logical operation."
);
opaque_id!(
    CausationId,
    "Identity of the event or command that caused an event."
);

/// Domain port for generating identifiers.
///
/// Infrastructure implementations are responsible for `UUIDv7` compatibility;
/// deterministic tests can provide sequential values.
pub trait IdGenerator<Id = EventId> {
    /// Produces the next unique, nonzero identifier.
    fn next_id(&mut self) -> Id;
}
