//! Golden metrics and explicit reference synchronization acceptance tests.

use serde::Deserialize;
use ste_experiment_validation::metrics::*;
use ste_experiment_validation::reference::*;

#[test]
fn agreement_bias_absolute_error_rmse_and_limits_match_golden_values() {
    let result = agreement(&[(2.0, 1.0), (4.0, 2.0), (6.0, 3.0)]).unwrap();
    assert_eq!(result.count, 3);
    assert_eq!(result.bias, 2.0);
    assert_eq!(result.mae, 2.0);
    assert!((result.rmse - (14.0_f64 / 3.0).sqrt()).abs() < 1.0e-12);
    assert!((result.lower_limit_of_agreement - 0.04).abs() < 1.0e-12);
    assert!((result.upper_limit_of_agreement - 3.96).abs() < 1.0e-12);
}

#[test]
fn calibration_and_selective_risk_are_deterministic_and_keep_counts() {
    let samples = [(0.1, false), (0.2, false), (0.8, true), (0.9, true)];
    let first = calibration(&samples, 2).unwrap();
    let second = calibration(&samples, 2).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.count, 4);
    assert_eq!(first.bins.len(), 2);
    assert!((first.brier_score - 0.025).abs() < 1.0e-12);
    assert!((first.expected_calibration_error - 0.15).abs() < 1.0e-12);

    let curve = selective_risk_coverage(&[(0.2, 1.0), (0.9, 0.0), (0.8, 0.25)]).unwrap();
    assert_eq!(curve.len(), 3);
    assert_eq!(curve[0].confidence_threshold, 0.9);
    assert_eq!(curve[0].coverage, 1.0 / 3.0);
    assert_eq!(curve[0].selective_risk, 0.0);
    assert_eq!(curve[2].coverage, 1.0);
}

#[test]
fn confidence_failure_rate_and_baseline_comparison_are_finite_and_explicit() {
    let interval = mean_confidence_interval(&[1.0, 2.0, 3.0], 1.96).unwrap();
    assert_eq!(interval.estimate, 2.0);
    assert!(interval.lower < interval.estimate && interval.upper > interval.estimate);

    let failure = failure_rate(2, 10, 1.96).unwrap();
    assert_eq!(failure.rate, 0.2);
    assert!(failure.lower >= 0.0 && failure.upper <= 1.0);
    assert!(failure.lower < failure.rate && failure.upper > failure.rate);

    let lower = compare_baseline(10.0, 8.0, ImprovementDirection::LowerIsBetter).unwrap();
    assert_eq!(lower.improvement, 2.0);
    assert_eq!(lower.relative_improvement, Some(0.2));
    let zero = compare_baseline(0.0, 1.0, ImprovementDirection::HigherIsBetter).unwrap();
    assert_eq!(zero.relative_improvement, None);
}

#[test]
fn every_metric_rejects_nonfinite_or_invalid_inputs_without_silent_row_dropping() {
    assert_eq!(
        agreement(&[(1.0, f64::NAN), (2.0, 2.0)]),
        Err(MetricError::NonFiniteOrOutOfRange)
    );
    assert_eq!(
        calibration(&[(1.1, true)], 10),
        Err(MetricError::NonFiniteOrOutOfRange)
    );
    assert_eq!(
        selective_risk_coverage(&[(0.5, -1.0)]),
        Err(MetricError::NonFiniteOrOutOfRange)
    );
    assert_eq!(
        failure_rate(2, 1, 1.96),
        Err(MetricError::InvalidConfiguration)
    );
    assert_eq!(
        agreement(&[(f64::MAX, -f64::MAX), (f64::MAX, -f64::MAX)]),
        Err(MetricError::NonFiniteOrOutOfRange)
    );
}

fn reference_samples() -> Vec<ReferenceSample> {
    vec![
        ReferenceSample::new(100, 1.0, 0.99).unwrap(),
        ReferenceSample::new(200, 2.0, 0.98).unwrap(),
        ReferenceSample::new(300, 3.0, 0.97).unwrap(),
    ]
}

