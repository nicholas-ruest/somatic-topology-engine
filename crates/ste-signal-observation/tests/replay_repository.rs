//! Content-addressed repository and deterministic observation replay tests.

use ste_signal_observation::dsp::{DspGraphSpec, PrimitiveCsiFrame};
use ste_signal_observation::{
    AlgorithmVersion, ContentAddressedEvidenceRepository, ContentAddressedStore, DspVersion,
    ObservationReplay, ObservationWindowId, PartitionRole, PutOutcome, ReplayEvidenceFrame,
    RepositoryError, WindowBounds, WindowPolicy,
};

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn frames() -> Vec<ReplayEvidenceFrame> {
    (0..4)
        .map(|index| ReplayEvidenceFrame {
            source_ref: format!("radio-frame:{}", index + 1),
            frame: PrimitiveCsiFrame {
                source_ref: format!("radio-frame:{}", index + 1),
                event_time_ns: 100 + index * 10,
                subcarriers: vec![(1.0 + index as f64 * 0.1, -0.25)],
            },
        })
        .collect()
}

fn artifact() -> ste_signal_observation::FeatureEvidenceArtifact {
    ObservationReplay::replay(
        ObservationWindowId::new("window-replay-1").unwrap(),
        WindowBounds::new(90, 150).unwrap(),
        WindowPolicy::new("fixed-v1", 1, 16, 0.5, 0.5).unwrap(),
        AlgorithmVersion::new("features-v1").unwrap(),
        DspVersion::new("dsp-v1").unwrap(),
        "calibration-v1".into(),
        PartitionRole::Development,
        DspGraphSpec {
            version: 1,
            sample_rate_hz: 100_000_000.0_f64.recip() * 1_000_000_000.0,
            window_len: 4,
            saturation_magnitude: 100.0,
            presence_threshold: 0.0,
            periodicity_min_lag: 1,
            periodicity_max_lag: 2,
        },
        &frames(),
    )
    .unwrap()
}

#[test]
fn exact_artifact_put_is_idempotent_and_retrievable() {
    let repository = ContentAddressedEvidenceRepository::default();
    let artifact = artifact();
    assert_eq!(repository.put(&artifact), Ok(PutOutcome::Inserted));
    assert_eq!(repository.put(&artifact), Ok(PutOutcome::AlreadyPresent));
    assert_eq!(repository.len(), Ok(1));
    assert_eq!(repository.get(artifact.digest()), Ok(Some(artifact)));
}

#[test]
fn occupied_digest_with_different_content_is_a_hard_collision() {
    let repository = ContentAddressedStore::default();
    assert_eq!(
        repository.put_verified(DIGEST, &"first"),
        Ok(PutOutcome::Inserted)
    );
    assert_eq!(
        repository.put_verified(DIGEST, &"different"),
        Err(RepositoryError::DigestCollision)
    );
    assert_eq!(repository.get(DIGEST), Ok(Some("first")));
}

#[test]
fn malformed_digest_is_rejected_before_storage() {
    let repository = ContentAddressedStore::default();
    assert_eq!(
        repository.put_verified("not-a-sha256", &1_u8),
        Err(RepositoryError::InvalidDigest)
    );
    assert_eq!(repository.is_empty(), Ok(true));
    assert_eq!(
        repository.put_verified(&"A".repeat(64), &1_u8),
        Err(RepositoryError::InvalidDigest)
    );
}

#[test]
fn identical_radio_replay_produces_identical_artifact_and_full_sources() {
    let first = artifact();
    let second = artifact();
    assert_eq!(first, second);
    assert_eq!(first.source_refs().len(), 4);
    assert_eq!(first.source_refs()[0], "radio-frame:1");
    assert_eq!(first.source_refs()[3], "radio-frame:4");
}

#[test]
fn public_replay_and_repository_types_do_not_introduce_claim_semantics() {
    let names = [
        std::any::type_name::<ObservationReplay>(),
        std::any::type_name::<ContentAddressedEvidenceRepository>(),
        std::any::type_name::<ReplayEvidenceFrame>(),
    ]
    .join(" ")
    .to_ascii_lowercase();
    for prohibited in ["physiology", "emotion", "workload", "decision", "valence"] {
        assert!(!names.contains(prohibited));
    }
}
