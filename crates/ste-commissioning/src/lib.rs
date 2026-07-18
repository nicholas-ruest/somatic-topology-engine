#![forbid(unsafe_code)]
//! Guided offline commissioning, site qualification, and signed acceptance records.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

fn required(value: impl Into<String>, label: &'static str) -> Result<String, CommissioningError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 256 {
        Err(CommissioningError::InvalidValue(label))
    } else {
        Ok(value)
    }
}

/// Every mandatory commissioning check; omission can never imply success.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CheckKind {
    /// Hardware revision and identity.
    Hardware,
    /// Signed supported firmware.
    Firmware,
    /// Required peripherals and visible sensing indicator.
    Peripherals,
    /// Supply voltage/current stability.
    Power,
    /// Idle/load thermal margin.
    Thermal,
    /// Supported AP/link profile.
    AccessPointLink,
    /// Minimum valid packet rate and loss budget.
    PacketRate,
    /// Measured room geometry and placement.
    Geometry,
    /// Consent and purpose authorization.
    Consent,
    /// Capacity, encryption, and health.
    Storage,
    /// Clock correlation and synchronization quality.
    Clocks,
    /// Current calibration profile.
    Calibration,
    /// Interference within operating envelope.
    Interference,
}
impl CheckKind {
    /// Complete required check set.
    pub const ALL: [Self; 13] = [
        Self::Hardware,
        Self::Firmware,
        Self::Peripherals,
        Self::Power,
        Self::Thermal,
        Self::AccessPointLink,
        Self::PacketRate,
        Self::Geometry,
        Self::Consent,
        Self::Storage,
        Self::Clocks,
        Self::Calibration,
        Self::Interference,
    ];
}

/// Evidence-bearing outcome for one fixed threshold check.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CheckOutcome {
    /// Check passed its versioned threshold.
    Passed {
        /// Immutable evidence digest/reference.
        evidence: String,
    },
    /// Check failed and cannot be overridden.
    Failed {
        /// Stable failure reason.
        reason: String,
    },
}
impl CheckOutcome {
    /// Creates a passed result.
    pub fn passed(evidence: impl Into<String>) -> Result<Self, CommissioningError> {
        Ok(Self::Passed {
            evidence: required(evidence, "check evidence")?,
        })
    }
    /// Creates a failed result.
    pub fn failed(reason: impl Into<String>) -> Result<Self, CommissioningError> {
        Ok(Self::Failed {
            reason: required(reason, "failure reason")?,
        })
    }
    fn passed_check(&self) -> bool {
        matches!(self, Self::Passed { .. })
    }
}

/// Exact measured coverage of one capability at this site.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityCoverage {
    /// Versioned capability identifier.
    pub capability: String,
    /// Whether its frozen coverage threshold passed.
    pub threshold_passed: bool,
    /// Evidence digest/reference.
    pub evidence: String,
}
impl CapabilityCoverage {
    /// Creates coverage without allowing threshold modification.
    pub fn new(
        capability: impl Into<String>,
        threshold_passed: bool,
        evidence: impl Into<String>,
    ) -> Result<Self, CommissioningError> {
        Ok(Self {
            capability: required(capability, "capability")?,
            threshold_passed,
            evidence: required(evidence, "coverage evidence")?,
        })
    }
}

/// Commissioning lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommissioningState {
    /// Checks may be recorded.
    InProgress,
    /// Signed acceptance issued.
    Qualified,
    /// A mandatory check failed.
    Rejected,
    /// Operator entered isolated recovery mode.
    Recovery,
}

