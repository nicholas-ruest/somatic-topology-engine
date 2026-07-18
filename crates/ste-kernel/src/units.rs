//! Validated physical-unit newtypes.

use core::fmt;

/// Error returned for a non-finite or negative physical magnitude.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnitError;

impl fmt::Display for UnitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("physical magnitude must be finite and nonnegative")
    }
}

impl std::error::Error for UnitError {}

macro_rules! nonnegative_unit {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
        pub struct $name(f64);

        impl $name {
            /// Validates and constructs the physical magnitude.
            ///
            /// # Errors
            ///
            /// Returns [`UnitError`] when the value is negative or non-finite.
            pub fn new(value: f64) -> Result<Self, UnitError> {
                if value.is_finite() && value >= 0.0 {
                    Ok(Self(value))
                } else {
                    Err(UnitError)
                }
            }

            /// Returns the magnitude in the unit named by this type.
            #[must_use]
            pub const fn get(self) -> f64 {
                self.0
            }
        }
    };
}

nonnegative_unit!(Hertz, "A finite, nonnegative frequency in hertz.");
nonnegative_unit!(Meters, "A finite, nonnegative distance in metres.");

/// A finite temperature in degrees Celsius.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Celsius(f64);

impl Celsius {
    /// Constructs a finite temperature. Negative Celsius values are valid.
    ///
    /// # Errors
    ///
    /// Returns [`UnitError`] when the value is non-finite.
    pub fn new(value: f64) -> Result<Self, UnitError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(UnitError)
        }
    }

    /// Returns degrees Celsius.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}
