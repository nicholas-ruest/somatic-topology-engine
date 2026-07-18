//! Deterministic respiration baseline and frozen-evaluation acceptance tests.

use serde::Deserialize;
use std::collections::BTreeSet;
use ste_physiology_estimation::estimator::*;
use ste_physiology_estimation::evaluation::*;

#[derive(Deserialize)]
struct KnownAnswer {
    package: RespirationModelPackage,
    evidence: RespirationObservationEvidence,
    expected_bpm: f64,
    expected_confidence: f64,
}

fn fixture() -> KnownAnswer {
    serde_json::from_str(include_str!("fixtures/respiration-known-answer.json")).unwrap()
}

#[test]
fn known_answer_and_repeat_parity_are_exact_and_package_bound() {
    let fixture = fixture();
    let first = estimate_respiration(&fixture.package, &fixture.evidence).unwrap();
    let second = estimate_respiration(&fixture.package, &fixture.evidence).unwrap();
    assert_eq!(first, second);
    let RespirationOutcome::Estimated(estimate) = first else {
        panic!("known answer must estimate");
    };
    assert_eq!(estimate.breaths_per_minute, fixture.expected_bpm);
    assert!((estimate.calibrated_confidence - fixture.expected_confidence).abs() < 1.0e-12);
    assert_eq!(estimate.package_digest, fixture.package.digest().unwrap());
    assert_eq!(estimate.source_artifact_ref, fixture.evidence.artifact_ref);
}

#[test]
fn every_quality_envelope_and_confidence_gate_abstains_without_a_value() {
    let base = fixture();
    let cases = [
        (
            RespirationObservationEvidence {
                contaminated: true,
                ..base.evidence.clone()
            },
            RespirationAbstentionReason::Contaminated,
        ),
        (
            RespirationObservationEvidence {
                missingness: 0.3,
                ..base.evidence.clone()
            },
            RespirationAbstentionReason::ExcessiveMissingness,
        ),
        (
            RespirationObservationEvidence {
                interference: 0.3,
                ..base.evidence.clone()
            },
            RespirationAbstentionReason::ExcessiveInterference,
        ),
        (
            RespirationObservationEvidence {
                periodicity: 0.2,
                ..base.evidence.clone()
            },
            RespirationAbstentionReason::WeakPeriodicity,
        ),
        (
            RespirationObservationEvidence {
                dominant_frequency_hz: Some(1.0),
                ..base.evidence.clone()
            },
            RespirationAbstentionReason::OutsideOperatingEnvelope,
        ),
        (
            RespirationObservationEvidence {
                dominant_frequency_hz: None,
                ..base.evidence.clone()
            },
            RespirationAbstentionReason::MissingEvidence,
        ),
        (
            RespirationObservationEvidence {
                dsp_graph_version: 2,
                ..base.evidence
            },
            RespirationAbstentionReason::DspVersionMismatch,
        ),
    ];
    for (evidence, expected) in cases {
        assert_eq!(
            estimate_respiration(&base.package, &evidence).unwrap(),
            RespirationOutcome::Abstained(expected)
        );
    }
}

#[test]
fn malformed_package_or_nonfinite_evidence_never_produces_an_estimate() {
    let mut invalid_package = fixture().package;
    invalid_package.confidence_calibration = vec![(0.0, 0.5), (1.0, 0.4)];
    assert_eq!(
        estimate_respiration(&invalid_package, &fixture().evidence),
        Err(EstimatorError::InvalidPackage)
    );
    let mut evidence = fixture().evidence;
    evidence.periodicity = f64::NAN;
    assert_eq!(
        estimate_respiration(&fixture().package, &evidence).unwrap(),
        RespirationOutcome::Abstained(RespirationAbstentionReason::MissingEvidence)
    );
}

fn estimated(bpm: f64, id: &str) -> RespirationOutcome {
    let fixture = fixture();
    let frequency = bpm / 60.0;
    estimate_respiration(
        &fixture.package,
        &RespirationObservationEvidence {
            artifact_ref: id.into(),
            dominant_frequency_hz: Some(frequency),
            ..fixture.evidence
        },
    )
    .unwrap()
}

