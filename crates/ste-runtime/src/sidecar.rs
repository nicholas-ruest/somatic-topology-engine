//! Optional, unprivileged sidecar trust boundary and failure containment.

use sha2::{Digest, Sha256};

/// Explicit advisory operations; no hardware, store, key, policy, or capability command exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SidecarOperation {
    /// Summarize an already-deidentified support payload.
    SummarizeDeidentified,
    /// Produce offline copy suggestions subject to later Rust review.
    OptimizeOfflineCopy,
}

/// Sidecar request with pinned contract and bounded non-authoritative payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidecarRequest {
    /// Unique request identity.
    pub request_id: String,
    /// Allowlisted operation.
    pub operation: SidecarOperation,
    /// Digest of generated contract bindings.
    pub contract_digest: [u8; 32],
    /// Deidentified/advisory payload only.
    pub payload: Vec<u8>,
}

/// Sidecar response; always advisory and never directly persisted or projected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidecarResponse {
    /// Correlated request identity.
    pub request_id: String,
    /// Pinned contract digest echoed by the process.
    pub contract_digest: [u8; 32],
    /// Advisory output requiring Rust-side review.
    pub advisory_payload: Vec<u8>,
}

/// OS-level containment requirements passed to the process launcher.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxPolicy {
    /// Dedicated non-root UID.
    pub uid: u32,
    /// CPU quota in milliseconds per second.
    pub cpu_millis_per_second: u16,
    /// Address-space limit.
    pub memory_bytes: u64,
    /// Writable scratch quota.
    pub storage_bytes: u64,
    /// Network is denied by default and required to remain false here.
    pub network_allowed: bool,
    /// Hardware device nodes are denied.
    pub hardware_allowed: bool,
    /// Authoritative storage paths are denied.
    pub authoritative_store_allowed: bool,
}
impl SandboxPolicy {
    /// Validates least privilege and bounded resources.
    pub fn validate(self) -> Result<Self, SidecarError> {
        if self.uid == 0
            || self.cpu_millis_per_second == 0
            || self.cpu_millis_per_second > 1_000
            || self.memory_bytes == 0
            || self.storage_bytes == 0
            || self.network_allowed
            || self.hardware_allowed
            || self.authoritative_store_allowed
        {
            Err(SidecarError::UnsafeSandbox)
        } else {
            Ok(self)
        }
    }
}

/// Optional process adapter, implemented by an OS-specific unprivileged launcher.
pub trait SidecarProcess {
    /// Starts under the exact validated sandbox.
    fn start(&mut self, sandbox: SandboxPolicy) -> Result<(), SidecarError>;
    /// Exchanges one authenticated, bounded local request before the deadline.
    fn exchange(
        &mut self,
        credential_digest: [u8; 32],
        request: &SidecarRequest,
        deadline_ns: u64,
    ) -> Result<SidecarResponse, SidecarError>;
    /// Terminates the optional process without stopping the Rust core.
    fn terminate(&mut self) -> Result<(), SidecarError>;
}

/// Runtime state for the optional feature only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SidecarHealth {
    /// Feature is not enabled.
    Disabled,
    /// Enabled but not started.
    Starting,
    /// Process passed launch and request checks.
    Healthy,
    /// Optional process is absent or failed.
    DegradedAbsent,
    /// Optional process exceeded deadline.
    DegradedHung,
    /// Optional process violated its response contract.
    DegradedCorrupt,
    /// Process was independently terminated.
    Killed,
}

/// Fixed policy for authentication, contracts, rate, payload, and sandbox.
#[derive(Clone, Debug)]
pub struct SidecarPolicy {
    /// Sidecar is opt-in and false by default.
    pub enabled: bool,
    /// Exact generated contract digest.
    pub contract_digest: [u8; 32],
    /// Maximum requests per supervisor window.
    pub maximum_requests_per_window: usize,
    /// Maximum request/response payload bytes.
    pub maximum_payload_bytes: usize,
    /// Process deadline delta.
    pub timeout_ns: u64,
    /// Validated sandbox constraints.
    pub sandbox: SandboxPolicy,
}

