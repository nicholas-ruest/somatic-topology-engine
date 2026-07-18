#![forbid(unsafe_code)]
//! Exact-scope, evidence-bearing production-readiness decisions.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

fn required(value: impl Into<String>, label: &'static str) -> Result<String, ReadinessError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 512 {
        Err(ReadinessError::InvalidValue(label))
    } else {
        Ok(value)
    }
}

/// Exact commercial release scope; approval cannot float to later artifacts or markets.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReleaseScope {
    /// Hardware profile digest/revision.
    pub hardware: String,
    /// Software release digest/version.
    pub software: String,
    /// Exact active model-package digests.
    pub models: BTreeSet<String>,
    /// Approved jurisdictions/markets.
    pub markets: BTreeSet<String>,
    /// Supported operating-envelope digest.
    pub operating_envelope: String,
    /// Support/warranty/runbook revision.
    pub support_revision: String,
}
impl ReleaseScope {
    /// Creates a complete exact scope.
    pub fn new(
        hardware: impl Into<String>,
        software: impl Into<String>,
        models: BTreeSet<String>,
        markets: BTreeSet<String>,
        envelope: impl Into<String>,
        support: impl Into<String>,
    ) -> Result<Self, ReadinessError> {
        if models.is_empty() || markets.is_empty() {
            return Err(ReadinessError::InvalidScope);
        }
        Ok(Self {
            hardware: required(hardware, "hardware binding")?,
            software: required(software, "software binding")?,
            models,
            markets,
            operating_envelope: required(envelope, "operating envelope")?,
            support_revision: required(support, "support revision")?,
        })
    }
}

/// Mandatory production-readiness criteria.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Criterion {
    /// Qualified legal/regulatory market review.
    LegalRegulatory,
    /// Controlled pilot acceptance.
    PilotAcceptance,
    /// Hardware-in-loop and soak evidence.
    HardwareInLoop,
    /// Claim-evidence matrix and marketing review.
    ClaimEvidence,
    /// Safety case and residual-risk review.
    SafetyCase,
    /// Security/privacy assessment and incident readiness.
    SecurityPrivacy,
    /// Reliability/SLO/resource benchmarks.
    ReliabilityPerformance,
    /// Manufacturing/site commissioning readiness.
    ManufacturingCommissioning,
    /// Support, warranty, field operations, and training.
    SupportOperations,
    /// Disaster recovery, backup/restore/reset exercises.
    DisasterRecovery,
    /// Reproducible release/SBOM/provenance evidence.
    ReleaseProvenance,
    /// Documentation and support matrix completeness.
    Documentation,
}
impl Criterion {
    /// Full mandatory gate set.
    pub const ALL: [Self; 12] = [
        Self::LegalRegulatory,
        Self::PilotAcceptance,
        Self::HardwareInLoop,
        Self::ClaimEvidence,
        Self::SafetyCase,
        Self::SecurityPrivacy,
        Self::ReliabilityPerformance,
        Self::ManufacturingCommissioning,
        Self::SupportOperations,
        Self::DisasterRecovery,
        Self::ReleaseProvenance,
        Self::Documentation,
    ];
}

/// Immutable criterion evidence bound to the exact release scope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CriterionEvidence {
    /// Criterion satisfied.
    pub criterion: Criterion,
    /// Content digest of review/evidence bundle.
    pub digest: String,
    /// Whether the frozen gate passed.
    pub passed: bool,
    /// Exclusive evidence expiry.
    pub expires_at: u64,
    /// Exact release-scope fingerprint supplied by release tooling.
    pub scope_binding: String,
}
impl CriterionEvidence {
    /// Creates bounded evidence.
    pub fn new(
        criterion: Criterion,
        digest: impl Into<String>,
        passed: bool,
        expires_at: u64,
        scope_binding: impl Into<String>,
    ) -> Result<Self, ReadinessError> {
        if expires_at == 0 {
            return Err(ReadinessError::InvalidEvidence);
        }
        Ok(Self {
            criterion,
            digest: required(digest, "evidence digest")?,
            passed,
            expires_at,
            scope_binding: required(scope_binding, "scope binding")?,
        })
    }
}

/// Residual risk disposition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResidualRisk {
    /// Risk identity.
    pub id: String,
    /// Severity from 1 (low) to 5 (critical).
    pub severity: u8,
    /// Named acceptance decision.
    pub accepted: bool,
    /// Mitigation/evidence digest.
    pub evidence: String,
}
impl ResidualRisk {
    /// Creates a reviewed risk.
    pub fn new(
        id: impl Into<String>,
        severity: u8,
        accepted: bool,
        evidence: impl Into<String>,
    ) -> Result<Self, ReadinessError> {
        if !(1..=5).contains(&severity) {
            return Err(ReadinessError::InvalidRisk);
        }
        Ok(Self {
            id: required(id, "risk identifier")?,
            severity,
            accepted,
            evidence: required(evidence, "risk evidence")?,
        })
    }
}

