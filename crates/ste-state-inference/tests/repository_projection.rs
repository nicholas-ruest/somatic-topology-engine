//! Immutable persistence and raw-output isolation tests.
use ste_state_inference::{
    AtomicStateRepository, SafeDisplayProjection, StateRepositoryError,
    application::StateAssessmentRepository,
    domain::{
        AbstentionReason, AssessLatentState, AssessmentOutcome, CalibratedProbability,
        ConstructDefinition, EvidenceBundle, GateEvidence, ModelScope, Provenance, StateAssessment,
    },
};

fn estimated() -> AssessmentOutcome {
    let evidence = EvidenceBundle::new(
        "obs-secret",
        "phys-secret",
        30_000,
        Provenance::new("model-secret", "cal-secret", "profile-secret").unwrap(),
    )
    .unwrap();
    let scope = ModelScope::new(
        "participant-secret",
        "room-secret",
        "task1",
        "seated",
        "pi5",
    )
    .unwrap();
    StateAssessment::assess(
        AssessLatentState::new(
            "assessment-1",
            ConstructDefinition::task_workload("task1", 1).unwrap(),
            evidence,
            scope,
            CalibratedProbability::new(0.812345).unwrap(),
            GateEvidence::all_passed(),
        )
        .unwrap(),
    )
}

#[test]
fn repository_is_idempotent_and_rejects_conflicting_replacement() {
    let mut repository = AtomicStateRepository::default();
    let outcome = estimated();
    repository.append("assessment-1", &outcome).unwrap();
    repository.append("assessment-1", &outcome).unwrap();
    assert_eq!(repository.len(), 1);
    assert_eq!(
        repository.append(
            "assessment-1",
            &AssessmentOutcome::Abstained {
                assessment_id: "assessment-1".into(),
                reason: AbstentionReason::PolicyDisabled
            }
        ),
        Err(StateRepositoryError::ImmutableConflict)
    );
    assert_eq!(repository.get("assessment-1"), Some(&outcome));
}

#[test]
fn only_fixed_approved_projection_crosses_ui_api_boundary() {
    let projection = SafeDisplayProjection::try_from(&estimated()).unwrap();
    let json = serde_json::to_string(&projection).unwrap();
    assert!(json.contains("task-specific non-medical workload estimate"));
    for forbidden in [
        "0.812345",
        "obs-secret",
        "phys-secret",
        "model-secret",
        "cal-secret",
        "profile-secret",
        "participant-secret",
        "room-secret",
        "probability",
        "features",
        "tensor",
    ] {
        assert!(!json.contains(forbidden), "leaked {forbidden}: {json}");
    }
}

#[test]
fn abstention_projects_explicit_unavailable_without_manufacturing_a_label() {
    let outcome = AssessmentOutcome::Abstained {
        assessment_id: "a2".into(),
        reason: AbstentionReason::ConstructNotPromoted,
    };
    let json = serde_json::to_string(&SafeDisplayProjection::try_from(&outcome).unwrap()).unwrap();
    assert!(json.contains("unavailable"));
    assert!(json.contains("ConstructNotPromoted"));
    assert!(!json.contains("workload"));
}
