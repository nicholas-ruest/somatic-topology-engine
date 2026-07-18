//! Canonical signed release evidence and identical channel promotion.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Required release artifact category; every category appears exactly once.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseArtifactKind {
    /// Exact source-tree digest.
    SourceTree,
    /// Locked dependency graph.
    DependencyLock,
    /// Hermetic compiler/toolchain image.
    Toolchain,
    /// Built software payload.
    Software,
    /// Patched firmware payload.
    Firmware,
    /// Software bill of materials.
    SoftwareSbom,
    /// Firmware bill of materials.
    FirmwareSbom,
    /// Model bill of materials.
    ModelSbom,
    /// Dataset bill of materials.
    DatasetSbom,
    /// Model cards.
    ModelCards,
    /// Dataset cards.
    DatasetCards,
    /// Hardware/software/model compatibility matrix.
    CompatibilityMatrix,
    /// Ordered storage/config migrations.
    Migrations,
    /// Update and operator documentation bundle.
    UpdateDocumentation,
    /// Tested rollback procedure.
    RollbackPlan,
    /// Versioned support/update compatibility matrix.
    SupportMatrix,
}
impl ReleaseArtifactKind {
    const ALL: [Self; 16] = [
        Self::SourceTree,
        Self::DependencyLock,
        Self::Toolchain,
        Self::Software,
        Self::Firmware,
        Self::SoftwareSbom,
        Self::FirmwareSbom,
        Self::ModelSbom,
        Self::DatasetSbom,
        Self::ModelCards,
        Self::DatasetCards,
        Self::CompatibilityMatrix,
        Self::Migrations,
        Self::UpdateDocumentation,
        Self::RollbackPlan,
        Self::SupportMatrix,
    ];
}

/// Content-addressed release artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseArtifact {
    /// Required artifact category.
    pub kind: ReleaseArtifactKind,
    /// Stable artifact filename/identity.
    pub name: String,
    /// SHA-256 digest of exact bytes.
    pub sha256: [u8; 32],
    /// Exact byte length.
    pub bytes: u64,
}

/// Canonical immutable release manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    /// Stable schema.
    pub schema: String,
    /// Release identity.
    pub release_id: String,
    /// Target triple.
    pub target: String,
    /// Exact verified hardware profile.
    pub hardware_profile: String,
    /// Sorted complete artifacts.
    pub artifacts: Vec<ReleaseArtifact>,
}

/// Signed manifest and digest used unchanged across channels.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedReleaseManifest {
    /// Canonical manifest.
    pub manifest: ReleaseManifest,
    /// SHA-256 of canonical manifest JSON.
    pub manifest_digest: [u8; 32],
    /// Release signing key identity.
    pub key_id: String,
    /// Ed25519 signature over the manifest digest.
    pub signature: Vec<u8>,
}

/// Hermetic manifest builder.
pub struct ReleaseManifestBuilder;
impl ReleaseManifestBuilder {
    /// Validates completeness, sorts canonically, and signs exact evidence.
    pub fn build_and_sign(
        release_id: &str,
        target: &str,
        hardware_profile: &str,
        mut artifacts: Vec<ReleaseArtifact>,
        key_id: &str,
        key: &SigningKey,
    ) -> Result<SignedReleaseManifest, ReleaseEvidenceError> {
        if [release_id, target, hardware_profile, key_id]
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(ReleaseEvidenceError::InvalidManifest);
        }
        artifacts.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.name.cmp(&right.name))
        });
        validate_artifacts(&artifacts)?;
        let manifest = ReleaseManifest {
            schema: "ste-release-manifest-v1".into(),
            release_id: release_id.into(),
            target: target.into(),
            hardware_profile: hardware_profile.into(),
            artifacts,
        };
        let manifest_digest = canonical_digest(&manifest)?;
        Ok(SignedReleaseManifest {
            manifest,
            manifest_digest,
            key_id: key_id.into(),
            signature: key.sign(&manifest_digest).to_bytes().to_vec(),
        })
    }
}

