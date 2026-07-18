//! Frozen calibration, operating-scope, OOD, and selective-risk primitives.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// A finite probability proven to lie in `[0,1]`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct CalibratedProbability(f64);

impl CalibratedProbability {
    /// Creates a checked probability.
    pub fn new(value: f64) -> Result<Self, UncertaintyError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(UncertaintyError::InvalidProbability)
        }
    }

    /// Returns the finite probability.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// Frozen piecewise-linear calibration artifact from a held-out partition.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CalibrationArtifact {
    /// Artifact identifier.
    pub calibration_id: String,
    /// Exact model-package digest.
    pub model_digest: String,
    /// Partition used to fit the model.
    pub training_partition_digest: String,
    /// Separate partition used only for calibration.
    pub calibration_partition_digest: String,
    /// Versioned monotonic `(raw, calibrated)` knots.
    pub knots: Vec<(f64, f64)>,
    /// Minimum calibrated confidence selected before held-out evaluation.
    pub serving_threshold: f64,
    /// Reported Brier score on the calibration partition.
    pub brier_score: f64,
    /// Reported expected calibration error.
    pub expected_calibration_error: f64,
    /// Artifact inputs and thresholds are immutable.
    pub frozen: bool,
}

impl CalibrationArtifact {
    /// Validates provenance separation, monotonicity, finiteness, and freezing.
    pub fn validate(&self) -> Result<(), UncertaintyError> {
        let probability = |value: f64| value.is_finite() && (0.0..=1.0).contains(&value);
        if !self.frozen
            || self.calibration_id.trim().is_empty()
            || self.model_digest.trim().is_empty()
            || self.training_partition_digest.trim().is_empty()
            || self.calibration_partition_digest.trim().is_empty()
            || self.training_partition_digest == self.calibration_partition_digest
            || self.knots.len() < 2
            || self.knots.first().map(|knot| knot.0) != Some(0.0)
            || self.knots.last().map(|knot| knot.0) != Some(1.0)
            || self
                .knots
                .iter()
                .any(|(raw, calibrated)| !probability(*raw) || !probability(*calibrated))
            || self
                .knots
                .windows(2)
                .any(|pair| pair[1].0 <= pair[0].0 || pair[1].1 < pair[0].1)
            || !probability(self.serving_threshold)
            || !self.brier_score.is_finite()
            || self.brier_score < 0.0
            || !probability(self.expected_calibration_error)
        {
            return Err(UncertaintyError::InvalidCalibration);
        }
        Ok(())
    }

    /// Calibrates one raw score and binds it to this immutable artifact.
    pub fn calibrate(&self, raw_score: f64) -> Result<CalibratedProbability, UncertaintyError> {
        self.validate()?;
        if !raw_score.is_finite() || !(0.0..=1.0).contains(&raw_score) {
            return Err(UncertaintyError::InvalidProbability);
        }
        for pair in self.knots.windows(2) {
            if raw_score <= pair[1].0 {
                let fraction = (raw_score - pair[0].0) / (pair[1].0 - pair[0].0);
                return CalibratedProbability::new(pair[0].1 + fraction * (pair[1].1 - pair[0].1));
            }
        }
        CalibratedProbability::new(self.knots.last().map_or(0.0, |knot| knot.1))
    }
}

/// Exact runtime conditions covered by the model card and validation evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OperatingScope {
    /// Allowed hardware-profile digests.
    pub hardware_profiles: Vec<String>,
    /// Allowed room/site profile identifiers.
    pub room_profiles: Vec<String>,
    /// Allowed task identifiers.
    pub tasks: Vec<String>,
    /// Allowed posture identifiers.
    pub postures: Vec<String>,
    /// Allowed jurisdiction identifiers.
    pub jurisdictions: Vec<String>,
}

/// Current explicit scope and quality evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct InferenceEvidence {
    /// Exact hardware-profile digest.
    pub hardware_profile: String,
    /// Exact room/site profile.
    pub room_profile: String,
    /// Exact task identifier.
    pub task: String,
    /// Exact posture identifier.
    pub posture: String,
    /// Current jurisdiction.
    pub jurisdiction: String,
    /// Finite feature vector in package order.
    pub features: Vec<f64>,
    /// Missing-input fraction.
    pub missingness: f64,
    /// Interference/contamination score.
    pub interference: f64,
}

