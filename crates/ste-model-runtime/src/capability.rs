//! Signed, exact-match capability policy and experimental-output isolation.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Purpose types permitted at the capability-policy boundary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CapabilityPurpose {
    /// Approved research protocol.
    Research,
    /// Approved non-medical product capability.
    Wellness,
    /// Site/device calibration without claim output.
    Calibration,
}

/// Scientific promotion level bound into the signed policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PromotionLevel {
    /// Experimental evaluation only.
    Experimental,
    /// Production claim level passed independent promotion.
    Production,
}

/// Whether a capability may emit production or isolated experimental output.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CapabilityMode {
    /// Eligible for production projections after every other gate passes.
    Production,
    /// Must remain visibly marked and stored in an isolated namespace.
    Experimental,
}

/// Canonical signed policy payload. Every binding uses exact equality.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityPolicy {
    /// Unique immutable policy identifier.
    pub policy_id: String,
    /// Exact capability identifier.
    pub capability_id: String,
    /// Output mode.
    pub mode: CapabilityMode,
    /// Exact software release digest.
    pub software_digest: String,
    /// Exact verified model-package digest.
    pub model_digest: String,
    /// Exact hardware/deployment profile digest.
    pub hardware_profile_digest: String,
    /// Exact operating-envelope digest.
    pub operating_envelope_digest: String,
    /// Exact immutable promotion-decision digest.
    pub promotion_digest: String,
    /// Required promotion level.
    pub promotion_level: PromotionLevel,
    /// Authorized participant purpose.
    pub purpose: CapabilityPurpose,
    /// Authorized jurisdiction.
    pub jurisdiction: String,
    /// Authorized deployment/device profile identifier.
    pub deployment_id: String,
    /// Inclusive policy validity start.
    pub not_before_unix_seconds: u64,
    /// Exclusive policy expiration.
    pub expires_at_unix_seconds: u64,
    /// Rollout kill switch; false always denies.
    pub enabled: bool,
}

/// Signed policy envelope containing no private key material.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SignedCapabilityPolicy {
    /// Canonical payload.
    pub policy: CapabilityPolicy,
    /// Trusted signer identifier.
    pub signer_key_id: String,
    /// Ed25519 signature bytes.
    pub signature: Vec<u8>,
}

impl SignedCapabilityPolicy {
    /// Signs canonical JSON bytes with the provided release-policy key.
    pub fn sign(
        policy: CapabilityPolicy,
        signer_key_id: impl Into<String>,
        signing_key: &SigningKey,
    ) -> Result<Self, CapabilityPolicyError> {
        validate_policy(&policy)?;
        let signer_key_id = signer_key_id.into();
        if signer_key_id.trim().is_empty() {
            return Err(CapabilityPolicyError::InvalidPolicy);
        }
        let bytes = canonical_bytes(&policy)?;
        Ok(Self {
            policy,
            signer_key_id,
            signature: signing_key.sign(&bytes).to_bytes().to_vec(),
        })
    }
}

/// Runtime values supplied locally to the policy decision point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityContext {
    /// Requested capability.
    pub capability_id: String,
    /// Running software release digest.
    pub software_digest: String,
    /// Atomically active verified model digest.
    pub model_digest: String,
    /// Active registry state confirms the model is neither suspended nor revoked.
    pub model_active_and_not_revoked: bool,
    /// Detected hardware/deployment profile digest.
    pub hardware_profile_digest: String,
    /// Current qualified operating-envelope digest.
    pub operating_envelope_digest: String,
    /// Latest immutable promotion-decision digest.
    pub promotion_digest: String,
    /// Latest promotion level.
    pub promotion_level: PromotionLevel,
    /// Whether the promotion decision remains active (not suspended/revoked).
    pub promotion_active: bool,
    /// Current participant-authorized purpose.
    pub purpose: CapabilityPurpose,
    /// Current jurisdiction.
    pub jurisdiction: String,
    /// Current deployment/device profile identifier.
    pub deployment_id: String,
    /// Explicit local evaluation time.
    pub evaluated_at_unix_seconds: u64,
    /// Whether current evidence passed OOD/scope/quality checks.
    pub evidence_in_scope: bool,
}

