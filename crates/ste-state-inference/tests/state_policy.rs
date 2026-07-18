//! Claim-boundary acceptance tests for state inference.
use ste_state_inference::domain::{
    AbstentionReason, AssessLatentState, AssessmentOutcome, CalibratedProbability, ClaimLevel,
    ConstructDefinition, DisplayProjectionV1, EvidenceBundle, GateEvidence, ModelScope, Provenance,
    StateAssessment, WorkloadBand,
};

fn evidence() -> EvidenceBundle {
    EvidenceBundle::new(
        "obs-1",
        "phys-1",
        30_000,
        Provenance::new("model-v1", "cal-v1", "profile-v1").unwrap(),
    )
    .unwrap()
}
fn scope() -> ModelScope {
    ModelScope::new("p1", "room1", "task1", "seated", "pi5").unwrap()
}
fn passing() -> GateEvidence {
    GateEvidence::all_passed()
}

#[test]
fn each_mandatory_gate_abstains_before_a_claim() {
    let failures = [
        (
            GateEvidence {
                construct_promoted: false,
                ..passing()
            },
            AbstentionReason::ConstructNotPromoted,
        ),
        (
            GateEvidence {
                physiology_valid: false,
                ..passing()
            },
            AbstentionReason::InvalidPhysiology,
        ),
        (
            GateEvidence {
                profile_valid: false,
                ..passing()
            },
            AbstentionReason::InvalidProfile,
        ),
        (
            GateEvidence {
                model_active: false,
                ..passing()
            },
            AbstentionReason::ModelUnavailable,
        ),
        (
            GateEvidence {
                calibration_valid: false,
                ..passing()
            },
            AbstentionReason::CalibrationInvalid,
        ),
        (
            GateEvidence {
                policy_authorized: false,
                ..passing()
            },
            AbstentionReason::PolicyDisabled,
        ),
        (
            GateEvidence {
                in_scope: false,
                ..passing()
            },
            AbstentionReason::OutOfScope,
        ),
        (
            GateEvidence {
                inside_envelope: false,
                ..passing()
            },
            AbstentionReason::OutsideOperatingEnvelope,
        ),
    ];
    for (gates, reason) in failures {
        let command = AssessLatentState::new(
            "a1",
            ConstructDefinition::task_workload("task1", 1).unwrap(),
            evidence(),
            scope(),
            CalibratedProbability::new(0.8).unwrap(),
            gates,
        )
        .unwrap();
        assert_eq!(
            StateAssessment::assess(command),
            AssessmentOutcome::Abstained {
                assessment_id: "a1".into(),
                reason
            }
        );
    }
}

#[test]
fn promoted_task_specific_workload_projects_only_an_approved_band() {
    let command = AssessLatentState::new(
        "a1",
        ConstructDefinition::task_workload("task1", 1).unwrap(),
        evidence(),
        scope(),
        CalibratedProbability::new(0.8).unwrap(),
        passing(),
    )
    .unwrap();
    let AssessmentOutcome::Estimated(assessment) = StateAssessment::assess(command) else {
        panic!("estimate")
    };
    assert_eq!(assessment.claim_level(), ClaimLevel::ValidatedNonMedical);
    assert_eq!(
        DisplayProjectionV1::try_from(&assessment).unwrap().workload,
        WorkloadBand::Elevated
    );
}

#[test]
fn baseline_is_a_no_claim_path_and_cannot_project_raw_scores() {
    let command = AssessLatentState::new(
        "a2",
        ConstructDefinition::baseline(),
        evidence(),
        scope(),
        CalibratedProbability::new(0.99).unwrap(),
        passing(),
    )
    .unwrap();
    assert_eq!(
        StateAssessment::assess(command),
        AssessmentOutcome::Abstained {
            assessment_id: "a2".into(),
            reason: AbstentionReason::BaselineNoClaim
        }
    );
}
