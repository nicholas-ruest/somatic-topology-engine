//! Temporal replay and complete application-gate adversarial tests.

use ed25519_dalek::SigningKey;
use ste_model_runtime::capability::*;
use ste_model_runtime::uncertainty::*;
use ste_state_inference::domain::*;
use ste_state_inference::temporal::*;
use ste_state_inference::validation::*;

fn temporal_policy() -> TemporalPolicy {
    TemporalPolicy {
        version: 1,
        required_consecutive: 3,
        minimum_dwell_ns: 20,
        maximum_gap_ns: 15,
        maximum_evidence_age_ns: 5,
    }
}

fn candidate(index: u64, state: &str) -> TemporalCandidate {
    TemporalCandidate {
        assessment_id: format!("assessment-{index}"),
        event_time_ns: index * 10,
        state_id: state.into(),
    }
}

#[test]
fn isolated_spikes_do_not_transition_and_replay_is_exact() {
    let inputs = vec![
        candidate(1, "moderate"),
        candidate(2, "moderate"),
        candidate(3, "elevated"),
        candidate(4, "moderate"),
        candidate(5, "moderate"),
        candidate(6, "moderate"),
    ];
    let first = replay_temporal(temporal_policy(), &inputs, 1).unwrap();
    let second = replay_temporal(temporal_policy(), &inputs, 1).unwrap();
    assert_eq!(first, second);
    assert!(
        first[..5]
            .iter()
            .all(|outcome| matches!(outcome, TemporalOutcome::Held { .. }))
    );
    assert!(matches!(
        first[5],
        TemporalOutcome::Transitioned { ref to, .. } if to == "moderate"
    ));
}

#[test]
fn gaps_reset_partial_runs_and_stale_or_reordered_evidence_fails_without_mutation() {
    let mut debouncer = TemporalDebouncer::new(temporal_policy()).unwrap();
    assert!(matches!(
        debouncer.apply(candidate(1, "moderate"), 11).unwrap(),
        TemporalOutcome::Held {
            candidate_count: 1,
            ..
        }
    ));
    let gap = TemporalCandidate {
        assessment_id: "after-gap".into(),
        event_time_ns: 30,
        state_id: "moderate".into(),
    };
    assert!(matches!(
        debouncer.apply(gap, 31).unwrap(),
        TemporalOutcome::Held {
            candidate_count: 1,
            ..
        }
    ));
    let stale = TemporalCandidate {
        assessment_id: "stale".into(),
        event_time_ns: 40,
        state_id: "moderate".into(),
    };
    assert_eq!(
        debouncer.apply(stale, 46),
        Err(TemporalError::StaleEvidence)
    );
    let reordered = TemporalCandidate {
        assessment_id: "reordered".into(),
        event_time_ns: 29,
        state_id: "moderate".into(),
    };
    assert_eq!(
        debouncer.apply(reordered, 30),
        Err(TemporalError::NonMonotonicEventTime)
    );
    assert_eq!(debouncer.current_state(), None);
}

#[derive(Clone)]
struct Hooks {
    promoted: bool,
    physiology: bool,
    profile: bool,
    envelope: bool,
    unavailable: bool,
}

impl InferenceValidationHooks for Hooks {
    type Error = ();
    fn construct_is_promoted(&self, _: &str) -> Result<bool, Self::Error> {
        if self.unavailable {
            Err(())
        } else {
            Ok(self.promoted)
        }
    }
    fn physiology_is_valid(&self, _: &str) -> Result<bool, Self::Error> {
        if self.unavailable {
            Err(())
        } else {
            Ok(self.physiology)
        }
    }
    fn profile_is_valid(&self, _: &str, _: &str) -> Result<bool, Self::Error> {
        if self.unavailable {
            Err(())
        } else {
            Ok(self.profile)
        }
    }
    fn operating_envelope_holds(&self, _: &ModelScope) -> Result<bool, Self::Error> {
        if self.unavailable {
            Err(())
        } else {
            Ok(self.envelope)
        }
    }
}

struct Artifacts {
    calibration: CalibrationArtifact,
    ood: OodArtifact,
    inference: InferenceEvidence,
    signed: SignedCapabilityPolicy,
    evaluator: CapabilityPolicyEvaluator,
    context: CapabilityContext,
}

