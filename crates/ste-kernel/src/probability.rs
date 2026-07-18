//! Validated probability semantics.

use core::fmt;

/// Error returned for a non-finite or out-of-range probability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProbabilityError;

impl fmt::Display for ProbabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("probability must be finite and in the inclusive range [0, 1]")
    }
}

impl std::error::Error for ProbabilityError {}

/// A finite IEEE-754 probability in the inclusive range `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct FiniteProbability(f64);

impl FiniteProbability {
    /// Validates and constructs a probability.
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError`] when the value is non-finite or outside
    /// the inclusive range `[0, 1]`.
    pub fn new(value: f64) -> Result<Self, ProbabilityError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ProbabilityError)
        }
    }

    /// Returns the finite probability value.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}