/// Frozen diagonal feature-distribution and quality thresholds.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OodArtifact {
    /// Artifact identifier.
    pub ood_id: String,
    /// Bound model-package digest.
    pub model_digest: String,
    /// Feature means in package order.
    pub means: Vec<f64>,
    /// Strictly positive feature standard deviations.
    pub standard_deviations: Vec<f64>,
    /// Maximum absolute standardized deviation.
    pub maximum_z_score: f64,
    /// Maximum missingness.
    pub maximum_missingness: f64,
    /// Maximum interference.
    pub maximum_interference: f64,
    /// Model-card scope.
    pub scope: OperatingScope,
    /// Artifact is immutable.
    pub frozen: bool,
}

/// Explicit OOD or scope failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OodReason {
    /// Runtime hardware is outside validation scope.
    Hardware,
    /// Runtime room/site is outside validation scope.
    Room,
    /// Runtime task is outside validation scope.
    Task,
    /// Runtime posture is outside validation scope.
    Posture,
    /// Runtime jurisdiction is outside validation scope.
    Jurisdiction,
    /// Feature vector shape differs from the package.
    FeatureShape,
    /// Feature is non-finite or beyond the standardized-distance threshold.
    FeatureDistribution,
    /// Missingness exceeds the frozen threshold.
    Missingness,
    /// Interference exceeds the frozen threshold.
    Interference,
}

/// OOD result used by the serving gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OodDecision {
    /// Evidence lies inside every frozen check.
    InDistribution,
    /// Evidence failed a stable check.
    OutOfDistribution(OodReason),
}

impl OodArtifact {
    /// Applies scope, quality, shape, finite, and standardized-distance checks.
    pub fn evaluate(&self, evidence: &InferenceEvidence) -> Result<OodDecision, UncertaintyError> {
        self.validate()?;
        let contains = |values: &[String], value: &str| values.iter().any(|item| item == value);
        let scope_checks = [
            (
                !contains(&self.scope.hardware_profiles, &evidence.hardware_profile),
                OodReason::Hardware,
            ),
            (
                !contains(&self.scope.room_profiles, &evidence.room_profile),
                OodReason::Room,
            ),
            (
                !contains(&self.scope.tasks, &evidence.task),
                OodReason::Task,
            ),
            (
                !contains(&self.scope.postures, &evidence.posture),
                OodReason::Posture,
            ),
            (
                !contains(&self.scope.jurisdictions, &evidence.jurisdiction),
                OodReason::Jurisdiction,
            ),
        ];
        if let Some((_, reason)) = scope_checks.into_iter().find(|(failed, _)| *failed) {
            return Ok(OodDecision::OutOfDistribution(reason));
        }
        if evidence.features.len() != self.means.len() {
            return Ok(OodDecision::OutOfDistribution(OodReason::FeatureShape));
        }
        if !evidence.missingness.is_finite()
            || !(0.0..=self.maximum_missingness).contains(&evidence.missingness)
        {
            return Ok(OodDecision::OutOfDistribution(OodReason::Missingness));
        }
        if !evidence.interference.is_finite()
            || !(0.0..=self.maximum_interference).contains(&evidence.interference)
        {
            return Ok(OodDecision::OutOfDistribution(OodReason::Interference));
        }
        if evidence
            .features
            .iter()
            .zip(&self.means)
            .zip(&self.standard_deviations)
            .any(|((feature, mean), standard_deviation)| {
                !feature.is_finite()
                    || ((feature - mean) / standard_deviation).abs() > self.maximum_z_score
            })
        {
            return Ok(OodDecision::OutOfDistribution(
                OodReason::FeatureDistribution,
            ));
        }
        Ok(OodDecision::InDistribution)
    }

    fn validate(&self) -> Result<(), UncertaintyError> {
        let probability = |value: f64| value.is_finite() && (0.0..=1.0).contains(&value);
        let scope_valid = [
            &self.scope.hardware_profiles,
            &self.scope.room_profiles,
            &self.scope.tasks,
            &self.scope.postures,
            &self.scope.jurisdictions,
        ]
        .into_iter()
        .all(|values| !values.is_empty() && values.iter().all(|value| !value.trim().is_empty()));
        if !self.frozen
            || self.ood_id.trim().is_empty()
            || self.model_digest.trim().is_empty()
            || self.means.is_empty()
            || self.means.len() != self.standard_deviations.len()
            || self.means.iter().any(|value| !value.is_finite())
            || self
                .standard_deviations
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0)
            || !self.maximum_z_score.is_finite()
            || self.maximum_z_score <= 0.0
            || !probability(self.maximum_missingness)
            || !probability(self.maximum_interference)
            || !scope_valid
        {
            return Err(UncertaintyError::InvalidOodArtifact);
        }
        Ok(())
    }
}

