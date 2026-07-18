//! Immutable repository and Experiment Validation registry integration tests.

use ste_experiment_validation::domain::{ArtifactDigest, PromotionDecision, PromotionRegistry};
use ste_physiology_estimation::{
    AtomicPhysiologyRepository, ExperimentValidationRegistry, PhysiologyRepositoryError,
    application::{PhysiologyAssessmentRepository, ValidationRegistry},
    domain::{AbstentionReason, AssessmentOutcome},
};

#[test]
fn repository_is_idempotent_but_rejects_conflicting_replacement() {
    let mut repository = AtomicPhysiologyRepository::default();
    let motion = AssessmentOutcome::Abstained(AbstentionReason::Motion);
    repository.append("assessment-1", &motion).unwrap();
    repository.append("assessment-1", &motion).unwrap();
    assert_eq!(repository.len(), 1);
    assert_eq!(repository.get("assessment-1"), Some(&motion));
    assert_eq!(
        repository.append(
            "assessment-1",
            &AssessmentOutcome::Abstained(AbstentionReason::NotPromoted),
        ),
        Err(PhysiologyRepositoryError::ImmutableConflict)
    );
    assert_eq!(repository.get("assessment-1"), Some(&motion));
}

#[test]
fn exact_latest_promotion_enables_only_the_bound_model_and_capability() {
    let mut decisions = PromotionRegistry::default();
    decisions
        .append(
            PromotionDecision::promoted(
                "respiration-v1",
                "study-1",
                ArtifactDigest::new([9; 32]),
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let adapter =
        ExperimentValidationRegistry::new(&decisions, "respiration-v1", "resp-baseline-v1");
    assert!(adapter.respiration_is_promoted("resp-baseline-v1").unwrap());
    assert!(!adapter.respiration_is_promoted("different-model").unwrap());
    let wrong_capability =
        ExperimentValidationRegistry::new(&decisions, "cardiac-v1", "resp-baseline-v1");
    assert!(
        !wrong_capability
            .respiration_is_promoted("resp-baseline-v1")
            .unwrap()
    );
}

#[test]
fn a_later_rejection_withdraws_promotion_fail_closed() {
    let mut decisions = PromotionRegistry::default();
    decisions
        .append(
            PromotionDecision::promoted(
                "respiration-v1",
                "study-1",
                ArtifactDigest::new([1; 32]),
                1,
            )
            .unwrap(),
        )
        .unwrap();
    decisions
        .append(
            PromotionDecision::rejected("respiration-v1", "study-2", "resource gate failed", 2)
                .unwrap(),
        )
        .unwrap();
    let adapter =
        ExperimentValidationRegistry::new(&decisions, "respiration-v1", "resp-baseline-v1");
    assert!(!adapter.respiration_is_promoted("resp-baseline-v1").unwrap());
}
