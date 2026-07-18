//! Dependency-free domain primitives shared by every STE bounded context.
//!
//! Domain crates use these opaque types and ports without acquiring a
//! serialization, async-runtime, operating-system, or hardware dependency.

#![forbid(unsafe_code)]

mod event;
mod id;
mod probability;
mod provenance;
mod time;
mod units;

pub use event::DomainEvent;
pub use id::{AggregateId, CausationId, CorrelationId, EventId, IdError, IdGenerator};
pub use probability::{FiniteProbability, ProbabilityError};
pub use provenance::{Provenance, ProvenanceError, ProvenanceRef};
pub use time::{
    Clock, ClockCorrelation, ClockReading, ClockSource, MonotonicInstant, SynchronizationQuality,
    TimeError, UtcTimestamp,
};
pub use units::{Celsius, Hertz, Meters, UnitError};
