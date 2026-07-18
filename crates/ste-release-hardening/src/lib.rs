#![forbid(unsafe_code)]
//! Release, recovery, integrity, and incident hardening primitives.

use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};
use zeroize::Zeroize;

/// A/B deployment slot.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Slot {
    /// First slot.
    A,
    /// Second slot.
    B,
}
impl Slot {
    const fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

/// Signed release metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpdateManifest {
    /// Strictly monotonic release counter.
    pub version: u64,
    /// Artifact digest.
    pub payload_digest: [u8; 32],
    /// Exact hardware profile.
    pub hardware: String,
    /// Exact configuration schema.
    pub config_schema: u32,
    /// Explicitly signed emergency rollback exception.
    pub emergency_rollback: bool,
}
/// Signed immutable update artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpdateBundle {
    /// Manifest.
    pub manifest: UpdateManifest,
    /// Release payload.
    pub payload: Vec<u8>,
    /// Detached signature over manifest.
    pub signature: Vec<u8>,
}
impl UpdateBundle {
    /// Builds and signs a release bundle.
    pub fn sign(
        version: u64,
        payload: Vec<u8>,
        hardware: impl Into<String>,
        config_schema: u32,
        emergency_rollback: bool,
        key: &SigningKey,
    ) -> Result<Self, HardeningError> {
        if version == 0 || payload.is_empty() || config_schema == 0 {
            return Err(HardeningError::InvalidUpdate);
        }
        let manifest = UpdateManifest {
            version,
            payload_digest: Sha256::digest(&payload).into(),
            hardware: hardware.into(),
            config_schema,
            emergency_rollback,
        };
        if manifest.hardware.trim().is_empty() {
            return Err(HardeningError::InvalidUpdate);
        }
        let signature = key
            .sign(&serde_json::to_vec(&manifest).map_err(|_| HardeningError::Serialization)?)
            .to_bytes()
            .to_vec();
        Ok(Self {
            manifest,
            payload,
            signature,
        })
    }
    /// Verifies signature, payload integrity, and exact platform compatibility.
    pub fn verify(
        &self,
        key: &VerifyingKey,
        hardware: &str,
        schema: u32,
    ) -> Result<VerifiedUpdate<'_>, HardeningError> {
        let signature =
            Signature::from_slice(&self.signature).map_err(|_| HardeningError::InvalidSignature)?;
        key.verify(
            &serde_json::to_vec(&self.manifest).map_err(|_| HardeningError::Serialization)?,
            &signature,
        )
        .map_err(|_| HardeningError::InvalidSignature)?;
        if Sha256::digest(&self.payload).as_slice() != self.manifest.payload_digest
            || self.manifest.hardware != hardware
            || self.manifest.config_schema != schema
        {
            return Err(HardeningError::IncompatibleOrCorrupt);
        }
        Ok(VerifiedUpdate(self))
    }
}
/// Proof of update verification.
#[derive(Clone, Copy, Debug)]
pub struct VerifiedUpdate<'a>(&'a UpdateBundle);
/// Atomic A/B update manager.
#[derive(Clone, Debug)]
pub struct AbUpdateManager {
    active: Slot,
    versions: BTreeMap<Slot, u64>,
    pending: Option<Slot>,
    previous: Option<Slot>,
    evidence: Vec<IncidentEvidence>,
}
impl AbUpdateManager {
    /// Creates a manager from the known-good active version.
    #[must_use]
    pub fn new(version: u64) -> Self {
        Self {
            active: Slot::A,
            versions: [(Slot::A, version)].into_iter().collect(),
            pending: None,
            previous: None,
            evidence: Vec::new(),
        }
    }
    /// Stages only to inactive slot and prevents unsigned downgrade.
    pub fn stage(&mut self, update: VerifiedUpdate<'_>) -> Result<Slot, HardeningError> {
        if self.pending.is_some() {
            return Err(HardeningError::UpdatePending);
        }
        let version = update.0.manifest.version;
        let current = self.versions.get(&self.active).copied().unwrap_or(0);
        if version <= current && !update.0.manifest.emergency_rollback {
            return Err(HardeningError::Downgrade);
        }
        let target = self.active.other();
        self.versions.insert(target, version);
        self.pending = Some(target);
        self.previous = Some(self.active);
        self.evidence.push(IncidentEvidence::new(
            "update-staged",
            format!("version={version};slot={target:?}"),
        ));
        Ok(target)
    }
    /// Atomically boots staged slot; health confirmation is still required.
    pub fn activate_pending(&mut self) -> Result<(), HardeningError> {
        let target = self.pending.ok_or(HardeningError::NoPendingUpdate)?;
        self.active = target;
        self.evidence.push(IncidentEvidence::new(
            "update-activated",
            format!("slot={target:?}"),
        ));
        Ok(())
    }
    /// Commits a healthy activation and clears rollback marker.
    pub fn confirm_health(&mut self) -> Result<(), HardeningError> {
        if self.pending != Some(self.active) {
            return Err(HardeningError::NoPendingUpdate);
        }
        self.pending = None;
        self.previous = None;
        self.evidence
            .push(IncidentEvidence::new("update-healthy", "committed"));
        Ok(())
    }
    /// Atomically restores previous known-good slot after failed health check.
    pub fn rollback(&mut self, reason: impl Into<String>) -> Result<(), HardeningError> {
        let previous = self.previous.ok_or(HardeningError::NoRollbackTarget)?;
        self.active = previous;
        self.pending = None;
        self.previous = None;
        self.evidence
            .push(IncidentEvidence::new("update-rollback", reason));
        Ok(())
    }
    /// Active slot/version.
    #[must_use]
    pub fn active(&self) -> (Slot, u64) {
        (
            self.active,
            self.versions.get(&self.active).copied().unwrap_or(0),
        )
    }
    /// Immutable incident evidence.
    #[must_use]
    pub fn evidence(&self) -> &[IncidentEvidence] {
        &self.evidence
    }
}

