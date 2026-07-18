//! Operational latent-state claims and fail-closed projection policy.

use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Compatibility marker for bounded-context tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainBoundary;

fn required(value: impl Into<String>, label: &'static str) -> Result<String, DomainError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 256 {
        Err(DomainError::InvalidValue(label))
    } else {
        Ok(value)
    }
}

/// Supported operational constructs; unsupported affect/decision labels are absent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationalConstruct {
    /// Deterministic no-claim path used for integration validation.
    Baseline,
    /// Workload defined only for one exact task and definition version.
    TaskSpecificWorkload {
        /// Exact task identifier.
        task: String,
        /// Non-zero operational-definition version.
        version: u32,
    },
}

/// Versioned construct definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstructDefinition {
    construct: OperationalConstruct,
}
impl ConstructDefinition {
    /// Creates the no-claim baseline.
    #[must_use]
    pub const fn baseline() -> Self {
        Self {
            construct: OperationalConstruct::Baseline,
        }
    }
    /// Creates an exact task-specific workload construct.
    pub fn task_workload(task: impl Into<String>, version: u32) -> Result<Self, DomainError> {
        if version == 0 {
            return Err(DomainError::InvalidValue("construct version"));
        }
        Ok(Self {
            construct: OperationalConstruct::TaskSpecificWorkload {
                task: required(task, "task")?,
                version,
            },
        })
    }
    /// Stable reference used by the external validation registry.
    #[must_use]
    pub fn reference(&self) -> String {
        match &self.construct {
            OperationalConstruct::Baseline => "baseline".into(),
            OperationalConstruct::TaskSpecificWorkload { task, version } => {
                format!("task-workload:{task}:v{version}")
            }
        }
    }
    /// Returns whether this is the deterministic no-claim construct.
    #[must_use]
    pub const fn is_baseline(&self) -> bool {
        matches!(self.construct, OperationalConstruct::Baseline)
    }
}

/// Product claim level; medical-grade and universal claims are unrepresentable.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ClaimLevel {
    /// Experiment-only output excluded from production projection.
    Experimental,
    /// Promoted, task-specific, non-medical claim.
    ValidatedNonMedical,
}

/// Finite calibrated probability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalibratedProbability(f64);
impl CalibratedProbability {
    /// Creates a probability in `[0,1]`.
    pub fn new(value: f64) -> Result<Self, DomainError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidProbability)
        }
    }
    /// Returns the calibrated value for internal policy calculations.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// Exact model, calibration, and profile lineage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    /// Active model version.
    pub model_version: String,
    /// Frozen calibration identifier.
    pub calibration_id: String,
    /// Participant/profile version.
    pub profile_version: String,
}
impl Provenance {
    /// Creates complete provenance.
    pub fn new(
        model: impl Into<String>,
        calibration: impl Into<String>,
        profile: impl Into<String>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            model_version: required(model, "model version")?,
            calibration_id: required(calibration, "calibration identifier")?,
            profile_version: required(profile, "profile version")?,
        })
    }
}

/// Evidence references remain distinct from the resulting state assessment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceBundle {
    /// Source observation identity.
    pub observation_id: String,
    /// Valid physiology assessment identity.
    pub physiology_id: String,
    /// Evidence duration, independent of display cadence.
    pub horizon_ms: u64,
    /// Full inference lineage.
    pub provenance: Provenance,
}
impl EvidenceBundle {
    /// Creates a complete evidence bundle.
    pub fn new(
        observation: impl Into<String>,
        physiology: impl Into<String>,
        horizon_ms: u64,
        provenance: Provenance,
    ) -> Result<Self, DomainError> {
        if horizon_ms == 0 {
            return Err(DomainError::InvalidValue("evidence horizon"));
        }
        Ok(Self {
            observation_id: required(observation, "observation identifier")?,
            physiology_id: required(physiology, "physiology identifier")?,
            horizon_ms,
            provenance,
        })
    }
}

