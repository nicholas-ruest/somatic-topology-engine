#![forbid(unsafe_code)]
//! Rust-owned wire contracts shared by STE bounded contexts and adapters.
//!
//! This crate deliberately contains transport shapes only. Domain invariants and
//! infrastructure mappings belong to their owning bounded contexts.

use std::fmt;

use schemars::JsonSchema;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use uuid::Uuid;

fn deserialize_non_empty<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = String::deserialize(deserializer)?;
    if value.trim().is_empty() {
        Err(de::Error::custom("required string must not be empty"))
    } else {
        Ok(value)
    }
}

fn deserialize_non_zero_u16<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u16, D::Error> {
    let value = u16::deserialize(deserializer)?;
    if value == 0 {
        Err(de::Error::custom("count must be greater than zero"))
    } else {
        Ok(value)
    }
}

/// A semantic version attached to every public contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ContractVersion {
    pub major: u16,
    pub minor: u16,
}

impl ContractVersion {
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Deserialize)]
struct RawContractVersion {
    major: u16,
    minor: u16,
}

impl TryFrom<RawContractVersion> for ContractVersion {
    type Error = ContractError;

    fn try_from(raw: RawContractVersion) -> Result<Self, Self::Error> {
        if raw.major == 0 {
            return Err(ContractError::InvalidSchemaVersion);
        }
        Ok(Self::new(raw.major, raw.minor))
    }
}

impl<'de> Deserialize<'de> for ContractVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        RawContractVersion::deserialize(deserializer)?
            .try_into()
            .map_err(de::Error::custom)
    }
}

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("contract schema major version must be greater than zero")]
    InvalidSchemaVersion,
    #[error("contract number must be finite")]
    NonFiniteNumber,
    #[error("required contract field `{0}` must not be empty")]
    EmptyRequiredField(&'static str),
}

/// A JSON-safe floating-point number. NaN and infinities cannot enter a DTO.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, JsonSchema)]
#[schemars(transparent)]
pub struct FiniteF64(f64);

impl FiniteF64 {
    pub fn new(value: f64) -> Result<Self, ContractError> {
        value
            .is_finite()
            .then_some(Self(value))
            .ok_or(ContractError::NonFiniteNumber)
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl Serialize for FiniteF64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !self.0.is_finite() {
            return Err(serde::ser::Error::custom(ContractError::NonFiniteNumber));
        }
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for FiniteF64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// Compatibility is intentionally asymmetric: a consumer accepts its own major
/// and a producer minor no newer than the consumer's supported minor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaSupport {
    major: u16,
    max_minor: u16,
}

impl SchemaSupport {
    #[must_use]
    pub const fn new(major: u16, max_minor: u16) -> Self {
        Self { major, max_minor }
    }

    #[must_use]
    pub const fn compatibility(self, version: ContractVersion) -> Compatibility {
        if version.major != self.major {
            Compatibility::UnsupportedMajor
        } else if version.minor > self.max_minor {
            Compatibility::UnsupportedMinor
        } else {
            Compatibility::Compatible
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    Compatible,
    UnsupportedMinor,
    UnsupportedMajor,
}

/// Metadata required for every context-crossing integration event (ADR-012).
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct ContractEnvelopeV1<P> {
    pub schema_version: ContractVersion,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub source_time_ns: u64,
    pub emitted_at_unix_ns: u64,
    pub producer_version: String,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub provenance_ref: String,
    pub idempotency_key: String,
    pub payload: P,
}

#[derive(Deserialize)]
struct RawEnvelope<P> {
    schema_version: ContractVersion,
    event_id: Uuid,
    aggregate_id: Uuid,
    source_time_ns: u64,
    emitted_at_unix_ns: u64,
    producer_version: String,
    correlation_id: Option<Uuid>,
    causation_id: Option<Uuid>,
    provenance_ref: String,
    idempotency_key: String,
    payload: P,
}

impl<'de, P> Deserialize<'de> for ContractEnvelopeV1<P>
where
    P: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawEnvelope::<P>::deserialize(deserializer)?;
        for (name, value) in [
            ("producer_version", raw.producer_version.as_str()),
            ("provenance_ref", raw.provenance_ref.as_str()),
            ("idempotency_key", raw.idempotency_key.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(de::Error::custom(ContractError::EmptyRequiredField(name)));
            }
        }
        Ok(Self {
            schema_version: raw.schema_version,
            event_id: raw.event_id,
            aggregate_id: raw.aggregate_id,
            source_time_ns: raw.source_time_ns,
            emitted_at_unix_ns: raw.emitted_at_unix_ns,
            producer_version: raw.producer_version,
            correlation_id: raw.correlation_id,
            causation_id: raw.causation_id,
            provenance_ref: raw.provenance_ref,
            idempotency_key: raw.idempotency_key,
            payload: raw.payload,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QualityDisposition {
    Usable,
    Degraded,
    Abstained,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidatedCsiFrameV1 {
    pub capture_session_id: Uuid,
    pub sequence: u64,
    pub monotonic_time_ns: u64,
    pub center_frequency_hz: u64,
    pub bandwidth_hz: u64,
    #[serde(deserialize_with = "deserialize_non_zero_u16")]
    pub antenna_count: u16,
    #[serde(deserialize_with = "deserialize_non_zero_u16")]
    pub subcarrier_count: u16,
    #[serde(deserialize_with = "deserialize_non_empty")]
    pub payload_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureHealthV1 {
    #[serde(deserialize_with = "deserialize_non_empty")]
    pub source_id: String,
    pub frames_received: u64,
    pub frames_rejected: u64,
    pub dropped_frames: u64,
    pub disposition: QualityDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservationWindowClosedV1 {
    pub window_id: Uuid,
    pub started_at_ns: u64,
    pub ended_at_ns: u64,
    pub accepted_frames: u64,
    pub rejected_frames: u64,
    #[serde(deserialize_with = "deserialize_non_empty")]
    pub feature_artifact_ref: String,
    pub disposition: QualityDisposition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysiologyEvidenceUpdatedV1 {
    pub assessment_id: Uuid,
    pub window_id: Uuid,
    pub respiration_hz: Option<FiniteF64>,
    pub confidence: FiniteF64,
    pub disposition: QualityDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    Presence,
    Motion,
    Respiration,
    InsufficientEvidence,
    Unauthorized,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DisplayProjectionV1 {
    pub projection_id: Uuid,
    pub revision: u64,
    pub mode: DisplayMode,
    #[serde(deserialize_with = "deserialize_non_empty")]
    pub headline: String,
    pub confidence: Option<FiniteF64>,
    pub stale: bool,
}

impl fmt::Display for ContractVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}