impl SignedReleaseManifest {
    /// Verifies completeness, canonical order, digest, and signature.
    pub fn verify(&self, key: &VerifyingKey) -> Result<(), ReleaseEvidenceError> {
        if self.manifest.schema != "ste-release-manifest-v1" || self.key_id.trim().is_empty() {
            return Err(ReleaseEvidenceError::InvalidManifest);
        }
        validate_artifacts(&self.manifest.artifacts)?;
        if canonical_digest(&self.manifest)? != self.manifest_digest {
            return Err(ReleaseEvidenceError::DigestMismatch);
        }
        let signature = Signature::from_slice(&self.signature)
            .map_err(|_| ReleaseEvidenceError::InvalidSignature)?;
        key.verify(&self.manifest_digest, &signature)
            .map_err(|_| ReleaseEvidenceError::InvalidSignature)
    }
}

fn validate_artifacts(artifacts: &[ReleaseArtifact]) -> Result<(), ReleaseEvidenceError> {
    if artifacts.iter().any(|artifact| {
        artifact.name.trim().is_empty() || artifact.bytes == 0 || artifact.sha256 == [0; 32]
    }) {
        return Err(ReleaseEvidenceError::InvalidArtifact);
    }
    if !artifacts.windows(2).all(|pair| pair[0].kind < pair[1].kind) {
        return Err(ReleaseEvidenceError::DuplicateOrNonCanonical);
    }
    let present: BTreeSet<_> = artifacts.iter().map(|artifact| artifact.kind).collect();
    if ReleaseArtifactKind::ALL
        .iter()
        .any(|kind| !present.contains(kind))
    {
        return Err(ReleaseEvidenceError::MissingRequiredArtifact);
    }
    Ok(())
}
fn canonical_digest(manifest: &ReleaseManifest) -> Result<[u8; 32], ReleaseEvidenceError> {
    serde_json::to_vec(manifest)
        .map(|bytes| Sha256::digest(bytes).into())
        .map_err(|_| ReleaseEvidenceError::Serialization)
}

/// Immutable release channel in required promotion order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseChannel {
    /// Exact signed production candidate under verification.
    Candidate,
    /// Same digest deployed to a bounded commercial pilot.
    Pilot,
    /// Same digest explicitly approved for production.
    Production,
}

/// Records channel pointers while forbidding rebuilds between stages.
#[derive(Clone, Debug, Default)]
pub struct ChannelPromotionLedger {
    candidate: Option<[u8; 32]>,
    pilot: Option<[u8; 32]>,
    production: Option<[u8; 32]>,
}
impl ChannelPromotionLedger {
    /// Promotes only the exact candidate digest in strict order.
    pub fn promote(
        &mut self,
        channel: ReleaseChannel,
        digest: [u8; 32],
    ) -> Result<(), ReleaseEvidenceError> {
        if digest == [0; 32] {
            return Err(ReleaseEvidenceError::DigestMismatch);
        }
        match channel {
            ReleaseChannel::Candidate if self.candidate.is_none() => self.candidate = Some(digest),
            ReleaseChannel::Pilot if self.candidate == Some(digest) && self.pilot.is_none() => {
                self.pilot = Some(digest)
            }
            ReleaseChannel::Production
                if self.pilot == Some(digest) && self.production.is_none() =>
            {
                self.production = Some(digest)
            }
            _ => return Err(ReleaseEvidenceError::NonIdenticalOrInvalidPromotion),
        }
        Ok(())
    }
    /// Returns the digest pinned to a channel.
    #[must_use]
    pub const fn digest(&self, channel: ReleaseChannel) -> Option<[u8; 32]> {
        match channel {
            ReleaseChannel::Candidate => self.candidate,
            ReleaseChannel::Pilot => self.pilot,
            ReleaseChannel::Production => self.production,
        }
    }
}

/// Release evidence verification failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseEvidenceError {
    /// Manifest identity is malformed.
    InvalidManifest,
    /// Artifact metadata/digest is malformed.
    InvalidArtifact,
    /// A mandatory evidence class is absent.
    MissingRequiredArtifact,
    /// Artifact categories repeat or are non-canonical.
    DuplicateOrNonCanonical,
    /// Manifest serialization failed.
    Serialization,
    /// Manifest content does not match its digest.
    DigestMismatch,
    /// Signature is malformed or invalid.
    InvalidSignature,
    /// Channel order or digest identity was violated.
    NonIdenticalOrInvalidPromotion,
}
