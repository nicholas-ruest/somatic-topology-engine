//! Signed, content-addressed model package verification.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

fn required(value: impl Into<String>, label: &'static str) -> Result<String, PackageError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 512 {
        return Err(PackageError::InvalidMetadata(label));
    }
    Ok(value)
}

/// Immutable metadata required to reproduce and govern an edge model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ModelMetadata {
    /// Unique immutable model version.
    pub model_id: String,
    /// Rust engine/model format identifier.
    pub format: String,
    /// Ordered feature schema digest.
    pub feature_schema: [u8; 32],
    /// Deterministic preprocessing digest.
    pub preprocessing: [u8; 32],
    /// Calibration artifact digest.
    pub calibration: [u8; 32],
    /// Versioned operating scope.
    pub operating_scope: String,
    /// Training/evaluation lineage reference.
    pub lineage: String,
    /// Resource benchmark/profile reference.
    pub resource_profile: String,
    /// Commercially reviewed model/data license.
    pub license: String,
    /// Required STE software compatibility identifier.
    pub software_compatibility: String,
    /// Required hardware compatibility identifier.
    pub hardware_compatibility: String,
    /// Model card digest.
    pub model_card: [u8; 32],
}

impl ModelMetadata {
    /// Builds complete package metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model_id: impl Into<String>,
        format: impl Into<String>,
        feature_schema: [u8; 32],
        preprocessing: [u8; 32],
        calibration: [u8; 32],
        operating_scope: impl Into<String>,
        lineage: impl Into<String>,
        resource_profile: impl Into<String>,
        license: impl Into<String>,
        software: impl Into<String>,
        hardware: impl Into<String>,
        model_card: [u8; 32],
    ) -> Result<Self, PackageError> {
        Ok(Self {
            model_id: required(model_id, "model identifier")?,
            format: required(format, "model format")?,
            feature_schema,
            preprocessing,
            calibration,
            operating_scope: required(operating_scope, "operating scope")?,
            lineage: required(lineage, "lineage")?,
            resource_profile: required(resource_profile, "resource profile")?,
            license: required(license, "license")?,
            software_compatibility: required(software, "software compatibility")?,
            hardware_compatibility: required(hardware, "hardware compatibility")?,
            model_card,
        })
    }
}

#[derive(Serialize)]
struct SignedPayload<'a> {
    metadata: &'a ModelMetadata,
    weights_digest: [u8; 32],
}

/// Signed model package containing immutable weights and governance metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ModelPackage {
    metadata: ModelMetadata,
    weights: Vec<u8>,
    weights_digest: [u8; 32],
    signature: Option<Vec<u8>>,
}

impl ModelPackage {
    /// Creates an unsigned package for an offline signing ceremony.
    pub fn unsigned(metadata: ModelMetadata, weights: Vec<u8>) -> Result<Self, PackageError> {
        if weights.is_empty() {
            return Err(PackageError::EmptyWeights);
        }
        let weights_digest = Sha256::digest(&weights).into();
        Ok(Self {
            metadata,
            weights,
            weights_digest,
            signature: None,
        })
    }
    fn payload(&self) -> Result<Vec<u8>, PackageError> {
        serde_json::to_vec(&SignedPayload {
            metadata: &self.metadata,
            weights_digest: self.weights_digest,
        })
        .map_err(|_| PackageError::Serialization)
    }
    /// Signs the package payload and returns a sealed package.
    pub fn sign(mut self, key: &SigningKey) -> Result<Self, PackageError> {
        self.signature = Some(key.sign(&self.payload()?).to_bytes().to_vec());
        Ok(self)
    }
    /// Returns immutable metadata.
    #[must_use]
    pub const fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }
    /// Returns model weights only after callers have verified the package.
    #[must_use]
    pub fn weights(&self) -> &[u8] {
        &self.weights
    }
    /// Recomputes content integrity and verifies the package signature.
    pub fn verify(
        &self,
        key: &VerifyingKey,
        compatibility: &Compatibility,
    ) -> Result<VerifiedPackage, PackageError> {
        let signature = self.signature.as_ref().ok_or(PackageError::Unsigned)?;
        let signature =
            Signature::from_slice(signature).map_err(|_| PackageError::InvalidSignature)?;
        let actual: [u8; 32] = Sha256::digest(&self.weights).into();
        if actual != self.weights_digest {
            return Err(PackageError::CorruptWeights);
        }
        key.verify(&self.payload()?, &signature)
            .map_err(|_| PackageError::InvalidSignature)?;
        if self.metadata.software_compatibility != compatibility.software
            || self.metadata.hardware_compatibility != compatibility.hardware
            || self.metadata.feature_schema != compatibility.feature_schema
        {
            return Err(PackageError::Incompatible);
        }
        Ok(VerifiedPackage(self.clone()))
    }
}

/// Local runtime compatibility matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Compatibility {
    software: String,
    hardware: String,
    feature_schema: [u8; 32],
}
impl Compatibility {
    /// Creates the exact supported runtime tuple.
    pub fn new(
        software: impl Into<String>,
        hardware: impl Into<String>,
        feature_schema: [u8; 32],
    ) -> Result<Self, PackageError> {
        Ok(Self {
            software: required(software, "software compatibility")?,
            hardware: required(hardware, "hardware compatibility")?,
            feature_schema,
        })
    }
}

/// Package proof obtainable only after integrity, signature, and compatibility checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPackage(ModelPackage);
impl VerifiedPackage {
    /// Returns immutable verified package content.
    #[must_use]
    pub const fn package(&self) -> &ModelPackage {
        &self.0
    }
    /// Consumes the proof into the verified package.
    #[must_use]
    pub fn into_package(self) -> ModelPackage {
        self.0
    }
}

/// Stable package rejection reasons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackageError {
    /// Required metadata was invalid.
    InvalidMetadata(&'static str),
    /// No weights were included.
    EmptyWeights,
    /// Package lacks a signature.
    Unsigned,
    /// Signature encoding or verification failed.
    InvalidSignature,
    /// Weights differ from their signed digest.
    CorruptWeights,
    /// Runtime compatibility did not match exactly.
    Incompatible,
    /// Canonical payload serialization failed.
    Serialization,
}
impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "model package rejected: {self:?}")
    }
}
impl Error for PackageError {}