/// Authenticated encrypted backup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncryptedBackup {
    /// Backup schema.
    pub schema: u32,
    /// Unique XChaCha20 nonce.
    pub nonce: [u8; 24],
    /// Authenticated ciphertext.
    pub ciphertext: Vec<u8>,
}
impl EncryptedBackup {
    /// Encrypts a local backup under a separate 256-bit key.
    pub fn create(
        schema: u32,
        plaintext: &[u8],
        key: &[u8; 32],
        nonce: [u8; 24],
    ) -> Result<Self, HardeningError> {
        if schema == 0 || plaintext.is_empty() {
            return Err(HardeningError::InvalidBackup);
        }
        let ciphertext = XChaCha20Poly1305::new(key.into())
            .encrypt(XNonce::from_slice(&nonce), plaintext)
            .map_err(|_| HardeningError::BackupAuthentication)?;
        Ok(Self {
            schema,
            nonce,
            ciphertext,
        })
    }
    /// Authenticates and restores only the exact supported schema.
    pub fn restore(&self, schema: u32, key: &[u8; 32]) -> Result<Vec<u8>, HardeningError> {
        if schema != self.schema {
            return Err(HardeningError::UnsupportedMigration);
        }
        XChaCha20Poly1305::new(key.into())
            .decrypt(XNonce::from_slice(&self.nonce), self.ciphertext.as_ref())
            .map_err(|_| HardeningError::BackupAuthentication)
    }
}

/// Hash-chained journal entry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JournalEntry {
    /// Monotonic sequence.
    pub sequence: u64,
    /// Schema version.
    pub schema: u32,
    /// Previous entry digest or zero genesis.
    pub previous_digest: [u8; 32],
    /// Payload.
    pub payload: Vec<u8>,
    /// This entry digest.
    pub digest: [u8; 32],
}
impl JournalEntry {
    /// Appends a content-addressed entry.
    pub fn append(
        sequence: u64,
        schema: u32,
        previous_digest: [u8; 32],
        payload: Vec<u8>,
    ) -> Result<Self, HardeningError> {
        if sequence == 0 || schema == 0 || payload.is_empty() {
            return Err(HardeningError::InvalidJournal);
        }
        let digest = journal_digest(sequence, schema, previous_digest, &payload);
        Ok(Self {
            sequence,
            schema,
            previous_digest,
            payload,
            digest,
        })
    }
}
fn journal_digest(sequence: u64, schema: u32, previous: [u8; 32], payload: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(sequence.to_be_bytes());
    hash.update(schema.to_be_bytes());
    hash.update(previous);
    hash.update(payload);
    hash.finalize().into()
}
/// Validates complete ordering, hash chain, and corruption absence.
pub fn validate_journal(entries: &[JournalEntry]) -> Result<(), HardeningError> {
    let mut prior = [0; 32];
    for (index, entry) in entries.iter().enumerate() {
        if entry.sequence != index as u64 + 1
            || entry.previous_digest != prior
            || entry.digest != journal_digest(entry.sequence, entry.schema, prior, &entry.payload)
        {
            return Err(HardeningError::JournalCorrupt);
        }
        prior = entry.digest;
    }
    Ok(())
}
/// Migrates only one schema step while preserving old journal evidence.
pub fn migrate_journal(
    entries: &[JournalEntry],
    from: u32,
    to: u32,
) -> Result<Vec<JournalEntry>, HardeningError> {
    validate_journal(entries)?;
    if to != from + 1 || entries.iter().any(|e| e.schema != from) {
        return Err(HardeningError::UnsupportedMigration);
    }
    let mut migrated = Vec::new();
    let mut prior = [0; 32];
    for entry in entries {
        let next = JournalEntry::append(entry.sequence, to, prior, entry.payload.clone())?;
        prior = next.digest;
        migrated.push(next);
    }
    Ok(migrated)
}

