//! Deterministic watchdog and signed synthetic verification evidence.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Critical task watchdog with explicit monotonic deadlines.
#[derive(Clone, Debug)]
pub struct Watchdog {
    timeout_ns: u64,
    heartbeats: BTreeMap<String, u64>,
}
impl Watchdog {
    /// Creates a bounded watchdog.
    pub fn new(timeout_ns: u64) -> Result<Self, VerificationError> {
        if timeout_ns == 0 {
            Err(VerificationError::InvalidWatchdog)
        } else {
            Ok(Self {
                timeout_ns,
                heartbeats: BTreeMap::new(),
            })
        }
    }
    /// Registers or refreshes a critical-task heartbeat at monotonic time.
    pub fn heartbeat(&mut self, task: &str, now_ns: u64) -> Result<(), VerificationError> {
        if task.trim().is_empty()
            || self
                .heartbeats
                .get(task)
                .is_some_and(|previous| now_ns < *previous)
        {
            return Err(VerificationError::InvalidHeartbeat);
        }
        self.heartbeats.insert(task.to_owned(), now_ns);
        Ok(())
    }
    /// Returns expired task names in stable order.
    #[must_use]
    pub fn expired(&self, now_ns: u64) -> Vec<String> {
        self.heartbeats
            .iter()
            .filter(|(_, last)| now_ns.saturating_sub(**last) > self.timeout_ns)
            .map(|(task, _)| task.clone())
            .collect()
    }
}

/// Honest evidence environment label.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceEnvironment {
    /// Deterministic development-host simulation only.
    SyntheticDevelopmentHost,
}

/// Canonical signed verification report; it makes no HIL or external-review claim.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationEvidence {
    /// Stable schema.
    pub schema: String,
    /// Exact test suite identity.
    pub suite_id: String,
    /// Source revision under test.
    pub source_revision: String,
    /// Explicit evidence environment.
    pub environment: EvidenceEnvironment,
    /// Stable passed control identifiers.
    pub passed_controls: Vec<String>,
    /// Always false for this harness.
    pub external_penetration_test: bool,
    /// Always false for this harness.
    pub physical_hil: bool,
    /// Signing key identity.
    pub key_id: String,
    /// Ed25519 signature over canonical unsigned fields.
    pub signature: Vec<u8>,
}
impl VerificationEvidence {
    /// Creates signed synthetic-only evidence with sorted unique controls.
    pub fn sign(
        suite_id: &str,
        source_revision: &str,
        controls: Vec<String>,
        key_id: &str,
        key: &SigningKey,
    ) -> Result<Self, VerificationError> {
        if suite_id.trim().is_empty()
            || source_revision.trim().is_empty()
            || key_id.trim().is_empty()
            || controls.is_empty()
        {
            return Err(VerificationError::InvalidEvidence);
        }
        let mut evidence = Self {
            schema: "ste-verification-evidence-v1".into(),
            suite_id: suite_id.into(),
            source_revision: source_revision.into(),
            environment: EvidenceEnvironment::SyntheticDevelopmentHost,
            passed_controls: controls,
            external_penetration_test: false,
            physical_hil: false,
            key_id: key_id.into(),
            signature: Vec::new(),
        };
        evidence.passed_controls.sort();
        evidence.passed_controls.dedup();
        evidence.signature = key.sign(&evidence.unsigned_bytes()?).to_bytes().to_vec();
        Ok(evidence)
    }
    /// Verifies schema, honest labels, canonical controls, and signature.
    pub fn verify(&self, key: &VerifyingKey) -> Result<(), VerificationError> {
        if self.schema != "ste-verification-evidence-v1"
            || self.external_penetration_test
            || self.physical_hil
            || self.passed_controls.is_empty()
            || !self
                .passed_controls
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        {
            return Err(VerificationError::InvalidEvidence);
        }
        let signature = Signature::from_slice(&self.signature)
            .map_err(|_| VerificationError::InvalidSignature)?;
        key.verify(&self.unsigned_bytes()?, &signature)
            .map_err(|_| VerificationError::InvalidSignature)
    }
    fn unsigned_bytes(&self) -> Result<Vec<u8>, VerificationError> {
        #[derive(Serialize)]
        struct Unsigned<'a> {
            schema: &'a str,
            suite_id: &'a str,
            source_revision: &'a str,
            environment: EvidenceEnvironment,
            passed_controls: &'a [String],
            external_penetration_test: bool,
            physical_hil: bool,
            key_id: &'a str,
        }
        serde_json::to_vec(&Unsigned {
            schema: &self.schema,
            suite_id: &self.suite_id,
            source_revision: &self.source_revision,
            environment: self.environment,
            passed_controls: &self.passed_controls,
            external_penetration_test: self.external_penetration_test,
            physical_hil: self.physical_hil,
            key_id: &self.key_id,
        })
        .map_err(|_| VerificationError::InvalidEvidence)
    }
}

/// Verification construction or integrity failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationError {
    /// Watchdog timeout is invalid.
    InvalidWatchdog,
    /// Heartbeat identity/time is invalid.
    InvalidHeartbeat,
    /// Evidence fields or honesty labels are invalid.
    InvalidEvidence,
    /// Evidence signature is invalid.
    InvalidSignature,
}
