//! Versioned envelope encryption and managed-key abstractions.

use crate::DataClass;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::Mutex;
use zeroize::Zeroizing;

/// Strength of identity/key protection available on this device.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AssuranceLevel {
    /// Key is held by an approved hardware-backed provider.
    ProductionHardwareBacked,
    /// Explicit local development fallback; never production-claimable.
    ProtectedDevelopmentFallback,
}

/// Public device identity boundary. Private signing material stays in its provider.
pub trait DeviceIdentity: Send + Sync {
    /// Stable, non-secret device identifier.
    fn device_id(&self) -> &str;
    /// Assurance of the underlying private identity.
    fn assurance_level(&self) -> AssuranceLevel;
}

/// Non-secret local identity descriptor.
#[derive(Clone, Debug)]
pub struct LocalDeviceIdentity {
    id: String,
    assurance: AssuranceLevel,
}

impl LocalDeviceIdentity {
    /// Creates an identity descriptor, rejecting empty identifiers.
    pub fn new(id: impl Into<String>, assurance: AssuranceLevel) -> Result<Self, CryptoError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(CryptoError::InvalidIdentity);
        }
        Ok(Self { id, assurance })
    }

    /// Production startup must call this and reject development fallback.
    pub fn require_production_assurance(&self) -> Result<(), CryptoError> {
        match self.assurance {
            AssuranceLevel::ProductionHardwareBacked => Ok(()),
            AssuranceLevel::ProtectedDevelopmentFallback => Err(CryptoError::DevelopmentKey),
        }
    }
}

impl DeviceIdentity for LocalDeviceIdentity {
    fn device_id(&self) -> &str {
        &self.id
    }

    fn assurance_level(&self) -> AssuranceLevel {
        self.assurance
    }
}

/// Key copied only into zeroizing memory for one cryptographic operation.
#[derive(Clone)]
pub struct DataKey {
    /// Non-secret rotation identifier.
    pub key_id: String,
    /// Secret bytes zeroized when dropped.
    pub secret: Zeroizing<[u8; 32]>,
    /// Provider assurance.
    pub assurance: AssuranceLevel,
}

/// Per-data-class managed key boundary.
pub trait KeyProvider: Send + Sync {
    /// Returns current key material in zeroizing memory.
    fn current_key(&self, class: DataClass) -> Result<DataKey, CryptoError>;
    /// Resolves an older key for decrypting a versioned envelope.
    fn key_by_id(&self, class: DataClass, key_id: &str) -> Result<DataKey, CryptoError>;
    /// Creates and activates a fresh class-specific key.
    fn rotate(&self, class: DataClass) -> Result<String, CryptoError>;
    /// Cryptographically erases every key for a class.
    fn erase(&self, class: DataClass) -> Result<(), CryptoError>;
    /// Erases all managed data keys during reset/decommission.
    fn erase_all(&self) -> Result<(), CryptoError>;
}

#[derive(Clone)]
struct KeyEntry {
    id: String,
    bytes: Zeroizing<[u8; 32]>,
}

#[derive(Default)]
struct KeyRing {
    generation: u64,
    keys: BTreeMap<DataClass, Vec<KeyEntry>>,
}

/// Protected software fallback for development and tests.
pub struct DevelopmentKeyProvider {
    ring: Mutex<KeyRing>,
}

impl Default for DevelopmentKeyProvider {
    fn default() -> Self {
        Self {
            ring: Mutex::new(KeyRing::default()),
        }
    }
}

impl DevelopmentKeyProvider {
    /// Creates a development provider. Its assurance can never be upgraded.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn generate(ring: &mut KeyRing, class: DataClass) -> String {
        ring.generation += 1;
        let id = format!("dev-key-{}", ring.generation);
        let mut bytes = Zeroizing::new([0_u8; 32]);
        OsRng.fill_bytes(bytes.as_mut());
        ring.keys.entry(class).or_default().push(KeyEntry {
            id: id.clone(),
            bytes,
        });
        id
    }
}

impl KeyProvider for DevelopmentKeyProvider {
    fn current_key(&self, class: DataClass) -> Result<DataKey, CryptoError> {
        let mut ring = self.ring.lock().map_err(|_| CryptoError::KeyUnavailable)?;
        if ring.keys.get(&class).is_none_or(Vec::is_empty) {
            Self::generate(&mut ring, class);
        }
        let entry = ring
            .keys
            .get(&class)
            .and_then(|keys| keys.last())
            .ok_or(CryptoError::KeyUnavailable)?;
        Ok(DataKey {
            key_id: entry.id.clone(),
            secret: entry.bytes.clone(),
            assurance: AssuranceLevel::ProtectedDevelopmentFallback,
        })
    }