/// Safe production grant or isolated experimental grant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapabilityGrant {
    /// Production output may proceed to a separate projection policy.
    Production {
        /// Exact enabled capability.
        capability_id: String,
    },
    /// Output may exist only in the explicit experimental isolation boundary.
    Experimental(ExperimentalIsolation),
}

/// Mandatory isolation metadata for experimental output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentalIsolation {
    /// Separate persistence/telemetry namespace.
    pub namespace: String,
    /// Mandatory visible label.
    pub visible_label: &'static str,
    /// Permanently false by type construction.
    pub production_projection_allowed: bool,
}

/// Stable fail-closed denial reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityDenial {
    /// Signer is unknown, signature malformed, or signature verification failed.
    SignatureInvalid,
    /// Policy fields are malformed or internally inconsistent.
    InvalidPolicy,
    /// Rollout was explicitly disabled.
    Disabled,
    /// Policy is not yet valid or has expired.
    OutsideValidity,
    /// Capability identifier differs.
    CapabilityMismatch,
    /// Software binding differs.
    SoftwareMismatch,
    /// Model binding differs.
    ModelMismatch,
    /// Bound model is inactive, suspended, quarantined, or revoked.
    ModelUnavailable,
    /// Hardware binding differs.
    HardwareMismatch,
    /// Operating-envelope binding differs.
    EnvelopeMismatch,
    /// Promotion is absent, inactive, revoked, or differs.
    PromotionMismatch,
    /// Participant purpose differs.
    PurposeMismatch,
    /// Jurisdiction differs.
    JurisdictionMismatch,
    /// Deployment identity differs.
    DeploymentMismatch,
    /// Current evidence is OOD or out of scope.
    EvidenceOutOfScope,
    /// Production mode is bound to a non-production promotion.
    ExperimentalPromotionCannotServe,
}

/// Local policy evaluator with an explicit trusted-key set.
#[derive(Clone, Default)]
pub struct CapabilityPolicyEvaluator {
    trusted_keys: BTreeMap<String, VerifyingKey>,
}

impl CapabilityPolicyEvaluator {
    /// Creates a verifier from explicit key IDs; duplicate IDs fail.
    pub fn new(
        trusted_keys: impl IntoIterator<Item = (String, VerifyingKey)>,
    ) -> Result<Self, CapabilityPolicyError> {
        let mut keys = BTreeMap::new();
        for (id, key) in trusted_keys {
            if id.trim().is_empty() || keys.insert(id, key).is_some() {
                return Err(CapabilityPolicyError::InvalidTrustedKeys);
            }
        }
        if keys.is_empty() {
            return Err(CapabilityPolicyError::InvalidTrustedKeys);
        }
        Ok(Self { trusted_keys: keys })
    }

