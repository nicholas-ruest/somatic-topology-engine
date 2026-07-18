//! Signal-only observation aggregate and immutable evidence artifacts.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, error::Error, fmt, fmt::Write as _};

/// Auditable list of public domain vocabulary used by semantic boundary tests.
pub const PUBLIC_OBSERVATION_TYPES: &[&str] = &[
    "ObservationWindow",
    "FrameEvidence",
    "MotionEnergy",
    "PresenceScore",
    "PeriodicityCandidate",
    "BaselineDrift",
    "QualityAssessment",
    "FeatureEvidenceArtifact",
];

macro_rules! text_value {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
        pub struct $name(String);
        impl $name {
            /// Creates a non-empty version/identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, ObservationError> {
                let value = value.into();
                if value.trim().is_empty() {
                    Err(ObservationError::InvalidValue)
                } else {
                    Ok(Self(value))
                }
            }
            /// Returns the exact value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}
text_value!(ObservationWindowId, "Opaque observation window identifier.");
text_value!(AlgorithmVersion, "Feature algorithm version.");
text_value!(DspVersion, "DSP graph version.");

/// Ordered non-empty event-time window.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WindowBounds {
    /// Inclusive start.
    pub start_ns: u64,
    /// Exclusive end.
    pub end_ns: u64,
}
impl WindowBounds {
    /// Validates ordered bounds.
    pub fn new(start_ns: u64, end_ns: u64) -> Result<Self, ObservationError> {
        if start_ns >= end_ns {
            Err(ObservationError::InvalidBounds)
        } else {
            Ok(Self { start_ns, end_ns })
        }
    }
}

/// Named immutable closure/quality policy.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowPolicy {
    name: String,
    min_frames: usize,
    max_frames: usize,
    max_missing_ratio: f64,
    max_interference_ratio: f64,
}
impl WindowPolicy {
    /// Validates bounded counts and finite ratios.
    pub fn new(
        name: impl Into<String>,
        min_frames: usize,
        max_frames: usize,
        max_missing_ratio: f64,
        max_interference_ratio: f64,
    ) -> Result<Self, ObservationError> {
        let name = name.into();
        if name.trim().is_empty()
            || min_frames == 0
            || min_frames > max_frames
            || max_frames > 1_000_000
            || !valid_ratio(max_missing_ratio)
            || !valid_ratio(max_interference_ratio)
        {
            Err(ObservationError::InvalidPolicy)
        } else {
            Ok(Self {
                name,
                min_frames,
                max_frames,
                max_missing_ratio,
                max_interference_ratio,
            })
        }
    }
}
fn valid_ratio(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

macro_rules! nonnegative {
    ($name:ident,$unit:literal,$doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq, Serialize)]
        pub struct $name(f64);
        impl $name {
            /// Creates a finite non-negative signal value.
            pub fn new(value: f64) -> Result<Self, ObservationError> {
                if value.is_finite() && value >= 0.0 {
                    Ok(Self(value))
                } else {
                    Err(ObservationError::InvalidValue)
                }
            }
            /// Returns the numeric value.
            #[must_use]
            pub const fn get(self) -> f64 {
                self.0
            }
            /// Returns the declared unit.
            #[must_use]
            pub const fn unit(self) -> &'static str {
                $unit
            }
        }
    };
}
nonnegative!(
    MotionEnergy,
    "normalized_energy",
    "Normalized signal motion energy."
);
nonnegative!(
    BaselineDrift,
    "normalized_drift",
    "Normalized baseline drift magnitude."
);

/// Finite presence score in `[0,1]` without identity semantics.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PresenceScore(f64);
impl PresenceScore {
    /// Creates a bounded score.
    pub fn new(value: f64) -> Result<Self, ObservationError> {
        if valid_ratio(value) {
            Ok(Self(value))
        } else {
            Err(ObservationError::InvalidValue)
        }
    }
    /// Returns the score.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// Signal periodicity candidate with strength; it carries no source interpretation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PeriodicityCandidate {
    frequency_hz: f64,
    strength: f64,
}
impl PeriodicityCandidate {
    /// Creates a finite positive frequency and bounded strength.
    pub fn new(frequency_hz: f64, strength: f64) -> Result<Self, ObservationError> {
        if frequency_hz.is_finite() && frequency_hz > 0.0 && valid_ratio(strength) {
            Ok(Self {
                frequency_hz,
                strength,
            })
        } else {
            Err(ObservationError::InvalidValue)
        }
    }
    /// Frequency unit.
    #[must_use]
    pub const fn frequency_unit(self) -> &'static str {
        "hertz"
    }
}

