//! Retention, portable export, deletion, reset, and decommission orchestration.

use crate::DataClass;
use crate::crypto::{CryptoError, EncryptedEnvelope, EnvelopeCipher, KeyProvider};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Runtime state after destructive lifecycle operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceLifecycleState {
    /// Device is provisioned; policy still independently gates capture.
    Operational,
    /// Device cannot capture until an authorized provisioning workflow completes.
    CaptureDisabled,
    /// Device was terminally retired and must not be reprovisioned in place.
    Decommissioned,
}

/// Explicit per-class lifecycle policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClassLifecyclePolicy {
    /// Governed storage class.
    pub data_class: DataClass,
    /// Maximum authorized lifetime in seconds.
    pub retention_seconds: u64,
    /// Whether participant-authorized portable export is permitted.
    pub export_allowed: bool,
    /// Whether this class may be restored from an eligible backup.
    pub backup_allowed: bool,
}

/// Narrow store port; implementations enumerate deletion instead of hiding it.
pub trait LifecycleStore: Send + Sync {
    /// Stable non-sensitive store name used in receipts.
    fn name(&self) -> &str;
    /// Deletes one subject/class and reports the number of removed records.
    fn delete_subject(&self, subject: &str, class: DataClass) -> Result<u64, LifecycleError>;
    /// Clears all eligible contents during reset or decommissioning.
    fn reset(&self) -> Result<(), LifecycleError>;
}

/// One store/class deletion result suitable for a payload-free receipt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeletionStep {
    /// Store that was visited.
    pub store: String,
    /// Class deleted from that store.
    pub data_class: DataClass,
    /// Number of records reported deleted.
    pub deleted_records: u64,
}

/// Proof that every registered store and class was visited.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeletionReceipt {
    /// Pseudonymous or hashed subject reference, never the source identity.
    pub subject_reference: String,
    /// Deterministic completion time.
    pub completed_at: u64,
    /// Explicit matrix of visited stores and classes.
    pub steps: Vec<DeletionStep>,
    /// Whether associated class keys were destroyed after store deletion.
    pub cryptographic_erasure: bool,
}

/// Secret-free manifest bound as AEAD associated data.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PortableExportManifest {
    /// Manifest schema version.
    pub version: u16,
    /// Unique non-secret export identifier.
    pub export_id: String,
    /// Public originating device identity.
    pub device_id: String,
    /// Exported data class.
    pub data_class: DataClass,
    /// Exact authorized export purpose.
    pub purpose: String,
    /// Non-secret authorization record reference.
    pub authorization_reference: String,
    /// Deterministic creation time.
    pub created_at: u64,
    /// Exclusive validity endpoint.
    pub expires_at: u64,
    /// Payload schema version.
    pub schema_version: u16,
    /// Public recipient-key identifier, not key material.
    pub recipient_key_id: String,
}

impl PortableExportManifest {
    /// Canonical bytes are authenticated but remain non-secret metadata.
    pub fn authenticated_bytes(&self) -> Result<Vec<u8>, LifecycleError> {
        serde_json::to_vec(self).map_err(|_| LifecycleError::InvalidManifest)
    }

    /// Rejects expired/malformed imports before decryption or store mutation.
    pub fn validate(&self, now: u64) -> Result<(), LifecycleError> {
        if self.version != 1
            || self.schema_version == 0
            || self.export_id.trim().is_empty()
            || self.device_id.trim().is_empty()
            || self.purpose.trim().is_empty()
            || self.authorization_reference.trim().is_empty()
            || self.recipient_key_id.trim().is_empty()
            || self.expires_at <= self.created_at
            || now >= self.expires_at
        {
            return Err(LifecycleError::InvalidManifest);
        }
        Ok(())
    }
}

/// Portable encrypted payload and its authenticated, secret-free manifest.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PortableEncryptedExport {
    /// Authenticated, secret-free metadata.
    pub manifest: PortableExportManifest,
    /// Authenticated encrypted payload.
    pub envelope: EncryptedEnvelope,
}

impl PortableEncryptedExport {
    /// Creates an export only for an explicit eligible policy.
    pub fn create(
        provider: &dyn KeyProvider,
        policy: ClassLifecyclePolicy,
        manifest: PortableExportManifest,
        plaintext: &[u8],
    ) -> Result<Self, LifecycleError> {
        if !policy.export_allowed || policy.data_class != manifest.data_class {
            return Err(LifecycleError::ExportForbidden);
        }
        manifest.validate(manifest.created_at)?;
        let aad = manifest.authenticated_bytes()?;
        let envelope = EnvelopeCipher::encrypt(provider, manifest.data_class, plaintext, &aad)?;
        Ok(Self { manifest, envelope })
    }

