//! Acceptance tests for study freeze, dataset leakage, and promotion history.

use ste_experiment_validation::domain::{
    ArtifactDigest, AuthorityEvidence, Cohort, DatasetCard, DatasetManifest, DatasetRecord,
    DatasetSplit, DomainError, PartitionRole, PromotionDecision, PromotionRegistry, Protocol,
    StudyResult, ValidationStudy,
};

fn digest(byte: u8) -> ArtifactDigest {
    ArtifactDigest::new([byte; 32])
}

fn complete_manifest(records: Vec<DatasetRecord>) -> DatasetManifest {
    DatasetManifest::new(
        "respiration-pilot-v1",
        "research:respiration",
        "consent-v1",
        "commercial-and-research",
        "belt-and-csi-v1",
        "rust-pipeline@abc123",
        "P30D",
        0.04,
        digest(1),
        records,
    )
    .expect("manifest is complete")
}

#[test]
fn should_reject_a_participant_session_room_or_day_crossing_partition_boundaries() {
    for contaminated in [
        ("p1", "s2", "r2", "d2"),
        ("p2", "s1", "r2", "d2"),
        ("p2", "s2", "r1", "d2"),
        ("p2", "s2", "r2", "d1"),
    ] {
        let records = vec![
            DatasetRecord::new("a", "p1", "s1", "r1", "d1", PartitionRole::Train).expect("record"),
            DatasetRecord::new(
                "b",
                contaminated.0,
                contaminated.1,
                contaminated.2,
                contaminated.3,
                PartitionRole::Test,
            )
            .expect("record"),
        ];

        assert_eq!(
            complete_manifest(records).validate_split(),
            Err(DomainError::SplitLeakage)
        );
    }
}

#[test]
fn should_accept_a_complete_non_leaking_four_way_split_and_dataset_card() {
    let records = [
        ("train", "p1", "s1", "r1", "d1", PartitionRole::Train),
        ("dev", "p2", "s2", "r2", "d2", PartitionRole::Development),
        ("cal", "p3", "s3", "r3", "d3", PartitionRole::Calibration),
        ("test", "p4", "s4", "r4", "d4", PartitionRole::Test),
    ]
    .map(|r| DatasetRecord::new(r.0, r.1, r.2, r.3, r.4, r.5).expect("record"))
    .to_vec();
    let manifest = complete_manifest(records);
    let card = DatasetCard::new(
        manifest,
        "single-adult pilot",
        "not population representative",
    )
    .expect("complete card");

    assert_eq!(
        card.manifest().validate_split(),
        Ok(DatasetSplit::ParticipantSessionRoomDay)
    );
}

#[test]
fn should_block_human_freeze_without_ethics_and_consent_authority() {
    let study = ValidationStudy::draft(
        "study-1",
        Protocol::new("respiration", digest(2)).unwrap(),
        Cohort::Human,
    )
    .expect("draft");

    assert_eq!(study.freeze(None), Err(DomainError::AuthorityRequired));
}

#[test]
fn should_allow_synthetic_freeze_and_preserve_immutable_negative_results() {
    let manifest = complete_manifest(vec![
        DatasetRecord::new("a", "p1", "s1", "r1", "d1", PartitionRole::Train).unwrap(),
        DatasetRecord::new("b", "p2", "s2", "r2", "d2", PartitionRole::Test).unwrap(),
    ]);
    let frozen = ValidationStudy::draft(
        "study-2",
        Protocol::new("synthetic-respiration", digest(3)).unwrap(),
        Cohort::Synthetic,
    )
    .unwrap()
    .with_dataset(manifest)
    .unwrap()
    .freeze(None)
    .expect("synthetic studies need no human authority");
    let run = frozen
        .start_run("run-1", digest(4), digest(5), digest(6), digest(7))
        .unwrap();
    let result = StudyResult::rejected("accuracy gate failed", digest(8)).unwrap();
    let completed = run.complete(result.clone()).unwrap();

    assert_eq!(completed.result(), &result);
    assert_eq!(
        completed.complete(result),
        Err(DomainError::RunAlreadyCompleted)
    );
}

#[test]
fn should_require_matching_human_authority_and_keep_promotion_registry_append_only() {
    let authority = AuthorityEvidence::new("ethics-2026-7", "consent-v4", "CA", 100, 200).unwrap();
    let frozen = ValidationStudy::draft(
        "study-3",
        Protocol::new("human-respiration", digest(9)).unwrap(),
        Cohort::Human,
    )
    .unwrap()
    .freeze(Some(authority))
    .expect("complete authority");
    let mut registry = PromotionRegistry::default();

    registry
        .append(
            PromotionDecision::rejected("respiration-v1", frozen.id(), "gate failed", 101).unwrap(),
        )
        .unwrap();
    registry
        .append(
            PromotionDecision::promoted("respiration-v1", frozen.id(), digest(10), 102).unwrap(),
        )
        .unwrap();

    assert_eq!(registry.history("respiration-v1").len(), 2);
}
