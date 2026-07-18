//! Deterministic, non-ML respiration baseline over promoted observation evidence.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;

/// Immutable algorithm package metadata used for every estimate.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RespirationModelPackage {
    /// Stable package identifier.
    pub package_id: String,
    /// Semantic package version.
    pub package_version: String,
    /// Fixed algorithm identifier; only the deterministic baseline is accepted.
    pub algorithm: String,
    /// Required upstream DSP graph version.
    pub dsp_graph_version: u16,
    /// Confidence calibration identifier.
    pub calibration_id: String,
    /// Supported minimum breaths per minute.
    pub minimum_bpm: f64,
    /// Supported maximum breaths per minute.
    pub maximum_bpm: f64,
    /// Required periodicity score.
    pub minimum_periodicity: f64,
    /// Maximum accepted missingness.
    pub maximum_missingness: f64,
    /// Maximum accepted interference.
    pub maximum_interference: f64,
    /// Minimum calibrated confidence required to emit an estimate.
    pub minimum_confidence: f64,
    /// Piecewise-linear calibration knots `(raw, calibrated)`.
    pub confidence_calibration: Vec<(f64, f64)>,
}

impl RespirationModelPackage {
    /// Validates metadata, operating envelope, and monotonic calibration.
    pub fn validate(&self) -> Result<(), EstimatorError> {
        let bounded = |value: f64| value.is_finite() && (0.0..=1.0).contains(&value);
        if self.package_id.trim().is_empty()
            || self.package_version.trim().is_empty()
            || self.algorithm != "deterministic-periodicity-v1"
            || self.dsp_graph_version == 0
            || self.calibration_id.trim().is_empty()
            || !self.minimum_bpm.is_finite()
            || !self.maximum_bpm.is_finite()
            || self.minimum_bpm <= 0.0
            || self.maximum_bpm <= self.minimum_bpm
            || !bounded(self.minimum_periodicity)
            || !bounded(self.maximum_missingness)
            || !bounded(self.maximum_interference)
            || !bounded(self.minimum_confidence)
            || self.confidence_calibration.len() < 2
            || self
                .confidence_calibration
                .iter()
                .any(|(raw, calibrated)| !bounded(*raw) || !bounded(*calibrated))
            || self
                .confidence_calibration
                .windows(2)
                .any(|pair| pair[1].0 <= pair[0].0 || pair[1].1 < pair[0].1)
            || self.confidence_calibration.first().map(|knot| knot.0) != Some(0.0)
            || self.confidence_calibration.last().map(|knot| knot.0) != Some(1.0)
        {
            return Err(EstimatorError::InvalidPackage);
        }
        Ok(())
    }

    /// Content digest for parity evidence and activation records.
    pub fn digest(&self) -> Result<String, EstimatorError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| EstimatorError::InvalidPackage)?;
        let mut digest = String::with_capacity(64);
        for byte in Sha256::digest(bytes) {
            write!(digest, "{byte:02x}").expect("String write cannot fail");
        }
        Ok(digest)
    }
}

/// Immutable label-free observation evidence required by the baseline.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RespirationObservationEvidence {
    /// Upstream immutable artifact reference.
    pub artifact_ref: String,
    /// DSP graph version that produced the evidence.
    pub dsp_graph_version: u16,
    /// Window-level normalized periodicity.
    pub periodicity: f64,
    /// Dominant periodic frequency, absent when upstream cannot support it.
    pub dominant_frequency_hz: Option<f64>,
    /// Window missingness ratio.
    pub missingness: f64,
    /// Window interference ratio.
    pub interference: f64,
    /// Whether upstream marked the artifact contaminated.
    pub contaminated: bool,
}

/// Stable reason no estimate was emitted.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RespirationAbstentionReason {
    /// Required observation evidence is absent or invalid.
    MissingEvidence,
    /// Evidence uses a different DSP graph.
    DspVersionMismatch,
    /// Observation was contaminated.
    Contaminated,
    /// Too many expected samples are absent.
    ExcessiveMissingness,
    /// Interference exceeds the package envelope.
    ExcessiveInterference,
    /// Periodicity is below the package threshold.
    WeakPeriodicity,
    /// Dominant frequency is outside the package envelope.
    OutsideOperatingEnvelope,
    /// Calibrated confidence is below the emission threshold.
    LowConfidence,
}

