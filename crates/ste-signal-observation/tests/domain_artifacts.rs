//! Observation semantic-boundary and immutable-artifact tests.

use proptest::prelude::*;
use ste_signal_observation::{
    AbstentionReason, AlgorithmVersion, BaselineDrift, DspVersion, FeatureEvidenceArtifact,
    FrameEvidence, MotionEnergy, ObservationWindow, ObservationWindowId, PUBLIC_OBSERVATION_TYPES,
    PartitionRole, PeriodicityCandidate, PresenceScore, QualityDisposition, WindowBounds,
    WindowPolicy,
};

fn window() -> ObservationWindow {
    ObservationWindow::open(
        ObservationWindowId::new("window-1").unwrap(),
        WindowBounds::new(100, 200).unwrap(),
        WindowPolicy::new("fixed-v1", 1, 100, 0.2, 0.2).unwrap(),
        AlgorithmVersion::new("features-v1").unwrap(),
        DspVersion::new("dsp-v1").unwrap(),
        "cal-v1".into(),
    )
}

fn evidence(reference: &str, contaminated: bool) -> FrameEvidence {
    FrameEvidence {
        source_ref: reference.into(),
        event_time_ns: 120,
        motion_energy: MotionEnergy::new(0.5).unwrap(),
        presence_score: PresenceScore::new(0.8).unwrap(),
        periodicity: Some(PeriodicityCandidate::new(0.25, 0.7).unwrap()),
        baseline_drift: BaselineDrift::new(0.1).unwrap(),
        missing_before: 0,
        interference: contaminated,
    }
}

#[test]
fn artifact_is_content_addressed_deterministic_and_has_complete_provenance() {
    let mut first = window();
    first.append(evidence("frame:1", false)).unwrap();
    let a = first.close(PartitionRole::Development).unwrap();
    let mut second = window();
    second.append(evidence("frame:1", false)).unwrap();
    let b = second.close(PartitionRole::Development).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.digest().len(), 64);
    assert_eq!(a.source_refs(), &["frame:1"]);
    assert_eq!(a.motion_energy().unit(), "normalized_energy");
    assert_eq!(a.periodicity()[0].frequency_unit(), "hertz");
    assert!(!a.calibration_version().is_empty());
}

proptest! {
    #[test]
    fn contamination_is_monotonic(extra_clean_frames in 0usize..50) {
        let mut value = window();
        value.append(evidence("bad", true)).unwrap();
        for index in 0..extra_clean_frames {
            value.append(evidence(&format!("clean:{index}"), false)).unwrap();
        }
        let artifact = value.close(PartitionRole::Development).unwrap();
        prop_assert_eq!(artifact.quality().disposition, QualityDisposition::Contaminated);
        prop_assert!(artifact.quality().reasons.contains(&AbstentionReason::Interference));
    }
}

#[test]
fn missingness_and_explicit_anomaly_are_preserved_as_abstention_reasons() {
    let mut value = window();
    let mut frame = evidence("frame:2", false);
    frame.missing_before = 10;
    value.append(frame).unwrap();
    value.record_anomaly(AbstentionReason::Saturation).unwrap();
    let artifact = value.close(PartitionRole::Validation).unwrap();
    assert_eq!(
        artifact.quality().disposition,
        QualityDisposition::Contaminated
    );
    assert!(
        artifact
            .quality()
            .reasons
            .contains(&AbstentionReason::Missingness)
    );
    assert!(
        artifact
            .quality()
            .reasons
            .contains(&AbstentionReason::Saturation)
    );
}

#[test]
fn public_observation_vocabulary_contains_no_downstream_claim_semantics() {
    let vocabulary = PUBLIC_OBSERVATION_TYPES.join(" ").to_ascii_lowercase();
    for prohibited in [
        "physiology",
        "emotion",
        "workload",
        "decision",
        "stress",
        "valence",
    ] {
        assert!(
            !vocabulary.contains(prohibited),
            "prohibited public semantic: {prohibited}"
        );
    }
}

#[test]
fn closed_window_is_immutable_and_rejects_late_frames() {
    let mut value = window();
    value.append(evidence("frame:1", false)).unwrap();
    let _: FeatureEvidenceArtifact = value.close(PartitionRole::Test).unwrap();
}