/// Failure-contained supervisor. It exposes no authoritative store or hardware handle.
pub struct SidecarSupervisor<P> {
    process: P,
    policy: SidecarPolicy,
    credential_digest: [u8; 32],
    used_in_window: usize,
    health: SidecarHealth,
}
impl<P: SidecarProcess> SidecarSupervisor<P> {
    /// Creates a disabled/starting supervisor without retaining the plaintext secret.
    pub fn new(process: P, policy: SidecarPolicy, credential: &str) -> Result<Self, SidecarError> {
        policy.sandbox.validate()?;
        if policy.maximum_requests_per_window == 0
            || policy.maximum_payload_bytes == 0
            || policy.timeout_ns == 0
            || credential.len() < 16
        {
            return Err(SidecarError::InvalidPolicy);
        }
        let enabled = policy.enabled;
        Ok(Self {
            process,
            policy,
            credential_digest: Sha256::digest(credential.as_bytes()).into(),
            used_in_window: 0,
            health: if enabled {
                SidecarHealth::Starting
            } else {
                SidecarHealth::Disabled
            },
        })
    }
    /// Starts only when explicitly enabled; absence degrades only the optional feature.
    pub fn start(&mut self) {
        if !self.policy.enabled {
            return;
        }
        self.health = if self.process.start(self.policy.sandbox).is_ok() {
            SidecarHealth::Healthy
        } else {
            SidecarHealth::DegradedAbsent
        };
    }
    /// Executes one allowlisted advisory operation with fail-closed validation.
    pub fn execute(
        &mut self,
        request: &SidecarRequest,
        now_ns: u64,
    ) -> Result<SidecarResponse, SidecarError> {
        if self.health != SidecarHealth::Healthy {
            return Err(SidecarError::Unavailable);
        }
        if request.request_id.trim().is_empty()
            || request.contract_digest != self.policy.contract_digest
            || request.payload.len() > self.policy.maximum_payload_bytes
        {
            return Err(SidecarError::InvalidContract);
        }
        if self.used_in_window >= self.policy.maximum_requests_per_window {
            return Err(SidecarError::RateLimited);
        }
        self.used_in_window += 1;
        let deadline = now_ns
            .checked_add(self.policy.timeout_ns)
            .ok_or(SidecarError::InvalidPolicy)?;
        match self
            .process
            .exchange(self.credential_digest, request, deadline)
        {
            Ok(response)
                if response.request_id == request.request_id
                    && response.contract_digest == self.policy.contract_digest
                    && response.advisory_payload.len() <= self.policy.maximum_payload_bytes =>
            {
                Ok(response)
            }
            Ok(_) => {
                self.health = SidecarHealth::DegradedCorrupt;
                let _ = self.process.terminate();
                Err(SidecarError::CorruptResponse)
            }
            Err(SidecarError::TimedOut) => {
                self.health = SidecarHealth::DegradedHung;
                let _ = self.process.terminate();
                Err(SidecarError::TimedOut)
            }
            Err(error) => {
                self.health = SidecarHealth::DegradedAbsent;
                Err(error)
            }
        }
    }
    /// Terminates the optional process independently.
    pub fn kill(&mut self) {
        let _ = self.process.terminate();
        self.health = SidecarHealth::Killed;
    }
    /// Optional-feature health; core health is intentionally not mutable here.
    #[must_use]
    pub const fn health(&self) -> SidecarHealth {
        self.health
    }
}

/// Sidecar containment failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SidecarError {
    /// Sandbox grants privilege or has unsafe limits.
    UnsafeSandbox,
    /// Policy or credential is malformed.
    InvalidPolicy,
    /// Optional process is absent or disabled.
    Unavailable,
    /// Contract digest, identity, or payload violates the schema.
    InvalidContract,
    /// Fixed-window request budget was exhausted.
    RateLimited,
    /// Process exceeded its deadline.
    TimedOut,
    /// Response failed identity, digest, or size validation.
    CorruptResponse,
    /// Process launch/exchange failed.
    ProcessFailed,
}