/// Verification-key rotation and compromise registry.
#[derive(Clone, Debug, Default)]
pub struct VerificationKeyRing {
    keys: BTreeMap<String, VerifyingKey>,
    revoked: BTreeSet<String>,
    active: Option<String>,
}
impl VerificationKeyRing {
    /// Adds and activates a new verification key.
    pub fn rotate(
        &mut self,
        id: impl Into<String>,
        key: VerifyingKey,
    ) -> Result<(), HardeningError> {
        let id = id.into();
        if id.trim().is_empty() || self.revoked.contains(&id) {
            return Err(HardeningError::InvalidKey);
        }
        self.keys.insert(id.clone(), key);
        self.active = Some(id);
        Ok(())
    }
    /// Marks a compromised key unusable immediately.
    pub fn compromise(&mut self, id: &str) -> Result<(), HardeningError> {
        if !self.keys.contains_key(id) {
            return Err(HardeningError::InvalidKey);
        }
        self.revoked.insert(id.into());
        if self.active.as_deref() == Some(id) {
            self.active = None;
        }
        Ok(())
    }
    /// Returns only a non-revoked key.
    #[must_use]
    pub fn trusted(&self, id: &str) -> Option<&VerifyingKey> {
        (!self.revoked.contains(id))
            .then(|| self.keys.get(id))
            .flatten()
    }
}

/// Payload-minimized incident/recovery evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncidentEvidence {
    /// Stable category.
    pub category: String,
    /// Redacted detail.
    pub detail: String,
}
impl IncidentEvidence {
    fn new(category: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            detail: detail.into(),
        }
    }
}
/// Factory-reset result proving cryptographic erasure without retaining secrets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetEvidence {
    /// Keys erased.
    pub erased_key_count: usize,
    /// Reset reason.
    pub reason: String,
}
/// Zeroizes supplied key material and returns minimized evidence.
pub fn factory_reset(
    keys: &mut [Vec<u8>],
    reason: impl Into<String>,
) -> Result<ResetEvidence, HardeningError> {
    let reason = reason.into();
    if reason.trim().is_empty() {
        return Err(HardeningError::InvalidReset);
    }
    for key in keys.iter_mut() {
        key.zeroize();
        key.clear();
    }
    Ok(ResetEvidence {
        erased_key_count: keys.len(),
        reason,
    })
}

/// Stable hardening failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HardeningError {
    /// Update malformed.
    InvalidUpdate,
    /// Signature invalid.
    InvalidSignature,
    /// Compatibility or integrity failed.
    IncompatibleOrCorrupt,
    /// Unsanctioned downgrade.
    Downgrade,
    /// Another update pending.
    UpdatePending,
    /// No pending update.
    NoPendingUpdate,
    /// Rollback unavailable.
    NoRollbackTarget,
    /// Serialization failed.
    Serialization,
    /// Backup malformed.
    InvalidBackup,
    /// Backup authentication failed.
    BackupAuthentication,
    /// Journal malformed.
    InvalidJournal,
    /// Journal corruption detected.
    JournalCorrupt,
    /// Migration unsupported.
    UnsupportedMigration,
    /// Key invalid/revoked.
    InvalidKey,
    /// Reset reason invalid.
    InvalidReset,
}
impl fmt::Display for HardeningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "release hardening rejected operation: {self:?}")
    }
}
impl Error for HardeningError {}