/// Reference and signal-only features derived from one accepted frame.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameEvidence {
    /// Immutable acquisition reference.
    pub source_ref: String,
    /// Monotonic event time.
    pub event_time_ns: u64,
    /// Motion energy.
    pub motion_energy: MotionEnergy,
    /// Presence score.
    pub presence_score: PresenceScore,
    /// Optional periodicity.
    pub periodicity: Option<PeriodicityCandidate>,
    /// Baseline drift.
    pub baseline_drift: BaselineDrift,
    /// Missing sequence positions before this frame.
    pub missing_before: u64,
    /// Explicit interference flag.
    pub interference: bool,
}

/// Quality/abstention cause.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AbstentionReason {
    /// Interference detected.
    Interference,
    /// Missingness exceeds policy.
    Missingness,
    /// Too few frames.
    InsufficientEvidence,
    /// Saturated source.
    Saturation,
    /// Motion contamination.
    MotionContamination,
}
/// Quality disposition; contaminated is monotonic.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum QualityDisposition {
    /// Meets declared policy.
    Clean,
    /// Usable with explicit limitation.
    Degraded,
    /// Must not become a clean downstream estimate.
    Contaminated,
}
/// Explicit quality result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct QualityAssessment {
    /// Disposition.
    pub disposition: QualityDisposition,
    /// Complete reasons.
    pub reasons: BTreeSet<AbstentionReason>,
}
/// Dataset partition role preserved to prevent leakage.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PartitionRole {
    /// Development/training exploration.
    Development,
    /// Validation selection.
    Validation,
    /// Held-out test.
    Test,
    /// Production observation.
    Production,
}

/// Immutable content-addressed feature/evidence artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureEvidenceArtifact {
    digest: String,
    bounds: WindowBounds,
    policy: String,
    algorithm_version: AlgorithmVersion,
    dsp_version: DspVersion,
    calibration_version: String,
    source_refs: Vec<String>,
    motion_energy: MotionEnergy,
    presence_score: PresenceScore,
    periodicity: Vec<PeriodicityCandidate>,
    baseline_drift: BaselineDrift,
    quality: QualityAssessment,
    partition_role: PartitionRole,
}
impl FeatureEvidenceArtifact {
    /// Content digest.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
    /// Source evidence references.
    #[must_use]
    pub fn source_refs(&self) -> &[String] {
        &self.source_refs
    }
    /// Aggregate motion energy.
    #[must_use]
    pub const fn motion_energy(&self) -> MotionEnergy {
        self.motion_energy
    }
    /// Periodicity candidates.
    #[must_use]
    pub fn periodicity(&self) -> &[PeriodicityCandidate] {
        &self.periodicity
    }
    /// Calibration version.
    #[must_use]
    pub fn calibration_version(&self) -> &str {
        &self.calibration_version
    }
    /// Quality result.
    #[must_use]
    pub const fn quality(&self) -> &QualityAssessment {
        &self.quality
    }
}

