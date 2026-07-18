//! Calibration, OOD, selective-risk, and signed capability-policy adversarial tests.

use ed25519_dalek::SigningKey;
use ste_model_runtime::capability::*;
use ste_model_runtime::uncertainty::*;

fn calibration() -> CalibrationArtifact {
    serde_json::from_str(include_str!("fixtures/adversarial-calibration.json")).unwrap()
}

fn scope() -> OperatingScope {
    OperatingScope {
        hardware_profiles: vec!["hardware-a".into()],
        room_profiles: vec!["room-a".into()],
        tasks: vec!["task-a".into()],
        postures: vec!["seated".into()],
        jurisdictions: vec!["CA".into()],
    }
}

fn ood() -> OodArtifact {
    OodArtifact {
        ood_id: "ood-v1".into(),
        model_digest: "model-sha256".into(),
        means: vec![0.0, 1.0],
        standard_deviations: vec![1.0, 2.0],
        maximum_z_score: 3.0,
        maximum_missingness: 0.1,
        maximum_interference: 0.2,
        scope: scope(),
        frozen: true,
    }
}

fn evidence() -> InferenceEvidence {
    InferenceEvidence {
        hardware_profile: "hardware-a".into(),
        room_profile: "room-a".into(),
        task: "task-a".into(),
        posture: "seated".into(),
        jurisdiction: "CA".into(),
        features: vec![0.5, 2.0],
        missingness: 0.0,
        interference: 0.0,
    }
}

#[test]
fn calibration_is_frozen_separate_monotonic_and_known_answer_deterministic() {
    let artifact = calibration();
    assert_eq!(artifact.calibrate(0.75).unwrap().get(), 0.7);
    assert_eq!(artifact.calibrate(0.75), artifact.calibrate(0.75));

    let mut leaking = artifact.clone();
    leaking.calibration_partition_digest = leaking.training_partition_digest.clone();
    assert_eq!(
        leaking.validate(),
        Err(UncertaintyError::InvalidCalibration)
    );
    let mut adaptive = artifact.clone();
    adaptive.frozen = false;
    assert_eq!(
        adaptive.validate(),
        Err(UncertaintyError::InvalidCalibration)
    );
    let mut non_monotonic = artifact;
    non_monotonic.knots[1].1 = 0.9;
    non_monotonic.knots[2].1 = 0.8;
    assert_eq!(
        non_monotonic.validate(),
        Err(UncertaintyError::InvalidCalibration)
    );
}

#[test]
fn every_scope_quality_shape_and_distribution_violation_is_explicit_ood() {
    let artifact = ood();
    assert_eq!(
        artifact.evaluate(&evidence()).unwrap(),
        OodDecision::InDistribution
    );
    let cases = [
        (
            InferenceEvidence {
                hardware_profile: "other".into(),
                ..evidence()
            },
            OodReason::Hardware,
        ),
        (
            InferenceEvidence {
                room_profile: "other".into(),
                ..evidence()
            },
            OodReason::Room,
        ),
        (
            InferenceEvidence {
                task: "other".into(),
                ..evidence()
            },
            OodReason::Task,
        ),
        (
            InferenceEvidence {
                posture: "standing".into(),
                ..evidence()
            },
            OodReason::Posture,
        ),
        (
            InferenceEvidence {
                jurisdiction: "US".into(),
                ..evidence()
            },
            OodReason::Jurisdiction,
        ),
        (
            InferenceEvidence {
                features: vec![0.0],
                ..evidence()
            },
            OodReason::FeatureShape,
        ),
        (
            InferenceEvidence {
                features: vec![4.0, 1.0],
                ..evidence()
            },
            OodReason::FeatureDistribution,
        ),
        (
            InferenceEvidence {
                missingness: 0.2,
                ..evidence()
            },
            OodReason::Missingness,
        ),
        (
            InferenceEvidence {
                missingness: -0.1,
                ..evidence()
            },
            OodReason::Missingness,
        ),
        (
            InferenceEvidence {
                interference: 0.3,
                ..evidence()
            },
            OodReason::Interference,
        ),
        (
            InferenceEvidence {
                interference: -0.1,
                ..evidence()
            },
            OodReason::Interference,
        ),
    ];
    for (input, reason) in cases {
        assert_eq!(
            artifact.evaluate(&input).unwrap(),
            OodDecision::OutOfDistribution(reason)
        );
    }
}

