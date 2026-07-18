//! Behavioral acceptance tests for shared domain primitives and ports.

use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use ste_kernel::{
    AggregateId, CausationId, Celsius, Clock, ClockCorrelation, ClockReading, ClockSource,
    CorrelationId, DomainEvent, EventId, FiniteProbability, Hertz, IdGenerator, Meters,
    MonotonicInstant, Provenance, ProvenanceRef, SynchronizationQuality, UtcTimestamp,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct Recorded(EventId);

impl DomainEvent for Recorded {
    fn event_id(&self) -> EventId {
        self.0
    }
    fn event_name(&self) -> &'static str {
        "Recorded"
    }
}

struct FixedClock(ClockReading);

impl Clock for FixedClock {
    fn now(&self) -> ClockReading {
        self.0
    }
}

struct SequentialIds(u128);

fn assert_same_float(actual: f64, expected: f64) {
    assert_eq!(actual.to_bits(), expected.to_bits());
}

impl IdGenerator for SequentialIds {
    fn next_id(&mut self) -> EventId {
        self.0 += 1;
        EventId::from_u128(self.0).expect("nonzero sequence")
    }
}

#[test]
fn should_reject_zero_ids_and_preserve_opaque_identity() {
    assert!(EventId::from_u128(0).is_err());
    assert!(AggregateId::from_u128(0).is_err());
    assert!(CorrelationId::from_u128(0).is_err());
    assert!(CausationId::from_u128(0).is_err());
    let first = EventId::from_u128(1).unwrap();
    let second = EventId::from_u128(2).unwrap();
    assert_ne!(first, second);
    assert_eq!(first.as_u128(), 1);
    assert_eq!(HashSet::from([first, second]).len(), 2);
}

#[test]
fn should_reject_negative_utc_time_and_order_monotonic_time() {
    assert!(UtcTimestamp::from_unix_nanos(-1).is_err());
    let start = MonotonicInstant::from_nanos(50);
    let end = MonotonicInstant::from_nanos(75);
    assert_eq!(
        end.checked_duration_since(start),
        Some(Duration::from_nanos(25))
    );
    assert_eq!(start.checked_duration_since(end), None);
}

#[test]
fn should_create_explicit_clock_correlation_with_uncertainty() {
    let correlation = ClockCorrelation::new(
        MonotonicInstant::from_nanos(10),
        UtcTimestamp::from_unix_nanos(1_000).unwrap(),
        ClockSource::System,
        SynchronizationQuality::Synchronized,
        Duration::from_micros(5),
    );
    let reading = FixedClock(correlation.into()).now();
    assert_eq!(reading.monotonic(), MonotonicInstant::from_nanos(10));
    assert_eq!(reading.uncertainty(), Duration::from_micros(5));
}

#[test]
fn should_reject_non_finite_or_out_of_range_probabilities() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01, 1.01] {
        assert!(FiniteProbability::new(invalid).is_err());
    }
    assert_same_float(FiniteProbability::new(0.0).unwrap().get(), 0.0);
    assert_same_float(FiniteProbability::new(1.0).unwrap().get(), 1.0);
}

#[test]
fn should_reject_non_finite_or_negative_physical_units() {
    for invalid in [f64::NAN, f64::INFINITY, -0.1] {
        assert!(Hertz::new(invalid).is_err());
    }
    assert_same_float(Hertz::new(2.4e9).unwrap().get(), 2.4e9);
    assert!(Meters::new(-1.0).is_err());
    assert!(Celsius::new(f64::NAN).is_err());
    assert_same_float(Celsius::new(-20.0).unwrap().get(), -20.0);
}

#[test]
fn should_require_complete_provenance_identity() {
    assert!(ProvenanceRef::new("").is_err());
    let provenance = Provenance::new(
        ProvenanceRef::new("sha256:abc").unwrap(),
        "ste-dsp/1.0.0",
        "capture-profile-7",
    )
    .unwrap();
    assert!(
        Provenance::new(
            ProvenanceRef::new("sha256:abc").unwrap(),
            "",
            "capture-profile-7"
        )
        .is_err()
    );
    assert_eq!(provenance.reference().as_str(), "sha256:abc");
    assert_eq!(provenance.producer(), "ste-dsp/1.0.0");
}

#[test]
fn should_expose_stable_domain_event_metadata() {
    let event = Recorded(EventId::from_u128(4).unwrap());
    assert_eq!(event.event_name(), "Recorded");
    assert_eq!(event.event_id().as_u128(), 4);
}

#[test]
fn should_generate_unique_nonzero_identifiers_through_port() -> Result<(), Box<dyn Error>> {
    let mut generator = SequentialIds(100);
    let first = generator.next_id();
    let second = generator.next_id();
    assert_ne!(first, second);
    assert_ne!(first.as_u128(), 0);
    let correlation = CorrelationId::from_u128(first.as_u128())?;
    assert_eq!(correlation.as_u128(), first.as_u128());
    Ok(())
}