/// Observation aggregate.
#[derive(Clone, Debug, PartialEq)]
pub struct ObservationWindow {
    id: ObservationWindowId,
    bounds: WindowBounds,
    policy: WindowPolicy,
    algorithm_version: AlgorithmVersion,
    dsp_version: DspVersion,
    calibration_version: String,
    frames: Vec<FrameEvidence>,
    reasons: BTreeSet<AbstentionReason>,
}
impl ObservationWindow {
    /// Opens a window with complete version provenance.
    pub fn open(
        id: ObservationWindowId,
        bounds: WindowBounds,
        policy: WindowPolicy,
        algorithm_version: AlgorithmVersion,
        dsp_version: DspVersion,
        calibration_version: String,
    ) -> Self {
        Self {
            id,
            bounds,
            policy,
            algorithm_version,
            dsp_version,
            calibration_version,
            frames: Vec::new(),
            reasons: BTreeSet::new(),
        }
    }
    /// Appends ordered frame evidence within bounds.
    pub fn append(&mut self, frame: FrameEvidence) -> Result<(), ObservationError> {
        if frame.source_ref.trim().is_empty()
            || frame.event_time_ns < self.bounds.start_ns
            || frame.event_time_ns >= self.bounds.end_ns
            || self
                .frames
                .last()
                .is_some_and(|last| frame.event_time_ns < last.event_time_ns)
            || self.frames.len() >= self.policy.max_frames
        {
            return Err(ObservationError::InvalidFrame);
        }
        if frame.interference {
            self.reasons.insert(AbstentionReason::Interference);
        }
        self.frames.push(frame);
        Ok(())
    }
    /// Records a signal anomaly without discarding evidence.
    pub fn record_anomaly(&mut self, reason: AbstentionReason) -> Result<(), ObservationError> {
        self.reasons.insert(reason);
        Ok(())
    }
    /// Closes and content-addresses the immutable artifact.
    pub fn close(
        mut self,
        partition_role: PartitionRole,
    ) -> Result<FeatureEvidenceArtifact, ObservationError> {
        if self.frames.len() < self.policy.min_frames {
            self.reasons.insert(AbstentionReason::InsufficientEvidence);
        }
        let total_missing: u64 = self.frames.iter().map(|f| f.missing_before).sum();
        let denominator = total_missing.saturating_add(self.frames.len() as u64);
        let missing_ratio = if denominator == 0 {
            1.0
        } else {
            total_missing as f64 / denominator as f64
        };
        if missing_ratio > self.policy.max_missing_ratio {
            self.reasons.insert(AbstentionReason::Missingness);
        }
        let interference_ratio = self.frames.iter().filter(|f| f.interference).count() as f64
            / self.frames.len().max(1) as f64;
        if interference_ratio > self.policy.max_interference_ratio
            || self.frames.iter().any(|f| f.interference)
        {
            self.reasons.insert(AbstentionReason::Interference);
        }
        let quality = QualityAssessment {
            disposition: if self.reasons.is_empty() {
                QualityDisposition::Clean
            } else {
                QualityDisposition::Contaminated
            },
            reasons: self.reasons,
        };
        let count = self.frames.len().max(1) as f64;
        let motion = MotionEnergy::new(
            self.frames
                .iter()
                .map(|f| f.motion_energy.get())
                .sum::<f64>()
                / count,
        )?;
        let presence = PresenceScore::new(
            self.frames
                .iter()
                .map(|f| f.presence_score.get())
                .sum::<f64>()
                / count,
        )?;
        let drift = BaselineDrift::new(
            self.frames
                .iter()
                .map(|f| f.baseline_drift.get())
                .sum::<f64>()
                / count,
        )?;
        let source_refs = self
            .frames
            .iter()
            .map(|f| f.source_ref.clone())
            .collect::<Vec<_>>();
        let periodicity = self
            .frames
            .iter()
            .filter_map(|f| f.periodicity)
            .collect::<Vec<_>>();
        let canonical = serde_json::to_vec(&(
            &self.id,
            &self.bounds,
            &self.policy.name,
            &self.algorithm_version,
            &self.dsp_version,
            &self.calibration_version,
            &source_refs,
            motion,
            presence,
            &periodicity,
            drift,
            &quality,
            partition_role,
        ))
        .map_err(|_| ObservationError::ArtifactFailure)?;
        let digest = hex(&Sha256::digest(canonical));
        Ok(FeatureEvidenceArtifact {
            digest,
            bounds: self.bounds,
            policy: self.policy.name,
            algorithm_version: self.algorithm_version,
            dsp_version: self.dsp_version,
            calibration_version: self.calibration_version,
            source_refs,
            motion_energy: motion,
            presence_score: presence,
            periodicity,
            baseline_drift: drift,
            quality,
            partition_role,
        })
    }
}
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(out, "{byte:02x}").expect("String write cannot fail");
    }
    out
}
/// Observation commands.
pub enum ObservationCommand {
    /// Append evidence.
    Append(FrameEvidence),
    /// Record anomaly.
    RecordAnomaly(AbstentionReason),
    /// Close window.
    Close(PartitionRole),
}
/// Observation events.
pub enum ObservationEvent {
    /// Window opened.
    Opened,
    /// Anomaly observed.
    AnomalyObserved(AbstentionReason),
    /// Window contaminated.
    Contaminated,
    /// Window closed with artifact digest.
    Closed(String),
}
/// Domain invariant error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationError {
    /// Invalid value.
    InvalidValue,
    /// Invalid bounds.
    InvalidBounds,
    /// Invalid policy.
    InvalidPolicy,
    /// Frame order/bounds/reference invalid.
    InvalidFrame,
    /// Artifact serialization failed.
    ArtifactFailure,
}
impl fmt::Display for ObservationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for ObservationError {}
/// Marker for architecture tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainBoundary;