/// Time-bound exception; mandatory criteria can never be excepted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReadinessException {
    /// Exception identity.
    pub id: String,
    /// Narrow non-mandatory condition.
    pub condition: String,
    /// Named approver.
    pub approved_by: String,
    /// Exclusive expiry.
    pub expires_at: u64,
}

/// Corrective action generated for each blocking condition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CorrectiveAction {
    /// Stable reason code.
    pub reason: BlockReason,
    /// Required remediation.
    pub action: String,
}
/// Stable blocking/suspension reason.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BlockReason {
    /// Criterion absent.
    MissingCriterion(Criterion),
    /// Criterion failed.
    FailedCriterion(Criterion),
    /// Criterion expired.
    ExpiredCriterion(Criterion),
    /// Evidence belongs to another scope.
    ScopeMismatch(Criterion),
    /// Residual risk not accepted.
    UnacceptedRisk(String),
    /// Critical risks cannot be accepted for release.
    CriticalRisk(String),
    /// Exception is malformed or expired.
    InvalidException(String),
    /// Active field incident requires suspension.
    FieldIncident(String),
}

/// Requested readiness assessment.
pub struct ReadinessReview {
    /// Exact release scope.
    pub scope: ReleaseScope,
    /// Canonical scope fingerprint.
    pub scope_binding: String,
    /// Criterion evidence by kind.
    pub evidence: BTreeMap<Criterion, CriterionEvidence>,
    /// Reviewed residual risks.
    pub residual_risks: Vec<ResidualRisk>,
    /// Narrow time-bound exceptions retained for audit.
    pub exceptions: Vec<ReadinessException>,
}

/// Signed decision status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DecisionStatus {
    /// Exact scope approved until expiry.
    Approved,
    /// Release blocked pending corrective action.
    Blocked,
    /// Previously approved scope suspended due to incident.
    Suspended,
}
/// Immutable production-readiness decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReadinessDecision {
    /// Exact bound scope.
    pub scope: ReleaseScope,
    /// Scope fingerprint.
    pub scope_binding: String,
    /// Decision status.
    pub status: DecisionStatus,
    /// Issue timestamp.
    pub issued_at: u64,
    /// Exclusive decision expiry.
    pub expires_at: u64,
    /// Every blocking corrective action.
    pub corrective_actions: Vec<CorrectiveAction>,
    /// Residual risks retained in decision evidence.
    pub residual_risks: Vec<ResidualRisk>,
    /// Exceptions retained but never used to bypass mandatory gates.
    pub exceptions: Vec<ReadinessException>,
    /// Named signing authority.
    pub approved_by: String,
}
/// Signed readiness decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SignedReadinessDecision {
    /// Immutable decision.
    pub decision: ReadinessDecision,
    /// Detached signature.
    pub signature: Vec<u8>,
}
impl SignedReadinessDecision {
    /// Verifies signature and time-bound exact scope.
    pub fn verify(
        &self,
        key: &VerifyingKey,
        expected_scope: &ReleaseScope,
        expected_binding: &str,
        now: u64,
    ) -> Result<(), ReadinessError> {
        let signature =
            Signature::from_slice(&self.signature).map_err(|_| ReadinessError::InvalidSignature)?;
        key.verify(
            &serde_json::to_vec(&self.decision).map_err(|_| ReadinessError::Serialization)?,
            &signature,
        )
        .map_err(|_| ReadinessError::InvalidSignature)?;
        if &self.decision.scope != expected_scope || self.decision.scope_binding != expected_binding
        {
            return Err(ReadinessError::ScopeMismatch);
        }
        if now >= self.decision.expires_at {
            return Err(ReadinessError::DecisionExpired);
        }
        Ok(())
    }
    /// Production serving is authorized only by a current approval.
    #[must_use]
    pub fn authorizes_release(&self, now: u64) -> bool {
        self.decision.status == DecisionStatus::Approved && now < self.decision.expires_at
    }
}