/// Serving decision after calibration and OOD/scope checks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UncertaintyDecision {
    /// Calibrated probability passed the frozen threshold.
    Serve(CalibratedProbability),
    /// No production result may be emitted.
    Abstain(UncertaintyAbstention),
}

/// Stable uncertainty abstention reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UncertaintyAbstention {
    /// Calibration artifact is invalid or bound to another model.
    CalibrationInvalid,
    /// OOD artifact is invalid or bound to another model.
    OodArtifactInvalid,
    /// Evidence is OOD or out of scope.
    Ood(OodReason),
    /// Probability is below the frozen serving threshold.
    InsufficientConfidence,
}

/// Runs the complete uncertainty gate, including exact model binding.
pub fn evaluate_uncertainty(
    model_digest: &str,
    raw_score: f64,
    calibration: &CalibrationArtifact,
    ood: &OodArtifact,
    evidence: &InferenceEvidence,
) -> UncertaintyDecision {
    if calibration.model_digest != model_digest || calibration.validate().is_err() {
        return UncertaintyDecision::Abstain(UncertaintyAbstention::CalibrationInvalid);
    }
    if ood.model_digest != model_digest {
        return UncertaintyDecision::Abstain(UncertaintyAbstention::OodArtifactInvalid);
    }
    match ood.evaluate(evidence) {
        Ok(OodDecision::InDistribution) => {}
        Ok(OodDecision::OutOfDistribution(reason)) => {
            return UncertaintyDecision::Abstain(UncertaintyAbstention::Ood(reason));
        }
        Err(_) => {
            return UncertaintyDecision::Abstain(UncertaintyAbstention::OodArtifactInvalid);
        }
    }
    let Ok(probability) = calibration.calibrate(raw_score) else {
        return UncertaintyDecision::Abstain(UncertaintyAbstention::CalibrationInvalid);
    };
    if probability.get() < calibration.serving_threshold {
        UncertaintyDecision::Abstain(UncertaintyAbstention::InsufficientConfidence)
    } else {
        UncertaintyDecision::Serve(probability)
    }
}

/// One selective-risk operating point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectiveRiskPoint {
    /// Retained sample fraction.
    pub coverage: f64,
    /// Mean non-negative loss among retained samples.
    pub risk: f64,
    /// Minimum retained confidence.
    pub threshold: f64,
}

/// Evaluates risk versus coverage from calibrated confidence and finite loss.
pub fn selective_risk_coverage(
    confidence_and_loss: &[(CalibratedProbability, f64)],
) -> Result<Vec<SelectiveRiskPoint>, UncertaintyError> {
    if confidence_and_loss.is_empty()
        || confidence_and_loss
            .iter()
            .any(|(_, loss)| !loss.is_finite() || *loss < 0.0)
    {
        return Err(UncertaintyError::InvalidSelectiveRisk);
    }
    let mut ordered = confidence_and_loss.to_vec();
    ordered.sort_by(|left, right| {
        right
            .0
            .get()
            .partial_cmp(&left.0.get())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut loss_sum = 0.0_f64;
    let mut output = Vec::with_capacity(ordered.len());
    for (index, (confidence, loss)) in ordered.iter().enumerate() {
        loss_sum += loss;
        let risk = loss_sum / (index + 1) as f64;
        if !risk.is_finite() {
            return Err(UncertaintyError::InvalidSelectiveRisk);
        }
        output.push(SelectiveRiskPoint {
            coverage: (index + 1) as f64 / ordered.len() as f64,
            risk,
            threshold: confidence.get(),
        });
    }
    Ok(output)
}

/// Stable invalid-artifact/input failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UncertaintyError {
    /// Value is not a finite probability.
    InvalidProbability,
    /// Calibration is unfrozen, leaking, malformed, or non-monotonic.
    InvalidCalibration,
    /// OOD artifact or scope is malformed or unfrozen.
    InvalidOodArtifact,
    /// Selective-risk input is empty, non-finite, or negative.
    InvalidSelectiveRisk,
}

impl fmt::Display for UncertaintyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for UncertaintyError {}