/// Guided commissioning aggregate.
#[derive(Clone, Debug)]
pub struct CommissioningSession {
    id: String,
    site_id: String,
    profile: String,
    previous_record: Option<String>,
    checks: BTreeMap<CheckKind, CheckOutcome>,
    requested_capabilities: BTreeSet<String>,
    coverage: BTreeMap<String, CapabilityCoverage>,
    state: CommissioningState,
    events: Vec<CommissioningEvent>,
}
impl CommissioningSession {
    /// Starts initial site commissioning.
    pub fn start(
        id: impl Into<String>,
        site: impl Into<String>,
        profile: impl Into<String>,
        requested: impl IntoIterator<Item = String>,
    ) -> Result<Self, CommissioningError> {
        Self::start_inner(id, site, profile, requested, None)
    }
    /// Starts requalification linked to the previous immutable acceptance record.
    pub fn requalify(
        id: impl Into<String>,
        site: impl Into<String>,
        profile: impl Into<String>,
        requested: impl IntoIterator<Item = String>,
        previous_record: impl Into<String>,
    ) -> Result<Self, CommissioningError> {
        Self::start_inner(
            id,
            site,
            profile,
            requested,
            Some(required(previous_record, "previous acceptance record")?),
        )
    }
    fn start_inner(
        id: impl Into<String>,
        site: impl Into<String>,
        profile: impl Into<String>,
        requested: impl IntoIterator<Item = String>,
        previous_record: Option<String>,
    ) -> Result<Self, CommissioningError> {
        let id = required(id, "commissioning identifier")?;
        let requested_capabilities = requested
            .into_iter()
            .map(|v| required(v, "capability"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        Ok(Self {
            events: vec![CommissioningEvent::Started {
                session_id: id.clone(),
            }],
            id,
            site_id: required(site, "site identifier")?,
            profile: required(profile, "deployment profile")?,
            previous_record,
            checks: BTreeMap::new(),
            requested_capabilities,
            coverage: BTreeMap::new(),
            state: CommissioningState::InProgress,
        })
    }
    /// Records one check exactly once; failures terminally reject this attempt.
    pub fn record_check(
        &mut self,
        kind: CheckKind,
        outcome: CheckOutcome,
    ) -> Result<(), CommissioningError> {
        self.ensure_progress()?;
        if self.checks.contains_key(&kind) {
            return Err(CommissioningError::DuplicateCheck);
        }
        let passed = outcome.passed_check();
        self.checks.insert(kind, outcome);
        self.events
            .push(CommissioningEvent::CheckRecorded { kind, passed });
        if !passed {
            self.state = CommissioningState::Rejected;
        }
        Ok(())
    }
    /// Records immutable per-capability coverage; failed coverage simply blocks that capability.
    pub fn record_coverage(
        &mut self,
        coverage: CapabilityCoverage,
    ) -> Result<(), CommissioningError> {
        self.ensure_progress()?;
        if !self.requested_capabilities.contains(&coverage.capability) {
            return Err(CommissioningError::UnknownCapability);
        }
        if self
            .coverage
            .insert(coverage.capability.clone(), coverage)
            .is_some()
        {
            return Err(CommissioningError::DuplicateCoverage);
        }
        Ok(())
    }
    /// Enters isolated recovery without manufacturing an acceptance record.
    pub fn enter_recovery(&mut self, reason: impl Into<String>) -> Result<(), CommissioningError> {
        if self.state == CommissioningState::Qualified {
            return Err(CommissioningError::TerminalState);
        }
        let reason = required(reason, "recovery reason")?;
        self.state = CommissioningState::Recovery;
        self.events
            .push(CommissioningEvent::RecoveryEntered { reason });
        Ok(())
    }
    /// Issues a signed record only after every mandatory check has passed and coverage is explicit.
    pub fn qualify(
        &mut self,
        issued_at: u64,
        key_id: impl Into<String>,
        key: &SigningKey,
    ) -> Result<SignedAcceptanceRecord, CommissioningError> {
        self.ensure_progress()?;
        if CheckKind::ALL
            .iter()
            .any(|k| !self.checks.get(k).is_some_and(CheckOutcome::passed_check))
        {
            return Err(CommissioningError::MandatoryCheckMissingOrFailed);
        }
        if self
            .requested_capabilities
            .iter()
            .any(|c| !self.coverage.contains_key(c))
        {
            return Err(CommissioningError::CapabilityCoverageMissing);
        }
        let enabled_capabilities = self
            .coverage
            .values()
            .filter(|c| c.threshold_passed)
            .map(|c| c.capability.clone())
            .collect();
        let blocked_capabilities = self
            .coverage
            .values()
            .filter(|c| !c.threshold_passed)
            .map(|c| c.capability.clone())
            .collect();
        let record = AcceptanceRecord {
            schema_version: 1,
            record_id: format!("acceptance:{}:{issued_at}", self.id),
            session_id: self.id.clone(),
            site_id: self.site_id.clone(),
            deployment_profile: self.profile.clone(),
            issued_at,
            previous_record: self.previous_record.clone(),
            checks: self.checks.clone(),
            coverage: self.coverage.clone(),
            enabled_capabilities,
            blocked_capabilities,
            signing_key_id: required(key_id, "signing key identifier")?,
        };
        let signature = key
            .sign(&serde_json::to_vec(&record).map_err(|_| CommissioningError::Serialization)?)
            .to_bytes()
            .to_vec();
        self.state = CommissioningState::Qualified;
        self.events.push(CommissioningEvent::Qualified {
            record_id: record.record_id.clone(),
        });
        Ok(SignedAcceptanceRecord { record, signature })
    }
    fn ensure_progress(&self) -> Result<(), CommissioningError> {
        if self.state == CommissioningState::InProgress {
            Ok(())
        } else {
            Err(CommissioningError::TerminalState)
        }
    }
    /// Current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> &CommissioningState {
        &self.state
    }
    /// Immutable events.
    #[must_use]
    pub fn events(&self) -> &[CommissioningEvent] {
        &self.events
    }
}

/// Signed, portable site acceptance or requalification record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AcceptanceRecord {
    /// Schema version.
    pub schema_version: u16,
    /// Stable record identity.
    pub record_id: String,
    /// Commissioning attempt.
    pub session_id: String,
    /// Qualified site.
    pub site_id: String,
    /// Signed deployment profile.
    pub deployment_profile: String,
    /// Deterministic issue timestamp.
    pub issued_at: u64,
    /// Previous record for requalification lineage.
    pub previous_record: Option<String>,
    /// Complete check evidence.
    pub checks: BTreeMap<CheckKind, CheckOutcome>,
    /// Complete requested capability coverage.
    pub coverage: BTreeMap<String, CapabilityCoverage>,
    /// Capabilities permitted by this site record.
    pub enabled_capabilities: BTreeSet<String>,
    /// Capabilities explicitly blocked without weakened thresholds.
    pub blocked_capabilities: BTreeSet<String>,
    /// Signing-key identity.
    pub signing_key_id: String,
}
/// Record plus detached Ed25519 signature.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SignedAcceptanceRecord {
    /// Immutable record.
    pub record: AcceptanceRecord,
    /// Detached signature bytes.
    pub signature: Vec<u8>,
}
impl SignedAcceptanceRecord {
    /// Verifies signature and record structure.
    pub fn verify(&self, key: &VerifyingKey) -> Result<(), CommissioningError> {
        let signature = Signature::from_slice(&self.signature)
            .map_err(|_| CommissioningError::InvalidSignature)?;
        key.verify(
            &serde_json::to_vec(&self.record).map_err(|_| CommissioningError::Serialization)?,
            &signature,
        )
        .map_err(|_| CommissioningError::InvalidSignature)
    }
    /// Fail-closed capability enablement query.
    #[must_use]
    pub fn enables(&self, capability: &str) -> bool {
        self.record.enabled_capabilities.contains(capability)
            && !self.record.blocked_capabilities.contains(capability)
    }
}

