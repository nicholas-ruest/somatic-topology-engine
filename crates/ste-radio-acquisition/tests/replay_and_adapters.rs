//! Hostile replay parsing and anti-corruption adapter acceptance tests.

use ste_radio_acquisition::replay::{ReplayError, ReplayLimits, parse_pcap, parse_rvcsi};
use ste_radio_acquisition::{
    CalibrationMetadata, CaptureLink, CaptureProfile, CaptureSession, CsiCaptureSource,
    FrameJournal, HardwareProvenance, InMemoryFrameJournal, InMemorySessionRepository,
    RvCsiAntiCorruptionAdapter, SessionRepository,
};

fn record(sequence: u64, sample: (f64, f64)) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.extend_from_slice(&(sequence * 1_000).to_le_bytes());
    bytes.extend_from_slice(&5_180_000_000_u64.to_le_bytes());
    bytes.extend_from_slice(&20_000_000_u32.to_le_bytes());
    bytes.push(1);
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&sample.0.to_bits().to_le_bytes());
    bytes.extend_from_slice(&sample.1.to_bits().to_le_bytes());
    bytes
}

fn rvcsi(records: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = b"RVCSIv1\0".to_vec();
    for record in records {
        bytes.extend_from_slice(&(record.len() as u32).to_le_bytes());
        bytes.extend_from_slice(record);
    }
    bytes
}

fn pcap(records: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0xd4, 0xc3, 0xb2, 0xa1, 2, 0, 4, 0];
    bytes.extend_from_slice(&[0; 8]);
    bytes.extend_from_slice(&65_535_u32.to_le_bytes());
    bytes.extend_from_slice(&147_u32.to_le_bytes());
    for record in records {
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&(record.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(record.len() as u32).to_le_bytes());
        bytes.extend_from_slice(record);
    }
    bytes
}

#[test]
fn rvcsi_replay_is_deterministic_and_preserves_sequence_gaps() {
    let bytes = rvcsi(&[record(1, (1.0, -1.0)), record(3, (2.0, -2.0))]);
    let first = parse_rvcsi(&bytes, ReplayLimits::default()).unwrap();
    let second = parse_rvcsi(&bytes, ReplayLimits::default()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.frames.len(), 2);
    assert_eq!(first.gaps.len(), 1);
    assert_eq!(first.gaps[0].missing, 1);
}

#[test]
fn pcap_replay_matches_rvcsi_record_semantics() {
    let records = [record(1, (0.5, -0.25)), record(2, (0.75, -0.5))];
    let from_pcap = parse_pcap(&pcap(&records), ReplayLimits::default()).unwrap();
    let from_rvcsi = parse_rvcsi(&rvcsi(&records), ReplayLimits::default()).unwrap();
    assert_eq!(from_pcap.frames, from_rvcsi.frames);
}

#[test]
fn malformed_non_finite_and_implausible_records_are_counted_without_panics() {
    let malformed = vec![1, 2];
    let non_finite = record(1, (f64::NAN, 0.0));
    let mut implausible = record(2, (0.0, 0.0));
    implausible[16..24].copy_from_slice(&100_u64.to_le_bytes());
    let report = parse_rvcsi(
        &rvcsi(&[malformed, non_finite, implausible]),
        ReplayLimits::default(),
    )
    .unwrap();
    assert!(report.frames.is_empty());
    assert_eq!(report.rejected_malformed, 1);
    assert_eq!(report.rejected_non_finite, 1);
    assert_eq!(report.rejected_implausible, 1);
}

#[test]
fn replay_rejects_duplicate_or_reordered_sequence_without_hiding_valid_frames() {
    let report = parse_rvcsi(
        &rvcsi(&[
            record(2, (1.0, 0.0)),
            record(1, (1.0, 0.0)),
            record(3, (1.0, 0.0)),
        ]),
        ReplayLimits::default(),
    )
    .unwrap();
    assert_eq!(
        report
            .frames
            .iter()
            .map(|frame| frame.sequence)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert_eq!(report.rejected_implausible, 1);
}

#[test]
fn declared_budgets_are_enforced_before_allocation() {
    let limits = ReplayLimits {
        max_input_bytes: 7,
        ..ReplayLimits::default()
    };
    assert_eq!(
        parse_rvcsi(b"RVCSIv1\0", limits),
        Err(ReplayError::InputTooLarge)
    );
    let mut oversized = b"RVCSIv1\0".to_vec();
    oversized.extend_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        parse_rvcsi(&oversized, ReplayLimits::default()),
        Err(ReplayError::RecordTooLarge)
    );
}

#[test]
fn every_truncated_prefix_and_arbitrary_small_input_returns_without_panicking() {
    let valid = rvcsi(&[record(1, (1.0, 2.0))]);
    for end in 0..valid.len() {
        assert!(
            std::panic::catch_unwind(|| {
                let _ = parse_rvcsi(&valid[..end], ReplayLimits::default());
                let _ = parse_pcap(&valid[..end], ReplayLimits::default());
            })
            .is_ok()
        );
    }
    for length in 0..512 {
        let bytes = (0..length)
            .map(|index| (index as u8).wrapping_mul(31))
            .collect::<Vec<_>>();
        assert!(
            std::panic::catch_unwind(|| {
                let _ = parse_rvcsi(&bytes, ReplayLimits::default());
                let _ = parse_pcap(&bytes, ReplayLimits::default());
            })
            .is_ok()
        );
    }
}

#[test]
fn anti_corruption_adapter_hides_upstream_shape_and_rejects_bad_iq() {
    let input = RvCsiAntiCorruptionAdapter::adapt_interleaved(
        1,
        1_000,
        5_180_000_000,
        20_000_000,
        1,
        &[1.0, -1.0],
        64,
    )
    .unwrap();
    assert_eq!(input.subcarriers, vec![(1.0, -1.0)]);
    assert!(
        RvCsiAntiCorruptionAdapter::adapt_interleaved(
            1,
            1_000,
            5_180_000_000,
            20_000_000,
            1,
            &[1.0],
            64,
        )
        .is_err()
    );
    assert!(
        RvCsiAntiCorruptionAdapter::adapt_interleaved(
            1,
            1_000,
            5_180_000_000,
            20_000_000,
            1,
            &[f32::NAN, 0.0],
            64,
        )
        .is_err()
    );
}

#[test]
fn replay_source_and_local_journal_repository_implement_application_ports() {
    let mut source = ste_radio_acquisition::ReplayCaptureSource::from_rvcsi(
        &rvcsi(&[record(1, (1.0, -1.0))]),
        ReplayLimits::default(),
    )
    .unwrap();
    let input = source.next_frame().unwrap().unwrap();
    assert!(source.next_frame().unwrap().is_none());
    let mut session = CaptureSession::start(
        CaptureProfile::new(5_180_000_000, 20_000_000, 1, 1).unwrap(),
        CaptureLink::new("wlan0", "link-1").unwrap(),
        HardwareProvenance::new("pi4", "bcm43455", "pinned", "kernel", "ap").unwrap(),
        CalibrationMetadata::new("profile-1", "geometry-1").unwrap(),
        1,
    )
    .unwrap();
    let accepted = session.accept(input).unwrap();
    let journal = InMemoryFrameJournal::default();
    journal.append(&accepted).unwrap();
    assert_eq!(journal.snapshot(), vec![accepted]);
    let repository = InMemorySessionRepository::default();
    repository.save(&session).unwrap();
    assert_eq!(repository.latest().unwrap().health().accepted, 1);
}
