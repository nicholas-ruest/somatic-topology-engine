//! Versioned, layered runtime configuration with isolated secret material.

use std::{error::Error, fmt, fmt::Write as _};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The current configuration schema major version.
pub const CURRENT_SCHEMA_VERSION: u16 = 2;

const DEFAULT_QUEUE_CAPACITY: usize = 256;
const MAX_QUEUE_CAPACITY: usize = 4096;

/// Identifies a configuration layer and therefore its precedence and policy.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LayerSource {
    /// A verified deployment profile.
    SignedProfile,
    /// Local non-secret device configuration.
    DeviceFile,
    /// An explicit allow-listed environment layer.
    Environment,
    /// Narrow runtime command-line overrides.
    CommandLine,
}

/// Values that a configuration layer may provide.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PartialConfig {
    /// Human-readable local instance name.
    pub display_name: Option<String>,
    /// Capacity of bounded synthetic-pipeline channels.
    pub queue_capacity: Option<usize>,
    /// Whether capture may be requested (authorization still gates capture).
    pub capture_enabled: Option<bool>,
    /// Whether production assurance rules apply.
    pub production_mode: Option<bool>,
    /// Development-only operational relaxations.
    pub developer_relaxations: Option<bool>,
}

/// One explicit configuration layer. It never reads ambient process state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigLayer {
    source: LayerSource,
    values: PartialConfig,
}

impl ConfigLayer {
    /// Creates a layer from explicitly supplied values.
    #[must_use]
    pub const fn new(source: LayerSource, values: PartialConfig) -> Self {
        Self { source, values }
    }

    /// Parses a versioned JSON document, migrating supported older schemas.
    pub fn from_json(source: LayerSource, document: &str) -> Result<Self, ConfigError> {
        let raw: serde_json::Value = serde_json::from_str(document)
            .map_err(|error| ConfigError::MalformedDocument(error.to_string()))?;
        let version = raw
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            .ok_or(ConfigError::MissingSchemaVersion)?;
        let version = u16::try_from(version)
            .map_err(|_| ConfigError::MalformedDocument("schema version exceeds u16".into()))?;

        let values = match version {
            1 => migrate_v1(raw)?,
            CURRENT_SCHEMA_VERSION => {
                let mut object = raw.as_object().cloned().ok_or_else(|| {
                    ConfigError::MalformedDocument("configuration must be an object".into())
                })?;
                object.remove("schema_version");
                serde_json::from_value(serde_json::Value::Object(object))
                    .map_err(|error| ConfigError::MalformedDocument(error.to_string()))?
            }
            unsupported => return Err(ConfigError::UnsupportedSchemaVersion(unsupported)),
        };
        Ok(Self::new(source, values))
    }
}

fn migrate_v1(raw: serde_json::Value) -> Result<PartialConfig, ConfigError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VersionOne {
        schema_version: u16,
        name: Option<String>,
        channel_capacity: Option<usize>,
    }

    let old: VersionOne = serde_json::from_value(raw)
        .map_err(|error| ConfigError::MalformedDocument(error.to_string()))?;
    if old.schema_version != 1 {
        return Err(ConfigError::UnsupportedSchemaVersion(old.schema_version));
    }
    Ok(PartialConfig {
        display_name: old.name,
        queue_capacity: old.channel_capacity,
        ..PartialConfig::default()
    })
}

/// A signed, serialized [`PartialConfig`] deployment profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedProfile {
    /// Exact bytes covered by the signature.
    pub payload: Vec<u8>,
    /// Signature bytes.
    pub signature: Vec<u8>,
}

/// Port used to authenticate deployment profile bytes before parsing them.
pub trait ProfileSignatureVerifier {
    /// Returns true only when `signature` authenticates the exact `payload`.
    fn verify(&self, payload: &[u8], signature: &[u8]) -> bool;
}

/// Ed25519 implementation of deployment profile authentication.
#[derive(Clone, Debug)]
pub struct Ed25519ProfileVerifier {
    key: VerifyingKey,
}