fn gates() -> RespirationEvaluationGates {
    RespirationEvaluationGates {
        minimum_coverage: 0.75,
        maximum_mae_bpm: 1.0,
        maximum_rmse_bpm: 1.0,
        failure_rate_critical_value: 1.96,
    }
}

fn run(origin: EvidenceOrigin) -> FrozenRespirationRun {
    FrozenRespirationRun {
        run_id: "frozen-run-1".into(),
        frozen: true,
        split: EvaluationSplit::Validation,
        origin,
        training_sessions: BTreeSet::from(["train-session".into()]),
        training_days: BTreeSet::from(["train-day".into()]),
        pairs: vec![
            EvaluationPair {
                session_id: "heldout-a".into(),
                day_id: "day-a".into(),
                estimate: estimated(15.0, "a"),
                belt_bpm: Some(15.5),
                belt_reference_valid: true,
            },
            EvaluationPair {
                session_id: "heldout-b".into(),
                day_id: "day-b".into(),
                estimate: estimated(18.0, "b"),
                belt_bpm: Some(17.5),
                belt_reference_valid: true,
            },
        ],
    }
}

#[test]
fn synthetic_known_answers_preserve_disabled_negative_outcome_even_when_metrics_pass() {
    let report = evaluate_respiration_run(&run(EvidenceOrigin::Synthetic), gates());
    assert_eq!(report.paired, 2);
    assert!(report.agreement.is_some());
    assert_eq!(
        report.status,
        EvaluationStatus::Disabled(DisabledReason::SyntheticEvidenceOnly)
    );
}

#[test]
fn absent_real_belt_data_preserves_disabled_outcome_and_explicit_failures() {
    let mut run = run(EvidenceOrigin::HumanRespiratoryBelt);
    for pair in &mut run.pairs {
        pair.belt_bpm = None;
    }
    let report = evaluate_respiration_run(&run, gates());
    assert_eq!(report.paired, 0);
    assert_eq!(report.coverage, 0.0);
    assert_eq!(report.failure_rate.unwrap().failures, 2);
    assert_eq!(
        report.status,
        EvaluationStatus::Disabled(DisabledReason::NoRealBeltEvidence)
    );
}

#[test]
fn session_and_day_leakage_and_nonfrozen_or_training_runs_are_rejected() {
    let mut session_leak = run(EvidenceOrigin::HumanRespiratoryBelt);
    session_leak.pairs[0].session_id = "train-session".into();
    assert_eq!(
        evaluate_respiration_run(&session_leak, gates()).status,
        EvaluationStatus::Disabled(DisabledReason::HeldOutLeakage)
    );
    let mut day_leak = run(EvidenceOrigin::HumanRespiratoryBelt);
    day_leak.pairs[0].day_id = "train-day".into();
    assert_eq!(
        evaluate_respiration_run(&day_leak, gates()).status,
        EvaluationStatus::Disabled(DisabledReason::HeldOutLeakage)
    );
    let mut not_frozen = run(EvidenceOrigin::HumanRespiratoryBelt);
    not_frozen.frozen = false;
    assert_eq!(
        evaluate_respiration_run(&not_frozen, gates()).status,
        EvaluationStatus::Disabled(DisabledReason::NotFrozen)
    );
    let mut training = run(EvidenceOrigin::HumanRespiratoryBelt);
    training.split = EvaluationSplit::Training;
    assert_eq!(
        evaluate_respiration_run(&training, gates()).status,
        EvaluationStatus::Disabled(DisabledReason::WrongSplit)
    );
}

#[test]
fn frozen_heldout_real_belt_run_can_only_become_eligible_for_independent_review() {
    let report = evaluate_respiration_run(&run(EvidenceOrigin::HumanRespiratoryBelt), gates());
    assert_eq!(
        report.status,
        EvaluationStatus::EligibleForIndependentReview
    );
    assert_eq!(report.coverage, 1.0);
    let agreement = report.agreement.unwrap();
    assert_eq!(agreement.bias, 0.0);
    assert_eq!(agreement.mae, 0.5);
    assert_eq!(report.failure_rate.unwrap().rate, 0.0);
}
