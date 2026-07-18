#![forbid(unsafe_code)]
//! Signed optional-sidecar contracts and failure-isolated supervision.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

fn required(value: impl Into<String>, label: &'static str) -> Result<String, SidecarError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 256 {
        Err(SidecarError::InvalidValue(label))
    } else {
        Ok(value)
    }
}

/// Maximum resources granted to an unprivileged local sidecar.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceLimits {
    /// CPU millicores.
    pub cpu_millicores: u32,
    /// Resident-memory bytes.
    pub memory_bytes: u64,
    /// Storage bytes.
    pub storage_bytes: u64,
    /// Maximum requests in each fixed minute.
    pub requests_per_minute: u32,
}
impl ResourceLimits {
    /// Creates positive, bounded resources.
    pub fn new(cpu: u32, memory: u64, storage: u64, rate: u32) -> Result<Self, SidecarError> {
        if cpu == 0
            || cpu > 1_000
            || memory == 0
            || memory > 512 * 1024 * 1024
            || storage > 1024 * 1024 * 1024
            || rate == 0
            || rate > 600
        {
            Err(SidecarError::InvalidResources)
        } else {
            Ok(Self {
                cpu_millicores: cpu,
                memory_bytes: memory,
                storage_bytes: storage,
                requests_per_minute: rate,
            })
        }
    }
}

/// Core-authorized ceiling that a sidecar can only narrow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreAuthority {
    /// Authorized purposes.
    pub purposes: BTreeSet<String>,
    /// Authorized capabilities.
    pub capabilities: BTreeSet<String>,
    /// Highest allowed claim level.
    pub claim_level: ClaimLevel,
}
/// Claim ceiling intentionally excludes medical/high-impact claims.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ClaimLevel {
    /// Offline experimental tooling only.
    Experimental,
    /// Validated non-medical claim.
    ValidatedNonMedical,
}

/// Immutable allowlisted sidecar manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SidecarManifest {
    /// Sidecar identity and version.
    pub sidecar_id: String,
    /// Executable/package content digest.
    pub executable_digest: String,
    /// Generated local IPC contract version.
    pub contract_version: u32,
    /// Exact allowlisted method names.
    pub methods: BTreeSet<String>,
    /// Narrow authorized purposes.
    pub purposes: BTreeSet<String>,
    /// Narrow authorized capabilities.
    pub capabilities: BTreeSet<String>,
    /// Claim ceiling.
    pub claim_level: ClaimLevel,
    /// Must remain true unless separately reviewed.
    pub offline: bool,
    /// Must always remain false.
    pub hardware_access: bool,
    /// Must always remain false.
    pub authoritative_store_access: bool,
    /// Enforced resource limits.
    pub resources: ResourceLimits,
}
impl SidecarManifest {
    /// Creates a complete manifest; signature and authority validation occur separately.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        digest: impl Into<String>,
        contract_version: u32,
        methods: BTreeSet<String>,
        purposes: BTreeSet<String>,
        capabilities: BTreeSet<String>,
        claim_level: ClaimLevel,
        offline: bool,
        hardware_access: bool,
        store_access: bool,
        resources: ResourceLimits,
    ) -> Result<Self, SidecarError> {
        if contract_version == 0
            || methods.is_empty()
            || methods.iter().any(|m| m.trim().is_empty())
        {
            return Err(SidecarError::InvalidContract);
        }
        Ok(Self {
            sidecar_id: required(id, "sidecar identifier")?,
            executable_digest: required(digest, "executable digest")?,
            contract_version,
            methods,
            purposes,
            capabilities,
            claim_level,
            offline,
            hardware_access,
            authoritative_store_access: store_access,
            resources,
        })
    }
    /// Rejects privilege or scientific-claim expansion against core authority.
    pub fn validate_authority(&self, core: &CoreAuthority) -> Result<(), SidecarError> {
        if !self.offline || self.hardware_access || self.authoritative_store_access {
            return Err(SidecarError::PrivilegeExpansion);
        }
        if !self.purposes.is_subset(&core.purposes)
            || !self.capabilities.is_subset(&core.capabilities)
            || self.claim_level > core.claim_level
        {
            return Err(SidecarError::AuthorityExpansion);
        }
        Ok(())
    }
}

/// Signed sidecar manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SignedSidecarManifest {
    /// Immutable manifest.
    pub manifest: SidecarManifest,
    /// Detached Ed25519 signature.
    pub signature: Vec<u8>,
}
impl SignedSidecarManifest {
    /// Signs during a controlled offline ceremony.
    pub fn sign(manifest: SidecarManifest, key: &SigningKey) -> Result<Self, SidecarError> {
        let signature = key
            .sign(&serde_json::to_vec(&manifest).map_err(|_| SidecarError::Serialization)?)
            .to_bytes()
            .to_vec();
        Ok(Self {
            manifest,
            signature,
        })
    }
    /// Verifies signature and authority ceiling.
    pub fn verify(
        &self,
        key: &VerifyingKey,
        core: &CoreAuthority,
    ) -> Result<VerifiedSidecarManifest<'_>, SidecarError> {
        let signature =
            Signature::from_slice(&self.signature).map_err(|_| SidecarError::InvalidSignature)?;
        key.verify(
            &serde_json::to_vec(&self.manifest).map_err(|_| SidecarError::Serialization)?,
            &signature,
        )
        .map_err(|_| SidecarError::InvalidSignature)?;
        self.manifest.validate_authority(core)?;
        Ok(VerifiedSidecarManifest(self))
    }
}

