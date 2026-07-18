//! Immutable evidence provenance.

use core::fmt;

/// Error returned when required provenance text is absent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceError;

impl fmt::Display for ProvenanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("provenance fields must be nonempty")
    }
}

impl std::error::Error for ProvenanceError {}

/// Content-addressed or otherwise immutable reference to source evidence.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProvenanceRef(String);

impl ProvenanceRef {
    /// Validates a provenance reference.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when the reference is blank.
    pub fn new(value: impl Into<String>) -> Result<Self, ProvenanceError> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(ProvenanceError)
        } else {
            Ok(Self(value))
        }
    }

    /// Borrows the reference representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Minimal immutable identity of an evidence-producing operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    reference: ProvenanceRef,
    producer: String,
    capture_profile: String,
}

impl Provenance {
    /// Validates and creates provenance. Producer and capture-profile identities
    /// must be nonempty.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when either identity is blank.
    pub fn new(
        reference: ProvenanceRef,
        producer: impl Into<String>,
        capture_profile: impl Into<String>,
    ) -> Result<Self, ProvenanceError> {
        let producer = producer.into();
        let capture_profile = capture_profile.into();
        if producer.trim().is_empty() || capture_profile.trim().is_empty() {
            Err(ProvenanceError)
        } else {
            Ok(Self {
                reference,
                producer,
                capture_profile,
            })
        }
    }

    /// Returns the immutable evidence reference.
    #[must_use]
    pub const fn reference(&self) -> &ProvenanceRef {
        &self.reference
    }
    /// Returns the producing component and version.
    #[must_use]
    pub fn producer(&self) -> &str {
        &self.producer
    }
    /// Returns the pinned capture-profile identity.
    #[must_use]
    pub fn capture_profile(&self) -> &str {
        &self.capture_profile
    }
}
