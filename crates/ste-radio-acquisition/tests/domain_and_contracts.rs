//! Outside-in acquisition validation and policy-gated publication tests.

use std::sync::{Arc, Mutex};

use proptest::prelude::*;
use ste_radio_acquisition::{
    AcquisitionError, AcquisitionService, CalibrationMetadata, CaptureAuthorizationPort,
    CaptureHealth, CaptureLink, CaptureProfile, CaptureSession, CsiFrameInput, FrameJournal,
    FramePublisher, HardwareProvenance, PublicationOutcome, SessionRepository,
};

fn profile() -> CaptureProfile {
    CaptureProfile::new(5_180_000_000, 20_000_000, 1, 2).unwrap()
}

fn provenance() -> HardwareProvenance {
    HardwareProvenance::new("pi4", "bcm43455", "nexmon-1", "linux-6", "ap-a").unwrap()
}

fn frame(sequence: u64) -> CsiFrameInput {
    CsiFrameInput {
        sequence,
        monotonic_time_ns: 100 + sequence,
        center_frequency_hz: 5_180_000_000,
        bandwidth_hz: 20_000_000,
        antenna_count: 1,
        subcarriers: vec![(1.0, -1.0), (0.5, 0.25)],
    }
}

proptest! {
    #[test]
    fn any_non_finite_component_is_rejected(sequence in 1u64..10_000, imaginary in any::<bool>()) {
        let mut input = frame(sequence);
        input.subcarriers[0] = if imaginary { (1.0, f64::NAN) } else { (f64::INFINITY, 1.0) };
        let session = CaptureSession::start(profile(), CaptureLink::new("wlan0", "link-a").unwrap(), provenance(), CalibrationMetadata::new("cal-v1", "geometry-a").unwrap(), 10).unwrap();
        prop_assert_eq!(session.validate_frame(input), Err(AcquisitionError::NonFiniteCsi));
    }
}

#[test]
fn frame_contract_preserves_sequence_gap_quality_and_full_provenance_reference() {
    let mut session = CaptureSession::start(
        profile(),
        CaptureLink::new("wlan0", "link-a").unwrap(),
        provenance(),
        CalibrationMetadata::new("cal-v1", "geometry-a").unwrap(),
        10,
    )
    .unwrap();
    session.accept(frame(1)).unwrap();
    let accepted = session.accept(frame(4)).unwrap();
    assert_eq!(accepted.contract.sequence, 4);
    assert_eq!(accepted.gap_before, 2);
    assert_eq!(
        session.health(),
        CaptureHealth {
            accepted: 2,
            rejected: 0,
            missing: 2,
            backpressured: 0
        }
    );
    assert!(accepted.provenance_ref.contains("nexmon-1"));
    assert!(accepted.provenance_ref.contains("cal-v1"));
    assert!(!accepted.contract.payload_ref.is_empty());
}

#[derive(Default)]
struct Harness {
    authorized: bool,
    published: Mutex<Vec<u64>>,
    journaled: Mutex<Vec<u64>>,
    outcome: Mutex<PublicationOutcome>,
    saved: Mutex<usize>,
}

impl CaptureAuthorizationPort for Harness {
    fn authorize_capture(&self, _: &CaptureSession) -> bool {
        self.authorized
    }
}
impl FramePublisher for Harness {
    fn publish(&self, frame: &ste_radio_acquisition::ValidatedFrame) -> PublicationOutcome {
        let outcome = *self.outcome.lock().unwrap();
        if outcome == PublicationOutcome::Published {
            self.published.lock().unwrap().push(frame.contract.sequence);
        }
        outcome
    }
}
impl FrameJournal for Harness {
    fn append(
        &self,
        frame: &ste_radio_acquisition::ValidatedFrame,
    ) -> Result<(), AcquisitionError> {
        self.journaled.lock().unwrap().push(frame.contract.sequence);
        Ok(())
    }
}
impl SessionRepository for Harness {
    fn save(&self, _: &CaptureSession) -> Result<(), AcquisitionError> {
        *self.saved.lock().unwrap() += 1;
        Ok(())
    }
}

#[test]
fn no_frame_is_journaled_or_published_without_current_policy_authorization() {
    let denied = Arc::new(Harness::default());
    let mut service = AcquisitionService::new(
        denied.clone(),
        denied.clone(),
        denied.clone(),
        denied.clone(),
    );
    let mut session = CaptureSession::start(
        profile(),
        CaptureLink::new("wlan0", "link-a").unwrap(),
        provenance(),
        CalibrationMetadata::new("cal-v1", "geometry-a").unwrap(),
        10,
    )
    .unwrap();
    assert_eq!(
        service.process(&mut session, frame(1)),
        Err(AcquisitionError::Unauthorized)
    );
    assert!(denied.published.lock().unwrap().is_empty());
    assert!(denied.journaled.lock().unwrap().is_empty());
}

#[test]
fn bounded_backpressure_is_explicit_and_never_reported_as_publication() {
    let harness = Arc::new(Harness {
        authorized: true,
        outcome: Mutex::new(PublicationOutcome::Backpressured),
        ..Harness::default()
    });
    let mut service = AcquisitionService::new(
        harness.clone(),
        harness.clone(),
        harness.clone(),
        harness.clone(),
    );
    let mut session = CaptureSession::start(
        profile(),
        CaptureLink::new("wlan0", "link-a").unwrap(),
        provenance(),
        CalibrationMetadata::new("cal-v1", "geometry-a").unwrap(),
        10,
    )
    .unwrap();
    assert_eq!(
        service.process(&mut session, frame(1)).unwrap(),
        PublicationOutcome::Backpressured
    );
    assert!(harness.published.lock().unwrap().is_empty());
    assert_eq!(session.health().backpressured, 1);
}

#[test]
fn implausible_channel_shape_and_reordered_sequence_fail_closed() {
    assert!(CaptureProfile::new(100, 20_000_000, 1, 2).is_err());
    let mut session = CaptureSession::start(
        profile(),
        CaptureLink::new("wlan0", "link-a").unwrap(),
        provenance(),
        CalibrationMetadata::new("cal-v1", "geometry-a").unwrap(),
        10,
    )
    .unwrap();
    session.accept(frame(2)).unwrap();
    assert_eq!(
        session.accept(frame(2)),
        Err(AcquisitionError::NonMonotonicSequence)
    );
}