    fn key_by_id(&self, class: DataClass, key_id: &str) -> Result<DataKey, CryptoError> {
        let ring = self.ring.lock().map_err(|_| CryptoError::KeyUnavailable)?;
        let entry = ring
            .keys
            .get(&class)
            .and_then(|keys| keys.iter().find(|key| key.id == key_id))
            .ok_or(CryptoError::KeyUnavailable)?;
        Ok(DataKey {
            key_id: entry.id.clone(),
            secret: entry.bytes.clone(),
            assurance: AssuranceLevel::ProtectedDevelopmentFallback,
        })
    }

    fn rotate(&self, class: DataClass) -> Result<String, CryptoError> {
        let mut ring = self.ring.lock().map_err(|_| CryptoError::KeyUnavailable)?;
        Ok(Self::generate(&mut ring, class))
    }

    fn erase(&self, class: DataClass) -> Result<(), CryptoError> {
        self.ring
            .lock()
            .map_err(|_| CryptoError::KeyUnavailable)?
            .keys
            .remove(&class);
        Ok(())
    }

    fn erase_all(&self) -> Result<(), CryptoError> {
        self.ring
            .lock()
            .map_err(|_| CryptoError::KeyUnavailable)?
            .keys
            .clear();
        Ok(())
    }
}

/// Portable, versioned authenticated-encryption envelope. It contains no key.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EncryptedEnvelope {
    /// Envelope schema.
    pub version: u16,
    /// Fixed algorithm identifier for dispatch/agility.
    pub algorithm: String,
    /// Governed class selecting the key partition.
    pub data_class: DataClass,
    /// Non-secret rotation identifier.
    pub key_id: String,
    /// Unique XChaCha20 nonce.
    pub nonce: Vec<u8>,
    /// Authenticated ciphertext and tag.
    pub ciphertext: Vec<u8>,
    /// Digest for early associated-data mismatch diagnostics.
    pub aad_sha256: Vec<u8>,
    /// Assurance at encryption time; development is visibly distinct.
    pub assurance: AssuranceLevel,
}

/// Stateless AEAD facade.
pub struct EnvelopeCipher;

impl EnvelopeCipher {
    /// Encrypts with fresh randomness and caller-supplied associated data.
    pub fn encrypt(
        provider: &dyn KeyProvider,
        class: DataClass,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<EncryptedEnvelope, CryptoError> {
        let key = provider.current_key(class)?;
        let cipher = XChaCha20Poly1305::new((&*key.secret).into());
        let mut nonce = [0_u8; 24];
        OsRng.fill_bytes(&mut nonce);
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::AuthenticationFailed)?;
        Ok(EncryptedEnvelope {
            version: 1,
            algorithm: "XChaCha20-Poly1305".into(),
            data_class: class,
            key_id: key.key_id,
            nonce: nonce.to_vec(),
            ciphertext,
            aad_sha256: Sha256::digest(aad).to_vec(),
            assurance: key.assurance,
        })
    }

    /// Authenticates and decrypts; tampering, wrong AAD, or erased keys fail closed.
    pub fn decrypt(
        provider: &dyn KeyProvider,
        envelope: &EncryptedEnvelope,
        aad: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
        if envelope.version != 1
            || envelope.algorithm != "XChaCha20-Poly1305"
            || envelope.nonce.len() != 24
        {
            return Err(CryptoError::UnsupportedEnvelope);
        }
        if envelope.aad_sha256.as_slice() != Sha256::digest(aad).as_slice() {
            return Err(CryptoError::AuthenticationFailed);
        }
        let key = provider.key_by_id(envelope.data_class, &envelope.key_id)?;
        let cipher = XChaCha20Poly1305::new((&*key.secret).into());
        cipher
            .decrypt(
                XNonce::from_slice(&envelope.nonce),
                Payload {
                    msg: &envelope.ciphertext,
                    aad,
                },
            )
            .map(Zeroizing::new)
            .map_err(|_| CryptoError::AuthenticationFailed)
    }
}

/// Stable cryptographic failure without secret data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CryptoError {
    /// Device identifier was empty or malformed.
    InvalidIdentity,
    /// A development-only key was presented to a production gate.
    DevelopmentKey,
    /// Requested current or historical key cannot be obtained.
    KeyUnavailable,
    /// Envelope version, algorithm, or nonce shape is unsupported.
    UnsupportedEnvelope,
    /// AEAD authentication or associated-data verification failed.
    AuthenticationFailed,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentity => "invalid device identity",
            Self::DevelopmentKey => "development key cannot claim production assurance",
            Self::KeyUnavailable => "encryption key unavailable",
            Self::UnsupportedEnvelope => "unsupported encryption envelope",
            Self::AuthenticationFailed => "authentication failed",
        })
    }
}

impl Error for CryptoError {}
