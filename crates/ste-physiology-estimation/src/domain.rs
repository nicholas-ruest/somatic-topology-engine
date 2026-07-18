//! Respiration-only physiology assessment domain.

use crate::application::{PhysiologyModel, ValidationRegistry};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Compatibility marker for bounded-context tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainBoundary;

fn required(value: impl Into<String>, label: &'static str) -> Result<String, DomainError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 256 {
        return Err(DomainError::InvalidValue(label));
    }
    Ok(value)
}

/// The sole physiology modality enabled in this phase.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PhysiologyModality {
    /// Respiratory rate inferred from a validated evidence horizon.
    RespirationRate,
}

/// Modality-specific stillness policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum StillnessRequirement {
    /// Respiration is emitted only for still observation windows.
    Required,
}

/// Calibrated probability constrained to `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalibratedConfidence(f64);
impl CalibratedConfidence {
    /// Creates finite calibrated confidence.
    pub fn new(value: f64) -> Result<Self, DomainError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidProbability)
        }
    }
    /// Returns the probability.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// Symmetric non-negative error bound in breaths per minute.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ErrorBounds(f64);
impl ErrorBounds {
    /// Creates a finite non-negative bound.
    pub fn new(value: f64) -> Result<Self, DomainError> {
        if value.is_finite() && value >= 0.0 {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidEstimate)
        }
    }
    /// Returns the bound.
    #[must_use]
    pub const fn breaths_per_minute(self) -> f64 {
        self.0
    }
}

/// Exact evidence duration; presentation cadence never changes it.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceHorizon(u64);
impl EvidenceHorizon {
    /// Creates a positive duration in milliseconds.
    pub fn new(milliseconds: u64) -> Result<Self, DomainError> {
        if milliseconds == 0 {
            Err(DomainError::InvalidValue("evidence horizon"))
        } else {
            Ok(Self(milliseconds))
        }
    }
    /// Returns the duration.
    #[must_use]
    pub const fn milliseconds(self) -> u64 {
        self.0
    }
}

/// Observation evidence plus explicit policy signals.
#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceWindow {
    features: Vec<f64>,
    horizon: EvidenceHorizon,
    quality: f64,
    still: bool,
    ood: bool,
    calibration_valid: bool,
    inside_envelope: bool,
}
impl EvidenceWindow {
    /// Creates finite evidence and its fail-closed policy metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        features: Vec<f64>,
        horizon: EvidenceHorizon,
        quality: f64,
        still: bool,
        ood: bool,
        calibration_valid: bool,
        inside_envelope: bool,
    ) -> Result<Self, DomainError> {
        if features.is_empty() || features.iter().any(|v| !v.is_finite()) {
            return Err(DomainError::InvalidFeatures);
        }
        if !quality.is_finite() || !(0.0..=1.0).contains(&quality) {
            return Err(DomainError::InvalidProbability);
        }
        Ok(Self {
            features,
            horizon,
            quality,
            still,
            ood,
            calibration_valid,
            inside_envelope,
        })
    }
}

/// Reasons an assessment emits no physiological estimate.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AbstentionReason {
    /// Validation registry has not promoted the model.
    NotPromoted,
    /// Required stillness was violated.
    Motion,
    /// Input quality was below policy.
    InsufficientQuality,
    /// Evidence was out of distribution.
    OutOfDistribution,
    /// Calibration artifact was invalid.
    CalibrationInvalid,
    /// Evidence horizon was too short.
    InsufficientEvidence,
    /// Operating scope was violated.
    OutsideOperatingEnvelope,
}

/// Explicit non-medical validation state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ValidationStatus {
    /// Promoted for a non-medical product claim only.
    PromotedNonMedical,
}

/// Finite model output before domain policy labels it.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelEstimate {
    breaths_per_minute: f64,
    confidence: f64,
    error_bound: f64,
    model_version: String,
}
impl ModelEstimate {
    /// Creates a plausible, calibrated respiratory-rate estimate.
    pub fn new(
        rate: f64,
        confidence: f64,
        error_bound: f64,
        model: impl Into<String>,
    ) -> Result<Self, DomainError> {
        if !rate.is_finite()
            || !(2.0..=80.0).contains(&rate)
            || !error_bound.is_finite()
            || error_bound < 0.0
        {
            return Err(DomainError::InvalidEstimate);
        }
        if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
            return Err(DomainError::InvalidProbability);
        }
        Ok(Self {
            breaths_per_minute: rate,
            confidence,
            error_bound,
            model_version: required(model, "model version")?,
        })
    }
}

/// Immutable promoted respiration estimate.
#[derive(Clone, Debug, PartialEq)]
pub struct RespirationEstimate {
    assessment_id: String,
    observation_id: String,
    breaths_per_minute: f64,
    confidence: f64,
    error_bound: f64,
    model_version: String,
    horizon: EvidenceHorizon,
    validation: ValidationStatus,
}
impl RespirationEstimate {
    /// Estimated breaths per minute.
    #[must_use]
    pub const fn breaths_per_minute(&self) -> f64 {
        self.breaths_per_minute
    }
    /// Validation label that cannot express medical grade.
    #[must_use]
    pub const fn validation_status(&self) -> ValidationStatus {
        self.validation
    }
}