    /// Authenticates manifest and ciphertext before returning zeroizing plaintext.
    pub fn restore(
        &self,
        provider: &dyn KeyProvider,
        policy: ClassLifecyclePolicy,
        now: u64,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, LifecycleError> {
        if !policy.backup_allowed || policy.data_class != self.manifest.data_class {
            return Err(LifecycleError::RestoreForbidden);
        }
        self.manifest.validate(now)?;
        let aad = self.manifest.authenticated_bytes()?;
        EnvelopeCipher::decrypt(provider, &self.envelope, &aad).map_err(Into::into)
    }
}

/// Coordinates lifecycle effects across an explicit registry.
pub struct LifecycleManager {
    stores: Vec<Arc<dyn LifecycleStore>>,
    keys: Arc<dyn KeyProvider>,
    state: Mutex<DeviceLifecycleState>,
}

impl LifecycleManager {
    /// Creates a manager from the complete store registry and managed key provider.
    #[must_use]
    pub fn new(stores: Vec<Arc<dyn LifecycleStore>>, keys: Arc<dyn KeyProvider>) -> Self {
        Self {
            stores,
            keys,
            state: Mutex::new(DeviceLifecycleState::Operational),
        }
    }

    /// Deletes a subject from every registered store/class, then erases class keys.
    /// Any incomplete step returns an error and no successful receipt.
    pub fn delete_everywhere(
        &self,
        subject_reference: &str,
        classes: &[DataClass],
        completed_at: u64,
    ) -> Result<DeletionReceipt, LifecycleError> {
        if subject_reference.trim().is_empty() || self.stores.is_empty() || classes.is_empty() {
            return Err(LifecycleError::IncompleteRegistry);
        }
        let mut steps = Vec::with_capacity(self.stores.len() * classes.len());
        for store in &self.stores {
            for class in classes {
                steps.push(DeletionStep {
                    store: store.name().to_owned(),
                    data_class: *class,
                    deleted_records: store.delete_subject(subject_reference, *class)?,
                });
            }
        }
        for class in classes {
            self.keys.erase(*class)?;
        }
        Ok(DeletionReceipt {
            subject_reference: subject_reference.to_owned(),
            completed_at,
            steps,
            cryptographic_erasure: true,
        })
    }

    /// Clears all registered stores and keys, returning capture-disabled state.
    pub fn factory_reset(&self) -> Result<DeviceLifecycleState, LifecycleError> {
        for store in &self.stores {
            store.reset()?;
        }
        self.keys.erase_all()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| LifecycleError::StateUnavailable)?;
        *state = DeviceLifecycleState::CaptureDisabled;
        Ok(*state)
    }

    /// Performs reset and makes the device terminally decommissioned.
    pub fn decommission(&self) -> Result<DeviceLifecycleState, LifecycleError> {
        self.factory_reset()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| LifecycleError::StateUnavailable)?;
        *state = DeviceLifecycleState::Decommissioned;
        Ok(*state)
    }

    /// Returns current state, failing safe to capture-disabled if poisoned.
    #[must_use]
    pub fn state(&self) -> DeviceLifecycleState {
        self.state
            .lock()
            .map_or(DeviceLifecycleState::CaptureDisabled, |state| *state)
    }
}

/// Lifecycle failure never embeds payload, key, or subject data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleError {
    /// Underlying cryptographic failure.
    Crypto(CryptoError),
    /// Class policy does not permit export.
    ExportForbidden,
    /// Class policy does not permit restore.
    RestoreForbidden,
    /// Manifest is malformed, inconsistent, or expired.
    InvalidManifest,
    /// Store/class registry cannot prove complete propagation.
    IncompleteRegistry,
    /// A registered store failed its lifecycle operation.
    StoreFailure,
    /// Lifecycle state lock is unavailable; callers must fail safe.
    StateUnavailable,
}

impl From<CryptoError> for LifecycleError {
    fn from(value: CryptoError) -> Self {
        Self::Crypto(value)
    }
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Crypto(_) => "cryptographic lifecycle operation failed",
            Self::ExportForbidden => "export is forbidden by lifecycle policy",
            Self::RestoreForbidden => "restore is forbidden by lifecycle policy",
            Self::InvalidManifest => "portable manifest is invalid",
            Self::IncompleteRegistry => "lifecycle store registry is incomplete",
            Self::StoreFailure => "lifecycle store operation failed",
            Self::StateUnavailable => "lifecycle state unavailable",
        })
    }
}

impl Error for LifecycleError {}