/// Exact participant, environment, task, posture, and hardware scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelScope {
    /// Participant/profile subject.
    pub participant: String,
    /// Qualified room.
    pub room: String,
    /// Exact operational task.
    pub task: String,
    /// Validated posture.
    pub posture: String,
    /// Hardware profile.
    pub hardware: String,
}
impl ModelScope {
    /// Creates complete model scope.
    pub fn new(
        participant: impl Into<String>,
        room: impl Into<String>,
        task: impl Into<String>,
        posture: impl Into<String>,
        hardware: impl Into<String>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            participant: required(participant, "participant")?,
            room: required(room, "room")?,
            task: required(task, "task")?,
            posture: required(posture, "posture")?,
            hardware: required(hardware, "hardware")?,
        })
    }
}

/// Complete non-configurable evidence for every serving gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GateEvidence {
    /// Experiment Validation promoted this exact construct.
    pub construct_promoted: bool,
    /// Physiology evidence is valid and non-abstained.
    pub physiology_valid: bool,
    /// Participant profile is valid.
    pub profile_valid: bool,
    /// Exact model is verified and active.
    pub model_active: bool,
    /// Calibration artifact is valid.
    pub calibration_valid: bool,
    /// Signed capability policy authorized serving.
    pub policy_authorized: bool,
    /// Runtime evidence matches model scope.
    pub in_scope: bool,
    /// Supported operating envelope holds.
    pub inside_envelope: bool,
}
impl GateEvidence {
    /// Creates the fully passing test/reference value.
    #[must_use]
    pub const fn all_passed() -> Self {
        Self {
            construct_promoted: true,
            physiology_valid: true,
            profile_valid: true,
            model_active: true,
            calibration_valid: true,
            policy_authorized: true,
            in_scope: true,
            inside_envelope: true,
        }
    }
}

/// Complete assessment command.
#[derive(Clone, Debug, PartialEq)]
pub struct AssessLatentState {
    assessment_id: String,
    construct: ConstructDefinition,
    evidence: EvidenceBundle,
    scope: ModelScope,
    probability: CalibratedProbability,
    gates: GateEvidence,
}
impl AssessLatentState {
    /// Creates a command after checking task definition/scope consistency.
    pub fn new(
        id: impl Into<String>,
        construct: ConstructDefinition,
        evidence: EvidenceBundle,
        scope: ModelScope,
        probability: CalibratedProbability,
        gates: GateEvidence,
    ) -> Result<Self, DomainError> {
        if let OperationalConstruct::TaskSpecificWorkload { task, .. } = &construct.construct {
            if task != &scope.task {
                return Err(DomainError::ScopeMismatch);
            }
        }
        Ok(Self {
            assessment_id: required(id, "assessment identifier")?,
            construct,
            evidence,
            scope,
            probability,
            gates,
        })
    }
}

/// Fail-closed assessment reasons.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum AbstentionReason {
    /// Construct lacks promotion.
    ConstructNotPromoted,
    /// Physiology invalid or abstained.
    InvalidPhysiology,
    /// Profile unavailable or invalid.
    InvalidProfile,
    /// Verified active model unavailable.
    ModelUnavailable,
    /// Calibration invalid.
    CalibrationInvalid,
    /// Signed feature policy disabled output.
    PolicyDisabled,
    /// Runtime evidence outside model scope.
    OutOfScope,
    /// Supported operating envelope violated.
    OutsideOperatingEnvelope,
    /// Baseline intentionally makes no claim.
    BaselineNoClaim,
}