fn artifacts(mode: CapabilityMode, promotion: PromotionLevel) -> Artifacts {
    let calibration = CalibrationArtifact {
        calibration_id: "cal-1".into(),
        model_digest: "model-digest".into(),
        training_partition_digest: "train".into(),
        calibration_partition_digest: "calibration".into(),
        knots: vec![(0.0, 0.0), (1.0, 1.0)],
        serving_threshold: 0.7,
        brier_score: 0.1,
        expected_calibration_error: 0.05,
        frozen: true,
    };
    let scope = OperatingScope {
        hardware_profiles: vec!["hardware-digest".into()],
        room_profiles: vec!["room-a".into()],
        tasks: vec!["n-back".into()],
        postures: vec!["seated".into()],
        jurisdictions: vec!["CA".into()],
    };
    let ood = OodArtifact {
        ood_id: "ood-1".into(),
        model_digest: "model-digest".into(),
        means: vec![0.0],
        standard_deviations: vec![1.0],
        maximum_z_score: 3.0,
        maximum_missingness: 0.1,
        maximum_interference: 0.1,
        scope,
        frozen: true,
    };
    let inference = InferenceEvidence {
        hardware_profile: "hardware-digest".into(),
        room_profile: "room-a".into(),
        task: "n-back".into(),
        posture: "seated".into(),
        jurisdiction: "CA".into(),
        features: vec![0.0],
        missingness: 0.0,
        interference: 0.0,
    };
    let signing = SigningKey::from_bytes(&[9_u8; 32]);
    let policy = CapabilityPolicy {
        policy_id: "policy-1".into(),
        capability_id: "task-workload".into(),
        mode,
        software_digest: "software".into(),
        model_digest: "model-digest".into(),
        hardware_profile_digest: "hardware-digest".into(),
        operating_envelope_digest: "envelope".into(),
        promotion_digest: "promotion".into(),
        promotion_level: promotion,
        purpose: CapabilityPurpose::Wellness,
        jurisdiction: "CA".into(),
        deployment_id: "deployment".into(),
        not_before_unix_seconds: 1,
        expires_at_unix_seconds: 100,
        enabled: true,
    };
    let signed = SignedCapabilityPolicy::sign(policy, "release", &signing).unwrap();
    let evaluator =
        CapabilityPolicyEvaluator::new([("release".into(), signing.verifying_key())]).unwrap();
    let context = CapabilityContext {
        capability_id: "task-workload".into(),
        software_digest: "software".into(),
        model_digest: "model-digest".into(),
        model_active_and_not_revoked: true,
        hardware_profile_digest: "hardware-digest".into(),
        operating_envelope_digest: "envelope".into(),
        promotion_digest: "promotion".into(),
        promotion_level: promotion,
        promotion_active: true,
        purpose: CapabilityPurpose::Wellness,
        jurisdiction: "CA".into(),
        deployment_id: "deployment".into(),
        evaluated_at_unix_seconds: 50,
        evidence_in_scope: true,
    };
    Artifacts {
        calibration,
        ood,
        inference,
        signed,
        evaluator,
        context,
    }
}

fn evidence() -> EvidenceBundle {
    EvidenceBundle::new(
        "observation-artifact-1",
        "physiology-assessment-1",
        30_000,
        Provenance::new("model-v1", "cal-1", "profile-v1").unwrap(),
    )
    .unwrap()
}

fn scope_domain() -> ModelScope {
    ModelScope::new(
        "participant-a",
        "room-a",
        "n-back",
        "seated",
        "hardware-digest",
    )
    .unwrap()
}

fn request(artifacts: &Artifacts, construct: ConstructDefinition) -> ValidatedInferenceRequest<'_> {
    ValidatedInferenceRequest {
        assessment_id: "assessment-1".into(),
        construct,
        evidence: evidence(),
        scope: scope_domain(),
        model_digest: "model-digest",
        raw_score: 0.8,
        calibration: &artifacts.calibration,
        ood: &artifacts.ood,
        inference_evidence: &artifacts.inference,
        capability_policy: &artifacts.signed,
        capability_context: &artifacts.context,
    }
}

fn passing_hooks() -> Hooks {
    Hooks {
        promoted: true,
        physiology: true,
        profile: true,
        envelope: true,
        unavailable: false,
    }
}

#[test]
fn production_assessment_preserves_distinct_observation_and_physiology_trace() {
    let artifacts = artifacts(CapabilityMode::Production, PromotionLevel::Production);
    let hooks = passing_hooks();
    let service = ValidatedInferenceService::new(
        &hooks,
        &artifacts.evaluator,
        ClaimPolicy {
            task_specific_workload_enabled: true,
        },
    );
    let outcome = service.assess(request(
        &artifacts,
        ConstructDefinition::task_workload("n-back", 1).unwrap(),
    ));
    let ValidatedInferenceOutcome::Production(AssessmentOutcome::Estimated(assessment)) = outcome
    else {
        panic!("must estimate")
    };
    assert_eq!(
        assessment.evidence().observation_id,
        "observation-artifact-1"
    );
    assert_eq!(
        assessment.evidence().physiology_id,
        "physiology-assessment-1"
    );
    assert_eq!(assessment.evidence().provenance.model_version, "model-v1");
    assert_eq!(assessment.scope().task, "n-back");
    assert_eq!(assessment.probability().get(), 0.8);
    let projection = DisplayProjectionV1::try_from(&assessment).unwrap();
    assert!(!projection.claim.contains("0.8"));
}

