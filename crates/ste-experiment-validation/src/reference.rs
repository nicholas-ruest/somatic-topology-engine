//! Reference-sensor ports and explicit time-alignment artifacts.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Supported validation reference source.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReferenceKind {
    /// Calibrated respiratory belt.
    RespiratoryBelt,
    /// Electrocardiogram reference.
    Ecg,
    /// PPG device explicitly validated for the protocol.
    ValidatedPpg,
    /// Preregistered task marker.
    TaskTimestamp,
    /// Participant self-report marker.
    SelfReportTimestamp,
}

/// One immutable reference observation or marker.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReferenceSample {
    /// Reference-clock timestamp.
    pub timestamp_ns: u64,
    /// Sensor value or preregistered marker code.
    pub value: f64,
    /// Bounded quality score from the reference adapter/protocol.
    pub quality: f64,
}

impl ReferenceSample {
    /// Validates finite value, time, and quality in `[0,1]`.
    pub fn new(timestamp_ns: u64, value: f64, quality: f64) -> Result<Self, SyncError> {
        if timestamp_ns == 0
            || !value.is_finite()
            || !quality.is_finite()
            || !(0.0..=1.0).contains(&quality)
        {
            return Err(SyncError::InvalidReference);
        }
        Ok(Self {
            timestamp_ns,
            value,
            quality,
        })
    }
}

/// Reference stream port; adapters return complete immutable samples.
pub trait ReferenceSensor: Send + Sync {
    /// Reference modality.
    fn kind(&self) -> ReferenceKind;
    /// Stable device/protocol identifier.
    fn source_id(&self) -> &str;
    /// Complete ordered sample set.
    fn samples(&self) -> &[ReferenceSample];
}

macro_rules! reference_adapter {
    ($name:ident, $kind:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $name {
            source_id: String,
            samples: Vec<ReferenceSample>,
        }

        impl $name {
            /// Creates an adapter after validating identity and strict timestamp order.
            pub fn new(
                source_id: impl Into<String>,
                samples: Vec<ReferenceSample>,
            ) -> Result<Self, SyncError> {
                let source_id = source_id.into();
                validate_stream(&source_id, &samples)?;
                Ok(Self { source_id, samples })
            }
        }

        impl ReferenceSensor for $name {
            fn kind(&self) -> ReferenceKind {
                $kind
            }
            fn source_id(&self) -> &str {
                &self.source_id
            }
            fn samples(&self) -> &[ReferenceSample] {
                &self.samples
            }
        }
    };
}

reference_adapter!(
    RespiratoryBeltAdapter,
    ReferenceKind::RespiratoryBelt,
    "Calibrated respiratory-belt adapter."
);
reference_adapter!(EcgAdapter, ReferenceKind::Ecg, "ECG reference adapter.");
reference_adapter!(
    ValidatedPpgAdapter,
    ReferenceKind::ValidatedPpg,
    "Protocol-validated PPG adapter."
);
reference_adapter!(
    TaskTimestampAdapter,
    ReferenceKind::TaskTimestamp,
    "Preregistered task-timestamp adapter."
);
reference_adapter!(
    SelfReportTimestampAdapter,
    ReferenceKind::SelfReportTimestamp,
    "Participant self-report timestamp adapter."
);

/// Explicit alignment method. There is no implicit default interpolation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AlignmentMethod {
    /// Only an exact timestamp may align.
    ExactOnly,
    /// Nearest sample may align within the declared uncertainty bound.
    Nearest,
    /// Linear interpolation may use two bracketing samples within the bound.
    Linear,
}

/// Result for one target timestamp.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AlignedReference {
    /// Target evidence timestamp.
    pub target_timestamp_ns: u64,
    /// Value when alignment succeeded.
    pub value: Option<f64>,
    /// Reference timestamps actually used; empty means missing.
    pub source_timestamps_ns: Vec<u64>,
    /// Maximum absolute clock/sample distance used.
    pub alignment_uncertainty_ns: Option<u64>,
    /// Conservative reference quality (minimum for interpolation).
    pub reference_quality: Option<f64>,
    /// Explicit method used; `None` means no alignment.
    pub method: Option<AlignmentMethod>,
}

/// Immutable synchronization evidence with no hidden interpolation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SynchronizationArtifact {
    /// Reference modality.
    pub reference_kind: ReferenceKind,
    /// Reference source/protocol identifier.
    pub source_id: String,
    /// Requested maximum time uncertainty.
    pub maximum_uncertainty_ns: u64,
    /// Requested alignment method.
    pub requested_method: AlignmentMethod,
    /// One result per target timestamp, including missing alignments.
    pub alignments: Vec<AlignedReference>,
}

