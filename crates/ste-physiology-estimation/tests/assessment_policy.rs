//! Outside-in policy tests for respiration-only physiology estimation.

use ste_physiology_estimation::{
    application::{PhysiologyModel, ValidationRegistry},
    domain::{
        AbstentionReason, AssessPhysiology, AssessmentOutcome, EvidenceHorizon, EvidenceWindow,
        ModelEstimate, PhysiologyAssessment, ValidationStatus,
    },
};

struct Registry(bool);
impl ValidationRegistry for Registry {
    type Error = ();
    fn respiration_is_promoted(&self, _: &str) -> Result<bool, Self::Error> {
        Ok(self.0)
    }
}

struct Model;
impl PhysiologyModel for Model {
    type Error = ();
    fn estimate_respiration(&self, _: &[f64]) -> Result<ModelEstimate, Self::Error> {
        ModelEstimate::new(15.0, 0.9, 1.5, "baseline-v1").map_err(|_| ())
    }
}

fn evidence() -> EvidenceWindow {
    EvidenceWindow::new(
        vec![1.0, 2.0, 3.0],
        EvidenceHorizon::new(30_000).unwrap(),
        0.95,
        true,
        false,
        true,
        true,
    )
    .unwrap()
}

#[test]
fn unpromoted_validation_gate_cannot_be_overridden() {
    let outcome = PhysiologyAssessment::assess(
        AssessPhysiology::new("a1", "obs1", "resp-v1", evidence()).unwrap(),
        &Registry(false),
        &Model,
    )
    .unwrap();
    assert_eq!(
        outcome,
        AssessmentOutcome::Abstained(AbstentionReason::NotPromoted)
    );
}

#[test]
fn every_operating_policy_failure_abstains() {
    let cases = [
        (
            EvidenceWindow::new(
                vec![1.0],
                EvidenceHorizon::new(30_000).unwrap(),
                0.95,
                false,
                false,
                true,
                true,
            )
            .unwrap(),
            AbstentionReason::Motion,
        ),
        (
            EvidenceWindow::new(
                vec![1.0],
                EvidenceHorizon::new(30_000).unwrap(),
                0.2,
                true,
                false,
                true,
                true,
            )
            .unwrap(),
            AbstentionReason::InsufficientQuality,
        ),
        (
            EvidenceWindow::new(
                vec![1.0],
                EvidenceHorizon::new(30_000).unwrap(),
                0.95,
                true,
                true,
                true,
                true,
            )
            .unwrap(),
            AbstentionReason::OutOfDistribution,
        ),
        (
            EvidenceWindow::new(
                vec![1.0],
                EvidenceHorizon::new(30_000).unwrap(),
                0.95,
                true,
                false,
                false,
                true,
            )
            .unwrap(),
            AbstentionReason::CalibrationInvalid,
        ),
        (
            EvidenceWindow::new(
                vec![1.0],
                EvidenceHorizon::new(5_000).unwrap(),
                0.95,
                true,
                false,
                true,
                true,
            )
            .unwrap(),
            AbstentionReason::InsufficientEvidence,
        ),
        (
            EvidenceWindow::new(
                vec![1.0],
                EvidenceHorizon::new(30_000).unwrap(),
                0.95,
                true,
                false,
                true,
                false,
            )
            .unwrap(),
            AbstentionReason::OutsideOperatingEnvelope,
        ),
    ];
    for (window, reason) in cases {
        let outcome = PhysiologyAssessment::assess(
            AssessPhysiology::new("a", "obs", "resp-v1", window).unwrap(),
            &Registry(true),
            &Model,
        )
        .unwrap();
        assert_eq!(outcome, AssessmentOutcome::Abstained(reason));
    }
}

#[test]
fn promoted_valid_respiration_emits_non_medical_evidence() {
    let outcome = PhysiologyAssessment::assess(
        AssessPhysiology::new("a1", "obs1", "resp-v1", evidence()).unwrap(),
        &Registry(true),
        &Model,
    )
    .unwrap();
    let AssessmentOutcome::Estimated(estimate) = outcome else {
        panic!("expected estimate")
    };
    assert_eq!(
        estimate.validation_status(),
        ValidationStatus::PromotedNonMedical
    );
    assert_eq!(estimate.breaths_per_minute(), 15.0);
}