#[test]
fn baseline_noop_preserves_trace_but_bypasses_models_and_can_never_claim() {
    let artifacts = artifacts(CapabilityMode::Production, PromotionLevel::Production);
    let hooks = Hooks {
        unavailable: true,
        ..passing_hooks()
    };
    let service =
        ValidatedInferenceService::new(&hooks, &artifacts.evaluator, ClaimPolicy::default());
    let outcome = service.assess(request(&artifacts, ConstructDefinition::baseline()));
    let ValidatedInferenceOutcome::Baseline(trace) = outcome else {
        panic!("must be baseline")
    };
    assert_eq!(trace.evidence.observation_id, "observation-artifact-1");
    assert_eq!(trace.evidence.physiology_id, "physiology-assessment-1");
    assert!(!trace.claim_emitted);
}

#[test]
fn validation_claim_scope_ood_and_policy_failures_all_abstain() {
    let mut scenarios = Vec::new();
    let artifacts = artifacts(CapabilityMode::Production, PromotionLevel::Production);
    scenarios.push((
        Hooks {
            promoted: false,
            ..passing_hooks()
        },
        ValidationAbstention::ConstructNotPromoted,
    ));
    scenarios.push((
        Hooks {
            physiology: false,
            ..passing_hooks()
        },
        ValidationAbstention::InvalidPhysiology,
    ));
    scenarios.push((
        Hooks {
            profile: false,
            ..passing_hooks()
        },
        ValidationAbstention::InvalidProfile,
    ));
    scenarios.push((
        Hooks {
            envelope: false,
            ..passing_hooks()
        },
        ValidationAbstention::OutsideOperatingEnvelope,
    ));
    scenarios.push((
        Hooks {
            unavailable: true,
            ..passing_hooks()
        },
        ValidationAbstention::HookUnavailable,
    ));
    for (hooks, expected) in scenarios {
        let service = ValidatedInferenceService::new(
            &hooks,
            &artifacts.evaluator,
            ClaimPolicy {
                task_specific_workload_enabled: true,
            },
        );
        assert!(
            matches!(service.assess(request(&artifacts, ConstructDefinition::task_workload("n-back", 1).unwrap())), ValidatedInferenceOutcome::Abstained { reason, .. } if reason == expected)
        );
    }
    let hooks = passing_hooks();
    let service =
        ValidatedInferenceService::new(&hooks, &artifacts.evaluator, ClaimPolicy::default());
    assert!(matches!(
        service.assess(request(
            &artifacts,
            ConstructDefinition::task_workload("n-back", 1).unwrap()
        )),
        ValidatedInferenceOutcome::Abstained {
            reason: ValidationAbstention::ClaimPolicyDisabled,
            ..
        }
    ));
}

#[test]
fn ood_and_experimental_outputs_never_enter_production_assessment_type() {
    let mut out_of_scope = artifacts(CapabilityMode::Production, PromotionLevel::Production);
    out_of_scope.inference.room_profile = "unknown-room".into();
    let hooks = passing_hooks();
    let service = ValidatedInferenceService::new(
        &hooks,
        &out_of_scope.evaluator,
        ClaimPolicy {
            task_specific_workload_enabled: true,
        },
    );
    assert!(matches!(
        service.assess(request(
            &out_of_scope,
            ConstructDefinition::task_workload("n-back", 1).unwrap()
        )),
        ValidatedInferenceOutcome::Abstained {
            reason: ValidationAbstention::Uncertainty(UncertaintyAbstention::Ood(OodReason::Room)),
            ..
        }
    ));

    let experimental = artifacts(CapabilityMode::Experimental, PromotionLevel::Experimental);
    let service = ValidatedInferenceService::new(
        &hooks,
        &experimental.evaluator,
        ClaimPolicy {
            task_specific_workload_enabled: true,
        },
    );
    let ValidatedInferenceOutcome::Experimental(output) = service.assess(request(
        &experimental,
        ConstructDefinition::task_workload("n-back", 1).unwrap(),
    )) else {
        panic!("must isolate")
    };
    assert!(!output.isolation.production_projection_allowed);
    assert!(output.isolation.namespace.starts_with("experimental/"));
    assert_eq!(output.evidence.physiology_id, "physiology-assessment-1");
}