/// Aligns target timestamps using only the explicitly selected method.
pub fn synchronize(
    sensor: &dyn ReferenceSensor,
    targets_ns: &[u64],
    method: AlignmentMethod,
    maximum_uncertainty_ns: u64,
    minimum_reference_quality: f64,
) -> Result<SynchronizationArtifact, SyncError> {
    validate_stream(sensor.source_id(), sensor.samples())?;
    if targets_ns.is_empty()
        || targets_ns.windows(2).any(|pair| pair[1] <= pair[0])
        || !minimum_reference_quality.is_finite()
        || !(0.0..=1.0).contains(&minimum_reference_quality)
    {
        return Err(SyncError::InvalidTarget);
    }
    let alignments = targets_ns
        .iter()
        .map(|target| {
            align_one(
                sensor.samples(),
                *target,
                method,
                maximum_uncertainty_ns,
                minimum_reference_quality,
            )
        })
        .collect();
    Ok(SynchronizationArtifact {
        reference_kind: sensor.kind(),
        source_id: sensor.source_id().to_owned(),
        maximum_uncertainty_ns,
        requested_method: method,
        alignments,
    })
}

fn align_one(
    samples: &[ReferenceSample],
    target: u64,
    method: AlignmentMethod,
    maximum: u64,
    minimum_quality: f64,
) -> AlignedReference {
    if let Some(sample) = samples
        .iter()
        .find(|sample| sample.timestamp_ns == target && sample.quality >= minimum_quality)
    {
        return aligned(
            target,
            sample.value,
            vec![target],
            0,
            sample.quality,
            AlignmentMethod::ExactOnly,
        );
    }
    match method {
        AlignmentMethod::ExactOnly => missing(target),
        AlignmentMethod::Nearest => samples
            .iter()
            .filter(|sample| sample.quality >= minimum_quality)
            .min_by_key(|sample| (sample.timestamp_ns.abs_diff(target), sample.timestamp_ns))
            .filter(|sample| sample.timestamp_ns.abs_diff(target) <= maximum)
            .map_or_else(
                || missing(target),
                |sample| {
                    aligned(
                        target,
                        sample.value,
                        vec![sample.timestamp_ns],
                        sample.timestamp_ns.abs_diff(target),
                        sample.quality,
                        AlignmentMethod::Nearest,
                    )
                },
            ),
        AlignmentMethod::Linear => {
            let lower = samples
                .iter()
                .rev()
                .find(|sample| sample.timestamp_ns < target && sample.quality >= minimum_quality);
            let upper = samples
                .iter()
                .find(|sample| sample.timestamp_ns > target && sample.quality >= minimum_quality);
            match (lower, upper) {
                (Some(lower), Some(upper))
                    if lower.timestamp_ns.abs_diff(target) <= maximum
                        && upper.timestamp_ns.abs_diff(target) <= maximum =>
                {
                    let fraction = (target - lower.timestamp_ns) as f64
                        / (upper.timestamp_ns - lower.timestamp_ns) as f64;
                    aligned(
                        target,
                        lower.value + fraction * (upper.value - lower.value),
                        vec![lower.timestamp_ns, upper.timestamp_ns],
                        lower
                            .timestamp_ns
                            .abs_diff(target)
                            .max(upper.timestamp_ns.abs_diff(target)),
                        lower.quality.min(upper.quality),
                        AlignmentMethod::Linear,
                    )
                }
                _ => missing(target),
            }
        }
    }
}

fn aligned(
    target: u64,
    value: f64,
    sources: Vec<u64>,
    uncertainty: u64,
    quality: f64,
    method: AlignmentMethod,
) -> AlignedReference {
    AlignedReference {
        target_timestamp_ns: target,
        value: Some(value),
        source_timestamps_ns: sources,
        alignment_uncertainty_ns: Some(uncertainty),
        reference_quality: Some(quality),
        method: Some(method),
    }
}

fn missing(target: u64) -> AlignedReference {
    AlignedReference {
        target_timestamp_ns: target,
        value: None,
        source_timestamps_ns: Vec::new(),
        alignment_uncertainty_ns: None,
        reference_quality: None,
        method: None,
    }
}

fn validate_stream(source_id: &str, samples: &[ReferenceSample]) -> Result<(), SyncError> {
    if source_id.trim().is_empty()
        || samples.is_empty()
        || samples
            .windows(2)
            .any(|pair| pair[1].timestamp_ns <= pair[0].timestamp_ns)
    {
        Err(SyncError::InvalidReference)
    } else {
        Ok(())
    }
}

/// Reference stream or synchronization input error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyncError {
    /// Invalid sensor identity, sample, or ordering.
    InvalidReference,
    /// Empty, unordered, or invalid target/alignment configuration.
    InvalidTarget,
}

impl fmt::Display for SyncError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for SyncError {}