#[test]
fn uncertainty_gate_rejects_mismatch_ood_invalid_score_and_low_confidence() {
    assert_eq!(
        evaluate_uncertainty("model-sha256", 0.75, &calibration(), &ood(), &evidence()),
        UncertaintyDecision::Serve(CalibratedProbability::new(0.7).unwrap())
    );
    assert_eq!(
        evaluate_uncertainty("other-model", 0.9, &calibration(), &ood(), &evidence()),
        UncertaintyDecision::Abstain(UncertaintyAbstention::CalibrationInvalid)
    );
    assert_eq!(
        evaluate_uncertainty(
            "model-sha256",
            0.9,
            &calibration(),
            &ood(),
            &InferenceEvidence {
                room_profile: "other".into(),
                ..evidence()
            },
        ),
        UncertaintyDecision::Abstain(UncertaintyAbstention::Ood(OodReason::Room))
    );
    assert_eq!(
        evaluate_uncertainty("model-sha256", 0.5, &calibration(), &ood(), &evidence()),
        UncertaintyDecision::Abstain(UncertaintyAbstention::InsufficientConfidence)
    );
    assert_eq!(
        evaluate_uncertainty(
            "model-sha256",
            f64::NAN,
            &calibration(),
            &ood(),
            &evidence()
        ),
        UncertaintyDecision::Abstain(UncertaintyAbstention::CalibrationInvalid)
    );
}

#[test]
fn selective_risk_coverage_is_sorted_deterministic_and_finite() {
    let values = [
        (CalibratedProbability::new(0.2).unwrap(), 1.0),
        (CalibratedProbability::new(0.9).unwrap(), 0.0),
        (CalibratedProbability::new(0.8).unwrap(), 0.25),
    ];
    let curve = selective_risk_coverage(&values).unwrap();
    assert_eq!(curve[0].threshold, 0.9);
    assert_eq!(curve[0].coverage, 1.0 / 3.0);
    assert_eq!(curve[0].risk, 0.0);
    assert_eq!(curve.last().unwrap().coverage, 1.0);
    assert_eq!(
        selective_risk_coverage(&[(CalibratedProbability::new(0.5).unwrap(), f64::INFINITY)]),
        Err(UncertaintyError::InvalidSelectiveRisk)
    );
}

fn policy(mode: CapabilityMode, promotion_level: PromotionLevel) -> CapabilityPolicy {
    CapabilityPolicy {
        policy_id: "policy-1".into(),
        capability_id: "task-workload-v1".into(),
        mode,
        software_digest: "software-sha256".into(),
        model_digest: "model-sha256".into(),
        hardware_profile_digest: "hardware-sha256".into(),
        operating_envelope_digest: "envelope-sha256".into(),
        promotion_digest: "promotion-sha256".into(),
        promotion_level,
        purpose: CapabilityPurpose::Wellness,
        jurisdiction: "CA".into(),
        deployment_id: "device-profile-1".into(),
        not_before_unix_seconds: 100,
        expires_at_unix_seconds: 200,
        enabled: true,
    }
}

fn context(promotion_level: PromotionLevel) -> CapabilityContext {
    CapabilityContext {
        capability_id: "task-workload-v1".into(),
        software_digest: "software-sha256".into(),
        model_digest: "model-sha256".into(),
        model_active_and_not_revoked: true,
        hardware_profile_digest: "hardware-sha256".into(),
        operating_envelope_digest: "envelope-sha256".into(),
        promotion_digest: "promotion-sha256".into(),
        promotion_level,
        promotion_active: true,
        purpose: CapabilityPurpose::Wellness,
        jurisdiction: "CA".into(),
        deployment_id: "device-profile-1".into(),
        evaluated_at_unix_seconds: 150,
        evidence_in_scope: true,
    }
}

fn evaluator_and_key() -> (CapabilityPolicyEvaluator, SigningKey) {
    let signing = SigningKey::from_bytes(&[7_u8; 32]);
    let evaluator =
        CapabilityPolicyEvaluator::new([("release-key-1".into(), signing.verifying_key())])
            .unwrap();
    (evaluator, signing)
}