/// Append-only commissioning events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommissioningEvent {
    /// Session started.
    Started {
        /// Session identity.
        session_id: String,
    },
    /// Check recorded.
    CheckRecorded {
        /// Check kind.
        kind: CheckKind,
        /// Pass status.
        passed: bool,
    },
    /// Recovery entered.
    RecoveryEntered {
        /// Recovery reason.
        reason: String,
    },
    /// Qualification issued.
    Qualified {
        /// Acceptance record identity.
        record_id: String,
    },
}
/// Persistence boundary for attempts and immutable signed records.
pub trait CommissioningRepository {
    /// Store failure.
    type Error;
    /// Saves current attempt.
    fn save_session(&mut self, session: &CommissioningSession) -> Result<(), Self::Error>;
    /// Appends signed acceptance.
    fn append_acceptance(&mut self, record: &SignedAcceptanceRecord) -> Result<(), Self::Error>;
}
/// Stable commissioning failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommissioningError {
    /// Required value invalid.
    InvalidValue(&'static str),
    /// Check was repeated.
    DuplicateCheck,
    /// Coverage was repeated.
    DuplicateCoverage,
    /// Coverage did not match requested capability.
    UnknownCapability,
    /// Session is no longer mutable.
    TerminalState,
    /// Required check absent or failed.
    MandatoryCheckMissingOrFailed,
    /// Requested coverage absent.
    CapabilityCoverageMissing,
    /// Record serialization failed.
    Serialization,
    /// Signature invalid.
    InvalidSignature,
}
impl fmt::Display for CommissioningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "commissioning rejected: {self:?}")
    }
}
impl Error for CommissioningError {}