impl Ed25519ProfileVerifier {
    /// Binds the verifier to a provisioned deployment-profile public key.
    #[must_use]
    pub const fn new(key: VerifyingKey) -> Self {
        Self { key }
    }
}

impl ProfileSignatureVerifier for Ed25519ProfileVerifier {
    fn verify(&self, payload: &[u8], signature: &[u8]) -> bool {
        let Ok(signature) = Signature::from_slice(signature) else {
            return false;
        };
        self.key.verify(payload, &signature).is_ok()
    }
}

/// Secret-provider configuration, excluded from serialization and redacted in diagnostics.
#[derive(Clone, Eq, PartialEq)]
pub struct SecretProviderConfig {
    /// Opaque lookup key for the selected secret provider.
    pub provider_key: String,
}

impl fmt::Debug for SecretProviderConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretProviderConfig(REDACTED)")
    }
}

/// Bounded pipeline settings.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PipelineConfig {
    /// Bounded channel capacity.
    pub queue_capacity: usize,
}

/// Fully resolved and validated runtime configuration.
#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct RuntimeConfig {
    /// Configuration schema major version.
    pub schema_version: u16,
    /// Local display name.
    pub display_name: String,
    /// Bounded pipeline settings.
    pub pipeline: PipelineConfig,
    /// Whether capture may be requested; authorization remains separately mandatory.
    pub capture_enabled: bool,
    /// Whether production assurance rules apply.
    pub production_mode: bool,
    /// Whether development-only operational relaxation is requested.
    pub developer_relaxations: bool,
    /// Isolated secret-provider state, omitted from serialization.
    #[serde(skip)]
    pub secrets: SecretProviderConfig,
}

impl fmt::Debug for RuntimeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeConfig")
            .field("schema_version", &self.schema_version)
            .field("display_name", &self.display_name)
            .field("pipeline", &self.pipeline)
            .field("capture_enabled", &self.capture_enabled)
            .field("production_mode", &self.production_mode)
            .field("developer_relaxations", &self.developer_relaxations)
            .field("secrets", &"REDACTED")
            .finish()
    }
}

impl fmt::Display for RuntimeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl RuntimeConfig {
    /// Computes the lowercase SHA-256 digest of canonical non-secret JSON.
    #[must_use]
    pub fn non_secret_digest(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("serializing a validated config cannot fail");
        let digest = Sha256::digest(bytes);
        let mut encoded = String::with_capacity(digest.len() * 2);
        for byte in digest {
            write!(encoded, "{byte:02x}").expect("writing to a String cannot fail");
        }
        encoded
    }
}

/// Builder that resolves explicitly supplied layers and validates the result.
#[derive(Clone, Debug)]
pub struct ConfigurationLoader {
    layers: Vec<ConfigLayer>,
    secret_provider_key: String,
}

impl Default for ConfigurationLoader {
    fn default() -> Self {
        Self {
            layers: Vec::new(),
            secret_provider_key: "local-development".into(),
        }
    }
}

impl ConfigurationLoader {
    /// Adds an explicit layer. Resolution is by source precedence, independent of call order.
    #[must_use]
    pub fn with_layer(mut self, layer: ConfigLayer) -> Self {
        self.layers.push(layer);
        self
    }

    /// Verifies and adds a signed profile. Invalid signatures and payloads fail closed.
    pub fn with_signed_profile(
        mut self,
        profile: SignedProfile,
        verifier: &dyn ProfileSignatureVerifier,
    ) -> Result<Self, ConfigError> {
        if !verifier.verify(&profile.payload, &profile.signature) {
            return Err(ConfigError::InvalidProfileSignature);
        }
        let values = serde_json::from_slice(&profile.payload)
            .map_err(|error| ConfigError::MalformedDocument(error.to_string()))?;
        self.layers
            .push(ConfigLayer::new(LayerSource::SignedProfile, values));
        Ok(self)
    }

    /// Selects a secret-provider lookup key without placing it in non-secret layers.
    #[must_use]
    pub fn with_secret_provider_key(mut self, key: impl Into<String>) -> Self {
        self.secret_provider_key = key.into();
        self
    }