#[test]
fn exact_alignment_never_interpolates_or_hides_missing_targets() {
    let belt = RespiratoryBeltAdapter::new("belt-1", reference_samples()).unwrap();
    let artifact = synchronize(
        &belt,
        &[100, 150, 300],
        AlignmentMethod::ExactOnly,
        1_000,
        0.9,
    )
    .unwrap();
    assert_eq!(artifact.alignments[0].value, Some(1.0));
    assert_eq!(artifact.alignments[0].alignment_uncertainty_ns, Some(0));
    assert_eq!(artifact.alignments[1].value, None);
    assert_eq!(artifact.alignments[1].method, None);
    assert!(artifact.alignments[1].source_timestamps_ns.is_empty());
}

#[test]
fn nearest_and_linear_alignment_record_sources_uncertainty_quality_and_method() {
    let ppg = ValidatedPpgAdapter::new("ppg-validated", reference_samples()).unwrap();
    let nearest = synchronize(&ppg, &[150], AlignmentMethod::Nearest, 60, 0.9).unwrap();
    assert_eq!(nearest.alignments[0].value, Some(1.0));
    assert_eq!(nearest.alignments[0].source_timestamps_ns, vec![100]);
    assert_eq!(nearest.alignments[0].alignment_uncertainty_ns, Some(50));
    assert_eq!(nearest.alignments[0].method, Some(AlignmentMethod::Nearest));

    let linear = synchronize(&ppg, &[150], AlignmentMethod::Linear, 60, 0.9).unwrap();
    assert_eq!(linear.alignments[0].value, Some(1.5));
    assert_eq!(linear.alignments[0].source_timestamps_ns, vec![100, 200]);
    assert_eq!(linear.alignments[0].reference_quality, Some(0.98));
    assert_eq!(linear.alignments[0].method, Some(AlignmentMethod::Linear));

    let missing = synchronize(&ppg, &[450], AlignmentMethod::Nearest, 60, 0.9).unwrap();
    assert_eq!(missing.alignments[0].value, None);
}

#[test]
fn all_reference_ports_have_explicit_modality_and_validated_ordering() {
    let samples = reference_samples();
    let sensors: Vec<Box<dyn ReferenceSensor>> = vec![
        Box::new(RespiratoryBeltAdapter::new("belt", samples.clone()).unwrap()),
        Box::new(EcgAdapter::new("ecg", samples.clone()).unwrap()),
        Box::new(ValidatedPpgAdapter::new("ppg", samples.clone()).unwrap()),
        Box::new(TaskTimestampAdapter::new("task", samples.clone()).unwrap()),
        Box::new(SelfReportTimestampAdapter::new("self", samples).unwrap()),
    ];
    assert_eq!(
        sensors
            .iter()
            .map(|sensor| sensor.kind())
            .collect::<Vec<_>>(),
        vec![
            ReferenceKind::RespiratoryBelt,
            ReferenceKind::Ecg,
            ReferenceKind::ValidatedPpg,
            ReferenceKind::TaskTimestamp,
            ReferenceKind::SelfReportTimestamp
        ]
    );
    let unordered = vec![
        ReferenceSample::new(200, 1.0, 1.0).unwrap(),
        ReferenceSample::new(100, 1.0, 1.0).unwrap(),
    ];
    assert!(EcgAdapter::new("ecg", unordered).is_err());
}

#[derive(Deserialize)]
struct RigFixture {
    source_id: String,
    samples: Vec<ReferenceSample>,
    targets_ns: Vec<u64>,
    maximum_uncertainty_ns: u64,
}

#[test]
fn simulated_rig_golden_is_reproducible_and_serializes_alignment_evidence() {
    let fixture: RigFixture =
        serde_json::from_str(include_str!("fixtures/simulated-reference-rig.json")).unwrap();
    let belt = RespiratoryBeltAdapter::new(fixture.source_id, fixture.samples).unwrap();
    let first = synchronize(
        &belt,
        &fixture.targets_ns,
        AlignmentMethod::Linear,
        fixture.maximum_uncertainty_ns,
        0.9,
    )
    .unwrap();
    let second = synchronize(
        &belt,
        &fixture.targets_ns,
        AlignmentMethod::Linear,
        fixture.maximum_uncertainty_ns,
        0.9,
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.alignments[0].value, Some(1.0));
    assert_eq!(first.alignments[1].value, Some(1.5));
    assert_eq!(first.alignments[2].value, Some(2.5));
    assert_eq!(first.alignments[3].value, None);
    let json = serde_json::to_string(&first).unwrap();
    assert!(json.contains("alignment_uncertainty_ns"));
    assert!(json.contains("requested_method"));
}