/// Assessment output is either evidence or a typed abstention.
#[derive(Clone, Debug, PartialEq)]
pub enum AssessmentOutcome {
    /// Valid promoted evidence.
    Estimated(RespirationEstimate),
    /// No estimate due to a policy gate.
    Abstained(AbstentionReason),
}

/// Domain events emitted by respiration assessment policy.
#[derive(Clone, Debug, PartialEq)]
pub enum PhysiologyEvent {
    /// A promoted non-medical estimate was emitted.
    Estimated(RespirationEstimate),
    /// Policy refused to emit a physiology estimate.
    Abstained {
        /// Stable assessment identity.
        assessment_id: String,
        /// Fail-closed reason.
        reason: AbstentionReason,
    },
    /// Experiment Validation withdrew the only enabled modality.
    RespirationWithdrawn {
        /// Immutable validation decision reference.
        decision_reference: String,
    },
}

/// Complete respiration assessment command.
#[derive(Clone, Debug, PartialEq)]
pub struct AssessPhysiology {
    assessment_id: String,
    observation_id: String,
    model_id: String,
    evidence: EvidenceWindow,
}
impl AssessPhysiology {
    /// Creates a respiration-only command.
    pub fn new(
        assessment: impl Into<String>,
        observation: impl Into<String>,
        model: impl Into<String>,
        evidence: EvidenceWindow,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            assessment_id: required(assessment, "assessment identifier")?,
            observation_id: required(observation, "observation identifier")?,
            model_id: required(model, "model identifier")?,
            evidence,
        })
    }
}

/// Stateless aggregate policy service producing immutable outcomes.
pub struct PhysiologyAssessment;
impl PhysiologyAssessment {
    /// Applies validation and operating gates before invoking the model.
    pub fn assess<R: ValidationRegistry, M: PhysiologyModel>(
        command: AssessPhysiology,
        registry: &R,
        model: &M,
    ) -> Result<AssessmentOutcome, AssessmentError<R::Error, M::Error>> {
        if !registry
            .respiration_is_promoted(&command.model_id)
            .map_err(AssessmentError::Registry)?
        {
            return Ok(AssessmentOutcome::Abstained(AbstentionReason::NotPromoted));
        }
        let e = &command.evidence;
        let reason = if !e.still {
            Some(AbstentionReason::Motion)
        } else if e.quality < 0.8 {
            Some(AbstentionReason::InsufficientQuality)
        } else if e.ood {
            Some(AbstentionReason::OutOfDistribution)
        } else if !e.calibration_valid {
            Some(AbstentionReason::CalibrationInvalid)
        } else if e.horizon.milliseconds() < 20_000 {
            Some(AbstentionReason::InsufficientEvidence)
        } else if !e.inside_envelope {
            Some(AbstentionReason::OutsideOperatingEnvelope)
        } else {
            None
        };
        if let Some(reason) = reason {
            return Ok(AssessmentOutcome::Abstained(reason));
        }
        let output = model
            .estimate_respiration(&e.features)
            .map_err(AssessmentError::Model)?;
        Ok(AssessmentOutcome::Estimated(RespirationEstimate {
            assessment_id: command.assessment_id,
            observation_id: command.observation_id,
            breaths_per_minute: output.breaths_per_minute,
            confidence: output.confidence,
            error_bound: output.error_bound,
            model_version: output.model_version,
            horizon: e.horizon,
            validation: ValidationStatus::PromotedNonMedical,
        }))
    }
}

/// Versioned boundary event for downstream evidence consumers.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PhysiologyEvidenceUpdatedV1 {
    /// Schema major.
    pub schema_major: u16,
    /// Assessment identifier.
    pub assessment_id: String,
    /// Observation provenance identifier.
    pub observation_id: String,
    /// Respiratory rate.
    pub breaths_per_minute: f64,
    /// Calibrated confidence.
    pub confidence: f64,
    /// Symmetric error bound.
    pub error_bound: f64,
    /// Evidence horizon in milliseconds.
    pub evidence_horizon_ms: u64,
    /// Explicit non-medical validation label.
    pub validation_status: ValidationStatus,
}
impl From<RespirationEstimate> for PhysiologyEvidenceUpdatedV1 {
    fn from(value: RespirationEstimate) -> Self {
        Self {
            schema_major: 1,
            assessment_id: value.assessment_id,
            observation_id: value.observation_id,
            breaths_per_minute: value.breaths_per_minute,
            confidence: value.confidence,
            error_bound: value.error_bound,
            evidence_horizon_ms: value.horizon.milliseconds(),
            validation_status: value.validation,
        }
    }
}

/// Domain construction errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    /// A required string or horizon was invalid.
    InvalidValue(&'static str),
    /// Feature vector was empty or non-finite.
    InvalidFeatures,
    /// Confidence or quality was invalid.
    InvalidProbability,
    /// Model output was invalid.
    InvalidEstimate,
}
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "physiology invariant failed: {self:?}")
    }
}
impl Error for DomainError {}

/// External-boundary failure; policy abstentions remain successful outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssessmentError<R, M> {
    /// Validation registry unavailable.
    Registry(R),
    /// Model execution failed.
    Model(M),
}
