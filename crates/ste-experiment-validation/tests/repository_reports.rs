//! Atomic repository, negative-evidence, and de-identified report tests.

use ste_experiment_validation::application::{EvidenceExportReader, ValidationStudyRepository};
use ste_experiment_validation::domain::{
    ArtifactDigest, Cohort, PromotionDecision, Protocol, StudyResult, ValidationStudy,
};
use ste_experiment_validation::{
    AtomicValidationRepository, RepositoryError, ReproducibleValidationReport,
};

fn digest(byte: u8) -> ArtifactDigest {
    ArtifactDigest::new([byte; 32])
}

fn frozen() -> ste_experiment_validation::domain::FrozenStudy {
    ValidationStudy::draft(
        "synthetic-study",
        Protocol::new("synthetic-protocol", digest(1)).unwrap(),
        Cohort::Synthetic,
    )
    .unwrap()
    .freeze(None)
    .unwrap()
}

#[test]
fn immutable_negative_run_and_rejection_are_preserved_idempotently() {
    let frozen = frozen();
    let run = frozen
        .start_run("run-negative", digest(2), digest(3), digest(4), digest(5))
        .unwrap()
        .complete(StudyResult::rejected("gate failed", digest(6)).unwrap())
        .unwrap();
    let decision =
        PromotionDecision::rejected("synthetic-capability", frozen.id(), "gate failed", 100)
            .unwrap();
    let mut repository = AtomicValidationRepository::default();
    repository.save_frozen(&frozen).unwrap();
    repository.append_run(&run).unwrap();
    repository.append_run(&run).unwrap();
    repository.append_promotion(&decision).unwrap();
    repository.append_promotion(&decision).unwrap();

    assert_eq!(repository.run_count(), 1);
    assert_eq!(repository.promotion_count(), 1);
    let export = repository.deidentified_export(frozen.id()).unwrap();
    let json = serde_json::to_string(&export).unwrap();
    assert!(json.contains("Rejected"));
    assert!(json.contains("gate failed"));
}

#[test]
fn same_run_identifier_with_different_result_is_rejected_atomically() {
    let frozen = frozen();
    let run = frozen
        .start_run("run-one", digest(2), digest(3), digest(4), digest(5))
        .unwrap();
    let rejected = run
        .clone()
        .complete(StudyResult::rejected("negative", digest(6)).unwrap())
        .unwrap();
    let passed = run
        .complete(StudyResult::Passed {
            evidence: digest(7),
        })
        .unwrap();
    let mut repository = AtomicValidationRepository::default();
    repository.append_run(&rejected).unwrap();
    assert_eq!(
        repository.append_run(&passed),
        Err(RepositoryError::ImmutableConflict)
    );
    assert_eq!(repository.run_count(), 1);
}

#[test]
fn evidence_export_omits_dataset_grouping_and_is_byte_reproducible() {
    let frozen = frozen();
    let run = frozen
        .start_run("run-report", digest(2), digest(3), digest(4), digest(5))
        .unwrap()
        .complete(StudyResult::rejected("preserved negative", digest(6)).unwrap())
        .unwrap();
    let mut repository = AtomicValidationRepository::default();
    repository.save_frozen(&frozen).unwrap();
    repository.append_run(&run).unwrap();
    let export = repository.deidentified_export(frozen.id()).unwrap();
    let first = ReproducibleValidationReport::generate(&export).unwrap();
    let second = ReproducibleValidationReport::generate(&export).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.digest().len(), 64);
    let body = String::from_utf8(first.bytes().to_vec()).unwrap();
    for prohibited in ["participant", "session", "room", "collection day"] {
        assert!(!body.contains(prohibited));
    }
}

#[test]
fn human_study_cannot_be_frozen_or_stored_without_authority() {
    let study = ValidationStudy::draft(
        "human-study",
        Protocol::new("human-protocol", digest(9)).unwrap(),
        Cohort::Human,
    )
    .unwrap();
    assert!(study.freeze(None).is_err());
}