    /// Verifies signature before evaluating every exact runtime binding.
    pub fn evaluate(
        &self,
        signed: &SignedCapabilityPolicy,
        context: &CapabilityContext,
    ) -> Result<CapabilityGrant, CapabilityDenial> {
        let Some(key) = self.trusted_keys.get(&signed.signer_key_id) else {
            return Err(CapabilityDenial::SignatureInvalid);
        };
        let Ok(signature_bytes): Result<[u8; 64], _> = signed.signature.as_slice().try_into()
        else {
            return Err(CapabilityDenial::SignatureInvalid);
        };
        let signature = Signature::from_bytes(&signature_bytes);
        let Ok(bytes) = canonical_bytes(&signed.policy) else {
            return Err(CapabilityDenial::InvalidPolicy);
        };
        if key.verify(&bytes, &signature).is_err() {
            return Err(CapabilityDenial::SignatureInvalid);
        }
        if validate_policy(&signed.policy).is_err() {
            return Err(CapabilityDenial::InvalidPolicy);
        }
        let policy = &signed.policy;
        if !policy.enabled {
            return Err(CapabilityDenial::Disabled);
        }
        if context.evaluated_at_unix_seconds < policy.not_before_unix_seconds
            || context.evaluated_at_unix_seconds >= policy.expires_at_unix_seconds
        {
            return Err(CapabilityDenial::OutsideValidity);
        }
        let exact = [
            (
                policy.capability_id == context.capability_id,
                CapabilityDenial::CapabilityMismatch,
            ),
            (
                policy.software_digest == context.software_digest,
                CapabilityDenial::SoftwareMismatch,
            ),
            (
                policy.model_digest == context.model_digest,
                CapabilityDenial::ModelMismatch,
            ),
            (
                policy.hardware_profile_digest == context.hardware_profile_digest,
                CapabilityDenial::HardwareMismatch,
            ),
            (
                policy.operating_envelope_digest == context.operating_envelope_digest,
                CapabilityDenial::EnvelopeMismatch,
            ),
            (
                policy.promotion_digest == context.promotion_digest
                    && policy.promotion_level == context.promotion_level
                    && context.promotion_active,
                CapabilityDenial::PromotionMismatch,
            ),
            (
                policy.purpose == context.purpose,
                CapabilityDenial::PurposeMismatch,
            ),
            (
                policy.jurisdiction == context.jurisdiction,
                CapabilityDenial::JurisdictionMismatch,
            ),
            (
                policy.deployment_id == context.deployment_id,
                CapabilityDenial::DeploymentMismatch,
            ),
        ];
        if let Some((_, denial)) = exact.into_iter().find(|(matches, _)| !matches) {
            return Err(denial);
        }
        if !context.evidence_in_scope {
            return Err(CapabilityDenial::EvidenceOutOfScope);
        }
        if !context.model_active_and_not_revoked {
            return Err(CapabilityDenial::ModelUnavailable);
        }
        match policy.mode {
            CapabilityMode::Production => {
                if policy.promotion_level != PromotionLevel::Production {
                    return Err(CapabilityDenial::ExperimentalPromotionCannotServe);
                }
                Ok(CapabilityGrant::Production {
                    capability_id: policy.capability_id.clone(),
                })
            }
            CapabilityMode::Experimental => {
                Ok(CapabilityGrant::Experimental(ExperimentalIsolation {
                    namespace: format!(
                        "experimental/{}/{}",
                        policy.capability_id, policy.policy_id
                    ),
                    visible_label: "EXPERIMENTAL — NOT FOR PRODUCTION USE",
                    production_projection_allowed: false,
                }))
            }
        }
    }
}

fn validate_policy(policy: &CapabilityPolicy) -> Result<(), CapabilityPolicyError> {
    let required = [
        &policy.policy_id,
        &policy.capability_id,
        &policy.software_digest,
        &policy.model_digest,
        &policy.hardware_profile_digest,
        &policy.operating_envelope_digest,
        &policy.promotion_digest,
        &policy.jurisdiction,
        &policy.deployment_id,
    ];
    if required
        .iter()
        .any(|value| value.trim().is_empty() || value.len() > 256)
        || policy.expires_at_unix_seconds <= policy.not_before_unix_seconds
    {
        Err(CapabilityPolicyError::InvalidPolicy)
    } else {
        Ok(())
    }
}

fn canonical_bytes(policy: &CapabilityPolicy) -> Result<Vec<u8>, CapabilityPolicyError> {
    serde_json::to_vec(policy).map_err(|_| CapabilityPolicyError::InvalidPolicy)
}

/// Policy construction/trusted-key configuration failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityPolicyError {
    /// Policy payload is malformed.
    InvalidPolicy,
    /// Trusted signer set is empty, duplicated, or malformed.
    InvalidTrustedKeys,
}
