//! Explicit monotonic and UTC time semantics.

use core::fmt;
use std::time::Duration;

/// Error returned for an invalid timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeError {
    /// STE does not support UTC instants before the Unix epoch.
    BeforeUnixEpoch,
    /// The timestamp cannot be represented by the supported nanosecond range.
    OutOfRange,
}

impl fmt::Display for TimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BeforeUnixEpoch => formatter.write_str("UTC timestamp precedes Unix epoch"),
            Self::OutOfRange => formatter.write_str("UTC timestamp exceeds the supported range"),
        }
    }
}

impl std::error::Error for TimeError {}

/// Process-local monotonic time in nanoseconds from an implementation-defined origin.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MonotonicInstant(u64);

impl MonotonicInstant {
    /// Constructs an instant from nanoseconds since the clock's monotonic origin.
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Returns nanoseconds since the clock's monotonic origin.
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// Returns elapsed time, or `None` when `earlier` is later than this instant.
    #[must_use]
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        self.0.checked_sub(earlier.0).map(Duration::from_nanos)
    }
}

/// UTC wall time expressed as nanoseconds since the Unix epoch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UtcTimestamp(u64);

impl UtcTimestamp {
    /// Constructs a supported UTC timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`TimeError::BeforeUnixEpoch`] for a negative value and
    /// [`TimeError::OutOfRange`] when the value cannot fit the supported range.
    pub fn from_unix_nanos(nanos: i128) -> Result<Self, TimeError> {
        if nanos < 0 {
            Err(TimeError::BeforeUnixEpoch)
        } else {
            u64::try_from(nanos)
                .map(Self)
                .map_err(|_| TimeError::OutOfRange)
        }
    }

    /// Returns nanoseconds since the Unix epoch.
    #[must_use]
    pub const fn as_unix_nanos(self) -> u64 {
        self.0
    }
}

/// Origin of a wall-clock reading.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockSource {
    /// The operating system's system clock.
    System,
    /// A deterministic clock controlled by a replay or test.
    Replay,
    /// An external reference clock.
    ExternalReference,
}

/// Declared synchronization state of a UTC reading.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SynchronizationQuality {
    /// Synchronization has been verified within the stated uncertainty.
    Synchronized,
    /// Synchronization is known to be degraded.
    Degraded,
    /// Synchronization has not been established.
    Unknown,
}

/// Correlation between monotonic and UTC clock domains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockCorrelation {
    monotonic: MonotonicInstant,
    utc: UtcTimestamp,
    source: ClockSource,
    quality: SynchronizationQuality,
    uncertainty: Duration,
}

impl ClockCorrelation {
    /// Creates an explicit clock correlation sample.
    #[must_use]
    pub const fn new(
        monotonic: MonotonicInstant,
        utc: UtcTimestamp,
        source: ClockSource,
        quality: SynchronizationQuality,
        uncertainty: Duration,
    ) -> Self {
        Self {
            monotonic,
            utc,
            source,
            quality,
            uncertainty,
        }
    }
}

/// A complete clock reading used at domain boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockReading(ClockCorrelation);

impl ClockReading {
    /// Returns monotonic event time.
    #[must_use]
    pub const fn monotonic(self) -> MonotonicInstant {
        self.0.monotonic
    }
    /// Returns correlated UTC time.
    #[must_use]
    pub const fn utc(self) -> UtcTimestamp {
        self.0.utc
    }
    /// Returns the clock source.
    #[must_use]
    pub const fn source(self) -> ClockSource {
        self.0.source
    }
    /// Returns synchronization quality.
    #[must_use]
    pub const fn quality(self) -> SynchronizationQuality {
        self.0.quality
    }
    /// Returns the maximum declared alignment uncertainty.
    #[must_use]
    pub const fn uncertainty(self) -> Duration {
        self.0.uncertainty
    }
}

impl From<ClockCorrelation> for ClockReading {
    fn from(value: ClockCorrelation) -> Self {
        Self(value)
    }
}

/// Domain port for obtaining correlated monotonic and UTC time.
pub trait Clock {
    /// Obtains a single coherent clock reading.
    fn now(&self) -> ClockReading;
}