/// Successful deterministic respiration estimate.
#[derive(Clone, Debug, PartialEq)]
pub struct RespirationEstimate {
    /// Breaths per minute.
    pub breaths_per_minute: f64,
    /// Calibrated bounded confidence.
    pub calibrated_confidence: f64,
    /// Exact model-package digest.
    pub package_digest: String,
    /// Upstream evidence reference.
    pub source_artifact_ref: String,
}

/// Fail-closed estimator result.
#[derive(Clone, Debug, PartialEq)]
pub enum RespirationOutcome {
    /// Estimate passed every gate.
    Estimated(RespirationEstimate),
    /// No value is emitted.
    Abstained(RespirationAbstentionReason),
}

/// Executes the deterministic periodicity baseline.
pub fn estimate_respiration(
    package: &RespirationModelPackage,
    evidence: &RespirationObservationEvidence,
) -> Result<RespirationOutcome, EstimatorError> {
    package.validate()?;
    if evidence.artifact_ref.trim().is_empty()
        || !evidence.periodicity.is_finite()
        || !(0.0..=1.0).contains(&evidence.periodicity)
        || !evidence.missingness.is_finite()
        || !(0.0..=1.0).contains(&evidence.missingness)
        || !evidence.interference.is_finite()
        || !(0.0..=1.0).contains(&evidence.interference)
        || evidence
            .dominant_frequency_hz
            .is_some_and(|frequency| !frequency.is_finite() || frequency <= 0.0)
    {
        return Ok(RespirationOutcome::Abstained(
            RespirationAbstentionReason::MissingEvidence,
        ));
    }
    let reason = if evidence.dsp_graph_version != package.dsp_graph_version {
        Some(RespirationAbstentionReason::DspVersionMismatch)
    } else if evidence.contaminated {
        Some(RespirationAbstentionReason::Contaminated)
    } else if evidence.missingness > package.maximum_missingness {
        Some(RespirationAbstentionReason::ExcessiveMissingness)
    } else if evidence.interference > package.maximum_interference {
        Some(RespirationAbstentionReason::ExcessiveInterference)
    } else if evidence.periodicity < package.minimum_periodicity {
        Some(RespirationAbstentionReason::WeakPeriodicity)
    } else {
        None
    };
    if let Some(reason) = reason {
        return Ok(RespirationOutcome::Abstained(reason));
    }
    let Some(frequency) = evidence.dominant_frequency_hz else {
        return Ok(RespirationOutcome::Abstained(
            RespirationAbstentionReason::MissingEvidence,
        ));
    };
    let breaths_per_minute = frequency * 60.0;
    if !breaths_per_minute.is_finite()
        || !(package.minimum_bpm..=package.maximum_bpm).contains(&breaths_per_minute)
    {
        return Ok(RespirationOutcome::Abstained(
            RespirationAbstentionReason::OutsideOperatingEnvelope,
        ));
    }
    let raw_confidence =
        (evidence.periodicity * (1.0 - evidence.missingness) * (1.0 - evidence.interference))
            .clamp(0.0, 1.0);
    let calibrated_confidence =
        interpolate_calibration(raw_confidence, &package.confidence_calibration);
    if calibrated_confidence < package.minimum_confidence {
        return Ok(RespirationOutcome::Abstained(
            RespirationAbstentionReason::LowConfidence,
        ));
    }
    Ok(RespirationOutcome::Estimated(RespirationEstimate {
        breaths_per_minute,
        calibrated_confidence,
        package_digest: package.digest()?,
        source_artifact_ref: evidence.artifact_ref.clone(),
    }))
}

fn interpolate_calibration(value: f64, knots: &[(f64, f64)]) -> f64 {
    for pair in knots.windows(2) {
        if value <= pair[1].0 {
            let fraction = (value - pair[0].0) / (pair[1].0 - pair[0].0);
            return pair[0].1 + fraction * (pair[1].1 - pair[0].1);
        }
    }
    knots.last().map_or(0.0, |knot| knot.1)
}

/// Package or checked-computation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EstimatorError {
    /// Package metadata or calibration is invalid.
    InvalidPackage,
}

impl fmt::Display for EstimatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for EstimatorError {}