/// Proof that signature and authority-ceiling validation succeeded.
#[derive(Clone, Copy, Debug)]
pub struct VerifiedSidecarManifest<'a>(&'a SignedSidecarManifest);

/// Supervised optional process lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SidecarState {
    /// No sidecar installed; core remains fully available.
    Absent,
    /// Manifest verified and process starting.
    Starting,
    /// Health checks passing within resource/rate limits.
    Healthy,
    /// Health or protocol failure observed.
    Unhealthy,
    /// Disabled after bounded failure; core remains available.
    Disabled,
}
/// Health/resource sample from the sandbox supervisor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealthSample {
    /// Process responded in time.
    pub responsive: bool,
    /// Current RSS bytes.
    pub memory_bytes: u64,
    /// Requests in current fixed minute.
    pub requests_this_minute: u32,
}
/// Failure-isolated optional-sidecar supervisor.
#[derive(Clone, Debug)]
pub struct SidecarSupervisor {
    state: SidecarState,
    limits: Option<ResourceLimits>,
    consecutive_failures: u8,
    events: Vec<SidecarEvent>,
}
impl Default for SidecarSupervisor {
    fn default() -> Self {
        Self {
            state: SidecarState::Absent,
            limits: None,
            consecutive_failures: 0,
            events: Vec::new(),
        }
    }
}
impl SidecarSupervisor {
    /// Installs only a previously verified manifest.
    pub fn install_verified(
        &mut self,
        signed: &VerifiedSidecarManifest<'_>,
    ) -> Result<(), SidecarError> {
        if self.state != SidecarState::Absent && self.state != SidecarState::Disabled {
            return Err(SidecarError::InvalidState);
        }
        self.limits = Some(signed.0.manifest.resources);
        self.state = SidecarState::Starting;
        self.consecutive_failures = 0;
        self.events.push(SidecarEvent::Starting);
        Ok(())
    }
    /// Applies one bounded health sample and disables after three consecutive failures.
    pub fn observe(&mut self, sample: HealthSample) -> Result<(), SidecarError> {
        if !matches!(
            self.state,
            SidecarState::Starting | SidecarState::Healthy | SidecarState::Unhealthy
        ) {
            return Err(SidecarError::InvalidState);
        }
        let limits = self.limits.ok_or(SidecarError::InvalidState)?;
        let good = sample.responsive
            && sample.memory_bytes <= limits.memory_bytes
            && sample.requests_this_minute <= limits.requests_per_minute;
        if good {
            self.consecutive_failures = 0;
            self.state = SidecarState::Healthy;
            self.events.push(SidecarEvent::Healthy);
        } else {
            self.consecutive_failures = self.consecutive_failures.saturating_add(1);
            self.state = if self.consecutive_failures >= 3 {
                SidecarState::Disabled
            } else {
                SidecarState::Unhealthy
            };
            self.events.push(SidecarEvent::FailureObserved {
                count: self.consecutive_failures,
            });
        }
        Ok(())
    }
    /// Core availability is invariant across every optional-sidecar state.
    #[must_use]
    pub const fn core_available(&self) -> bool {
        true
    }
    /// Current sidecar state.
    #[must_use]
    pub const fn state(&self) -> SidecarState {
        self.state
    }
    /// Immutable supervisor evidence.
    #[must_use]
    pub fn events(&self) -> &[SidecarEvent] {
        &self.events
    }
}
/// Supervisor events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SidecarEvent {
    /// Verified process starting.
    Starting,
    /// Health sample passed.
    Healthy,
    /// Failure observed.
    FailureObserved {
        /// Consecutive failure count.
        count: u8,
    },
}
/// Stable policy/supervision failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SidecarError {
    /// Required value invalid.
    InvalidValue(&'static str),
    /// Resource limit unsafe.
    InvalidResources,
    /// IPC contract incomplete.
    InvalidContract,
    /// Sidecar requested privileged access.
    PrivilegeExpansion,
    /// Sidecar widened purpose/capability/claim.
    AuthorityExpansion,
    /// Signature invalid.
    InvalidSignature,
    /// Serialization failed.
    Serialization,
    /// Lifecycle transition invalid.
    InvalidState,
}
impl fmt::Display for SidecarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sidecar rejected: {self:?}")
    }
}
impl Error for SidecarError {}