    /// Resolves all layers and validates the effective configuration.
    pub fn load(mut self) -> Result<RuntimeConfig, ConfigError> {
        self.layers.sort_by_key(|layer| layer.source);
        let mut config = safe_defaults(self.secret_provider_key);
        for layer in self.layers {
            reject_forbidden_overrides(&layer)?;
            apply(&mut config, layer.values);
        }
        validate(&config)?;
        Ok(config)
    }
}

fn safe_defaults(secret_provider_key: String) -> RuntimeConfig {
    RuntimeConfig {
        schema_version: CURRENT_SCHEMA_VERSION,
        display_name: "somatic-topology-engine".into(),
        pipeline: PipelineConfig {
            queue_capacity: DEFAULT_QUEUE_CAPACITY,
        },
        capture_enabled: false,
        production_mode: false,
        developer_relaxations: false,
        secrets: SecretProviderConfig {
            provider_key: secret_provider_key,
        },
    }
}

fn reject_forbidden_overrides(layer: &ConfigLayer) -> Result<(), ConfigError> {
    if layer.source == LayerSource::Environment {
        for (present, field) in [
            (layer.values.capture_enabled.is_some(), "capture_enabled"),
            (layer.values.production_mode.is_some(), "production_mode"),
            (
                layer.values.developer_relaxations.is_some(),
                "developer_relaxations",
            ),
        ] {
            if present {
                return Err(ConfigError::ForbiddenOverride {
                    source: layer.source,
                    field,
                });
            }
        }
    }
    Ok(())
}

fn apply(config: &mut RuntimeConfig, values: PartialConfig) {
    if let Some(value) = values.display_name {
        config.display_name = value;
    }
    if let Some(value) = values.queue_capacity {
        config.pipeline.queue_capacity = value;
    }
    if let Some(value) = values.capture_enabled {
        config.capture_enabled = value;
    }
    if let Some(value) = values.production_mode {
        config.production_mode = value;
    }
    if let Some(value) = values.developer_relaxations {
        config.developer_relaxations = value;
    }
}

fn validate(config: &RuntimeConfig) -> Result<(), ConfigError> {
    if config.display_name.trim().is_empty() {
        return Err(ConfigError::Validation("display_name must not be blank"));
    }
    if !(1..=MAX_QUEUE_CAPACITY).contains(&config.pipeline.queue_capacity) {
        return Err(ConfigError::Validation(
            "queue_capacity must be between 1 and 4096",
        ));
    }
    if config.production_mode && config.developer_relaxations {
        return Err(ConfigError::Validation(
            "production_mode forbids developer_relaxations",
        ));
    }
    if config.secrets.provider_key.trim().is_empty() {
        return Err(ConfigError::Validation(
            "a secret provider key must be configured",
        ));
    }
    Ok(())
}

/// Configuration resolution or validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// A layer attempted to alter a restricted setting.
    ForbiddenOverride {
        /// Offending layer.
        source: LayerSource,
        /// Stable field name.
        field: &'static str,
    },
    /// A signed profile could not be authenticated.
    InvalidProfileSignature,
    /// A versioned document omitted its version.
    MissingSchemaVersion,
    /// A document uses an unsupported schema major version.
    UnsupportedSchemaVersion(u16),
    /// A document could not be decoded without ambiguity.
    MalformedDocument(String),
    /// Effective values violate an invariant.
    Validation(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForbiddenOverride { source, field } => {
                write!(formatter, "{source:?} cannot override {field}")
            }
            Self::InvalidProfileSignature => formatter.write_str("invalid profile signature"),
            Self::MissingSchemaVersion => formatter.write_str("missing schema_version"),
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "unsupported schema version {version}")
            }
            Self::MalformedDocument(reason) => write!(formatter, "malformed document: {reason}"),
            Self::Validation(reason) => write!(formatter, "invalid configuration: {reason}"),
        }
    }
}

impl Error for ConfigError {}