/// Stateless exact-scope readiness decision engine.
pub struct ReadinessEngine;
impl ReadinessEngine {
    /// Evaluates every mandatory criterion and signs approve/block.
    pub fn decide(
        review: ReadinessReview,
        issued_at: u64,
        expires_at: u64,
        approver: impl Into<String>,
        key: &SigningKey,
    ) -> Result<SignedReadinessDecision, ReadinessError> {
        if issued_at >= expires_at {
            return Err(ReadinessError::InvalidDecisionWindow);
        }
        let mut actions = Vec::new();
        for criterion in Criterion::ALL {
            match review.evidence.get(&criterion) {
                None => actions.push(action(
                    BlockReason::MissingCriterion(criterion),
                    "provide required evidence",
                )),
                Some(e) if e.scope_binding != review.scope_binding => actions.push(action(
                    BlockReason::ScopeMismatch(criterion),
                    "repeat review for exact release scope",
                )),
                Some(e) if issued_at >= e.expires_at => actions.push(action(
                    BlockReason::ExpiredCriterion(criterion),
                    "renew expired evidence",
                )),
                Some(e) if !e.passed => actions.push(action(
                    BlockReason::FailedCriterion(criterion),
                    "complete and pass corrective verification",
                )),
                Some(_) => {}
            }
        }
        for risk in &review.residual_risks {
            if risk.severity == 5 {
                actions.push(action(
                    BlockReason::CriticalRisk(risk.id.clone()),
                    "eliminate or materially reduce critical risk",
                ));
            } else if !risk.accepted {
                actions.push(action(
                    BlockReason::UnacceptedRisk(risk.id.clone()),
                    "mitigate and obtain named risk acceptance",
                ));
            }
        }
        for exception in &review.exceptions {
            if exception.id.trim().is_empty()
                || exception.condition.trim().is_empty()
                || exception.approved_by.trim().is_empty()
                || issued_at >= exception.expires_at
            {
                actions.push(action(
                    BlockReason::InvalidException(exception.id.clone()),
                    "remove, renew, or obtain named approval for the exception",
                ));
            }
        }
        let status = if actions.is_empty() {
            DecisionStatus::Approved
        } else {
            DecisionStatus::Blocked
        };
        Self::sign(
            ReadinessDecision {
                scope: review.scope,
                scope_binding: review.scope_binding,
                status,
                issued_at,
                expires_at,
                corrective_actions: actions,
                residual_risks: review.residual_risks,
                exceptions: review.exceptions,
                approved_by: required(approver, "approver")?,
            },
            key,
        )
    }
    /// Suspends an exact approved scope after a field incident.
    pub fn suspend(
        prior: &SignedReadinessDecision,
        incident: impl Into<String>,
        issued_at: u64,
        expires_at: u64,
        approver: impl Into<String>,
        key: &SigningKey,
    ) -> Result<SignedReadinessDecision, ReadinessError> {
        if issued_at >= expires_at {
            return Err(ReadinessError::InvalidDecisionWindow);
        }
        let incident = required(incident, "incident identifier")?;
        Self::sign(
            ReadinessDecision {
                scope: prior.decision.scope.clone(),
                scope_binding: prior.decision.scope_binding.clone(),
                status: DecisionStatus::Suspended,
                issued_at,
                expires_at,
                corrective_actions: vec![action(
                    BlockReason::FieldIncident(incident),
                    "contain incident, investigate, verify correction, and repeat readiness review",
                )],
                residual_risks: prior.decision.residual_risks.clone(),
                exceptions: prior.decision.exceptions.clone(),
                approved_by: required(approver, "approver")?,
            },
            key,
        )
    }
    fn sign(
        decision: ReadinessDecision,
        key: &SigningKey,
    ) -> Result<SignedReadinessDecision, ReadinessError> {
        let signature = key
            .sign(&serde_json::to_vec(&decision).map_err(|_| ReadinessError::Serialization)?)
            .to_bytes()
            .to_vec();
        Ok(SignedReadinessDecision {
            decision,
            signature,
        })
    }
}
fn action(reason: BlockReason, text: &str) -> CorrectiveAction {
    CorrectiveAction {
        reason,
        action: text.into(),
    }
}

/// Stable readiness failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadinessError {
    /// Required value invalid.
    InvalidValue(&'static str),
    /// Scope incomplete.
    InvalidScope,
    /// Evidence invalid.
    InvalidEvidence,
    /// Risk invalid.
    InvalidRisk,
    /// Decision time invalid.
    InvalidDecisionWindow,
    /// Serialization failed.
    Serialization,
    /// Signature invalid.
    InvalidSignature,
    /// Verified scope differs.
    ScopeMismatch,
    /// Decision expired.
    DecisionExpired,
}
impl fmt::Display for ReadinessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "production readiness rejected: {self:?}")
    }
}
impl Error for ReadinessError {}