#[test]
fn exact_signed_production_policy_grants_only_matching_promoted_in_scope_context() {
    let (evaluator, signing) = evaluator_and_key();
    let signed = SignedCapabilityPolicy::sign(
        policy(CapabilityMode::Production, PromotionLevel::Production),
        "release-key-1",
        &signing,
    )
    .unwrap();
    assert_eq!(
        evaluator.evaluate(&signed, &context(PromotionLevel::Production)),
        Ok(CapabilityGrant::Production {
            capability_id: "task-workload-v1".into()
        })
    );

    let mismatch_cases = [
        (
            CapabilityContext {
                software_digest: "other".into(),
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::SoftwareMismatch,
        ),
        (
            CapabilityContext {
                model_digest: "other".into(),
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::ModelMismatch,
        ),
        (
            CapabilityContext {
                hardware_profile_digest: "other".into(),
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::HardwareMismatch,
        ),
        (
            CapabilityContext {
                operating_envelope_digest: "other".into(),
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::EnvelopeMismatch,
        ),
        (
            CapabilityContext {
                purpose: CapabilityPurpose::Research,
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::PurposeMismatch,
        ),
        (
            CapabilityContext {
                jurisdiction: "US".into(),
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::JurisdictionMismatch,
        ),
        (
            CapabilityContext {
                deployment_id: "other".into(),
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::DeploymentMismatch,
        ),
        (
            CapabilityContext {
                promotion_active: false,
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::PromotionMismatch,
        ),
        (
            CapabilityContext {
                evidence_in_scope: false,
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::EvidenceOutOfScope,
        ),
        (
            CapabilityContext {
                model_active_and_not_revoked: false,
                ..context(PromotionLevel::Production)
            },
            CapabilityDenial::ModelUnavailable,
        ),
    ];
    for (runtime, denial) in mismatch_cases {
        assert_eq!(evaluator.evaluate(&signed, &runtime), Err(denial));
    }
}

#[test]
fn tampered_disabled_expired_unknown_and_experimental_policies_fail_or_isolate() {
    let (evaluator, signing) = evaluator_and_key();
    let mut tampered = SignedCapabilityPolicy::sign(
        policy(CapabilityMode::Production, PromotionLevel::Production),
        "release-key-1",
        &signing,
    )
    .unwrap();
    tampered.policy.model_digest = "attacker-model".into();
    assert_eq!(
        evaluator.evaluate(&tampered, &context(PromotionLevel::Production)),
        Err(CapabilityDenial::SignatureInvalid)
    );
    let mut disabled_policy = policy(CapabilityMode::Production, PromotionLevel::Production);
    disabled_policy.enabled = false;
    let disabled =
        SignedCapabilityPolicy::sign(disabled_policy, "release-key-1", &signing).unwrap();
    assert_eq!(
        evaluator.evaluate(&disabled, &context(PromotionLevel::Production)),
        Err(CapabilityDenial::Disabled)
    );
    let mut expired_context = context(PromotionLevel::Production);
    expired_context.evaluated_at_unix_seconds = 200;
    let valid = SignedCapabilityPolicy::sign(
        policy(CapabilityMode::Production, PromotionLevel::Production),
        "release-key-1",
        &signing,
    )
    .unwrap();
    assert_eq!(
        evaluator.evaluate(&valid, &expired_context),
        Err(CapabilityDenial::OutsideValidity)
    );
    let mut unknown = valid.clone();
    unknown.signer_key_id = "unknown".into();
    assert_eq!(
        evaluator.evaluate(&unknown, &context(PromotionLevel::Production)),
        Err(CapabilityDenial::SignatureInvalid)
    );

    let experimental = SignedCapabilityPolicy::sign(
        policy(CapabilityMode::Experimental, PromotionLevel::Experimental),
        "release-key-1",
        &signing,
    )
    .unwrap();
    let CapabilityGrant::Experimental(isolation) = evaluator
        .evaluate(&experimental, &context(PromotionLevel::Experimental))
        .unwrap()
    else {
        panic!("must isolate experimental output")
    };
    assert!(isolation.namespace.starts_with("experimental/"));
    assert!(isolation.visible_label.contains("EXPERIMENTAL"));
    assert!(!isolation.production_projection_allowed);

    let invalid_production = SignedCapabilityPolicy::sign(
        policy(CapabilityMode::Production, PromotionLevel::Experimental),
        "release-key-1",
        &signing,
    )
    .unwrap();
    assert_eq!(
        evaluator.evaluate(&invalid_production, &context(PromotionLevel::Experimental)),
        Err(CapabilityDenial::ExperimentalPromotionCannotServe)
    );
}