/// Immutable state assessment; raw model outputs never implement projection directly.
#[derive(Clone, Debug, PartialEq)]
pub struct StateAssessment {
    id: String,
    construct: OperationalConstruct,
    evidence: EvidenceBundle,
    scope: ModelScope,
    probability: CalibratedProbability,
    claim_level: ClaimLevel,
}
impl StateAssessment {
    /// Applies every gate in deterministic fail-closed order.
    #[must_use]
    pub fn assess(command: AssessLatentState) -> AssessmentOutcome {
        let reason = if matches!(command.construct.construct, OperationalConstruct::Baseline) {
            Some(AbstentionReason::BaselineNoClaim)
        } else if !command.gates.construct_promoted {
            Some(AbstentionReason::ConstructNotPromoted)
        } else if !command.gates.physiology_valid {
            Some(AbstentionReason::InvalidPhysiology)
        } else if !command.gates.profile_valid {
            Some(AbstentionReason::InvalidProfile)
        } else if !command.gates.model_active {
            Some(AbstentionReason::ModelUnavailable)
        } else if !command.gates.calibration_valid {
            Some(AbstentionReason::CalibrationInvalid)
        } else if !command.gates.policy_authorized {
            Some(AbstentionReason::PolicyDisabled)
        } else if !command.gates.in_scope {
            Some(AbstentionReason::OutOfScope)
        } else if !command.gates.inside_envelope {
            Some(AbstentionReason::OutsideOperatingEnvelope)
        } else {
            None
        };
        if let Some(reason) = reason {
            return AssessmentOutcome::Abstained {
                assessment_id: command.assessment_id,
                reason,
            };
        }
        AssessmentOutcome::Estimated(Self {
            id: command.assessment_id,
            construct: command.construct.construct,
            evidence: command.evidence,
            scope: command.scope,
            probability: command.probability,
            claim_level: ClaimLevel::ValidatedNonMedical,
        })
    }
    /// Stable assessment identity.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Approved claim level.
    #[must_use]
    pub const fn claim_level(&self) -> ClaimLevel {
        self.claim_level
    }
    /// Returns immutable source evidence and lineage.
    #[must_use]
    pub const fn evidence(&self) -> &EvidenceBundle {
        &self.evidence
    }
    /// Returns the exact model operating scope.
    #[must_use]
    pub const fn scope(&self) -> &ModelScope {
        &self.scope
    }
    /// Returns the calibrated internal probability; projections remain banded.
    #[must_use]
    pub const fn probability(&self) -> CalibratedProbability {
        self.probability
    }
}

/// Assessment outcome persisted and emitted as a domain event.
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum AssessmentOutcome {
    /// Approved task-specific estimate.
    Estimated(StateAssessment),
    /// Typed no-claim outcome.
    Abstained {
        /// Stable assessment identity.
        assessment_id: String,
        /// Fail-closed reason.
        reason: AbstentionReason,
    },
}

/// Approved coarse display vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkloadBand {
    /// Lower calibrated task-specific band.
    Lower,
    /// Middle calibrated task-specific band.
    Moderate,
    /// Elevated calibrated task-specific band.
    Elevated,
}

/// Versioned user-facing boundary containing no raw score or unsupported label.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DisplayProjectionV1 {
    /// Schema major.
    pub schema_major: u16,
    /// Source assessment identity.
    pub assessment_id: String,
    /// Explicit task-specific workload band.
    pub workload: WorkloadBand,
    /// Non-medical claim statement.
    pub claim: String,
}
impl TryFrom<&StateAssessment> for DisplayProjectionV1 {
    type Error = ProjectionError;
    fn try_from(value: &StateAssessment) -> Result<Self, Self::Error> {
        if value.claim_level != ClaimLevel::ValidatedNonMedical
            || !matches!(
                value.construct,
                OperationalConstruct::TaskSpecificWorkload { .. }
            )
        {
            return Err(ProjectionError::NotProjectable);
        }
        let workload = if value.probability.get() < 0.34 {
            WorkloadBand::Lower
        } else if value.probability.get() < 0.67 {
            WorkloadBand::Moderate
        } else {
            WorkloadBand::Elevated
        };
        Ok(Self {
            schema_major: 1,
            assessment_id: value.id.clone(),
            workload,
            claim: "task-specific non-medical workload estimate".into(),
        })
    }
}

/// State-inference events.
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StateInferenceEvent {
    /// Assessment produced approved evidence.
    Estimated(StateAssessment),
    /// Assessment abstained.
    Abstained {
        /// Assessment identifier.
        assessment_id: String,
        /// Abstention reason.
        reason: AbstentionReason,
    },
    /// Construct was disabled by immutable decision.
    ConstructDisabled {
        /// Construct definition reference.
        construct_reference: String,
    },
}

/// Domain construction failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    /// Invalid bounded value.
    InvalidValue(&'static str),
    /// Invalid probability.
    InvalidProbability,
    /// Construct task differs from model scope.
    ScopeMismatch,
}
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "state inference invariant failed: {self:?}")
    }
}
impl Error for DomainError {}

/// Projection failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionError {
    /// Assessment is not a promoted production claim.
    NotProjectable,
}
