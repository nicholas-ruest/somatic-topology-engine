//! Application gate composing validation hooks, uncertainty, and capability policy.

use crate::domain::{
    AssessLatentState, AssessmentOutcome, CalibratedProbability, ConstructDefinition,
    EvidenceBundle, GateEvidence, ModelScope, StateAssessment,
};
use ste_model_runtime::capability::{
    CapabilityContext, CapabilityDenial, CapabilityGrant, CapabilityPolicyEvaluator,
    ExperimentalIsolation, SignedCapabilityPolicy,
};
use ste_model_runtime::uncertainty::{
    CalibrationArtifact, InferenceEvidence, OodArtifact, UncertaintyAbstention,
    UncertaintyDecision, evaluate_uncertainty,
};

/// Read-only validation hooks; unavailable dependencies fail closed.
pub trait InferenceValidationHooks {
    /// Hook failure type, never converted to an authorization.
    type Error;

    /// Exact construct promotion decision.
    fn construct_is_promoted(&self, construct_reference: &str) -> Result<bool, Self::Error>;
    /// Physiology evidence exists and did not abstain.
    fn physiology_is_valid(&self, physiology_id: &str) -> Result<bool, Self::Error>;
    /// Participant/profile lineage remains valid.
    fn profile_is_valid(
        &self,
        participant: &str,
        profile_version: &str,
    ) -> Result<bool, Self::Error>;
    /// Current evidence lies inside the supported operating envelope.
    fn operating_envelope_holds(&self, scope: &ModelScope) -> Result<bool, Self::Error>;
}

/// Claim rollout policy separate from scientific promotion and signed capability policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ClaimPolicy {
    /// Whether an independently promoted task-specific workload construct may run.
    pub task_specific_workload_enabled: bool,
}

/// Complete application request with explicit raw score and all immutable artifacts.
pub struct ValidatedInferenceRequest<'a> {
    /// Assessment identity.
    pub assessment_id: String,
    /// Domain construct definition.
    pub construct: ConstructDefinition,
    /// Full observation and physiology evidence lineage.
    pub evidence: EvidenceBundle,
    /// Exact model operating scope.
    pub scope: ModelScope,
    /// Active model-package digest.
    pub model_digest: &'a str,
    /// Raw model score, never returned to display consumers.
    pub raw_score: f64,
    /// Frozen calibration artifact.
    pub calibration: &'a CalibrationArtifact,
    /// Frozen OOD/scope artifact.
    pub ood: &'a OodArtifact,
    /// Runtime OOD evidence.
    pub inference_evidence: &'a InferenceEvidence,
    /// Signed capability policy.
    pub capability_policy: &'a SignedCapabilityPolicy,
    /// Exact runtime capability context.
    pub capability_context: &'a CapabilityContext,
}

/// No-op integration outcome that carries provenance but emits no construct claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselineNoopTrace {
    /// Assessment identity.
    pub assessment_id: String,
    /// Full distinct observation/physiology evidence bundle.
    pub evidence: EvidenceBundle,
    /// Exact scope supplied to the no-op path.
    pub scope: ModelScope,
    /// Always false; baseline cannot become a projection.
    pub claim_emitted: bool,
}

/// Isolated experimental result, structurally separate from production assessment.
#[derive(Clone, Debug, PartialEq)]
pub struct ExperimentalInference {
    /// Assessment identity.
    pub assessment_id: String,
    /// Complete source lineage.
    pub evidence: EvidenceBundle,
    /// Exact scope.
    pub scope: ModelScope,
    /// Calibrated probability; never a raw score.
    pub probability: CalibratedProbability,
    /// Mandatory experimental namespace/label/projection prohibition.
    pub isolation: ExperimentalIsolation,
}

/// Fail-closed application reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationAbstention {
    /// A read-only validation hook was unavailable.
    HookUnavailable,
    /// Construct lacks exact immutable promotion.
    ConstructNotPromoted,
    /// Physiology evidence is absent or abstained.
    InvalidPhysiology,
    /// Participant/profile lineage is invalid.
    InvalidProfile,
    /// Operating envelope is violated.
    OutsideOperatingEnvelope,
    /// Claim rollout remains disabled.
    ClaimPolicyDisabled,
    /// Calibration/OOD/confidence gate abstained.
    Uncertainty(UncertaintyAbstention),
    /// Signed capability policy denied output.
    Capability(CapabilityDenial),
    /// Domain construction rejected inconsistent values.
    DomainInvariant,
}

/// Production, isolated experimental, baseline no-op, or typed abstention.
#[derive(Clone, Debug, PartialEq)]
pub enum ValidatedInferenceOutcome {
    /// Domain assessment; raw model score is absent by construction.
    Production(AssessmentOutcome),
    /// Experimental output cannot enter production projection/storage types.
    Experimental(ExperimentalInference),
    /// Baseline integration path made no claim.
    Baseline(BaselineNoopTrace),
    /// Application gate denied output.
    Abstained {
        /// Assessment identity.
        assessment_id: String,
        /// Stable denial reason.
        reason: ValidationAbstention,
    },
}

/// Complete application orchestrator.
pub struct ValidatedInferenceService<'a, H: InferenceValidationHooks> {
    hooks: &'a H,
    capability: &'a CapabilityPolicyEvaluator,
    claims: ClaimPolicy,
}

impl<'a, H: InferenceValidationHooks> ValidatedInferenceService<'a, H> {
    /// Composes read-only hooks, signed-policy evaluator, and local claim rollout.
    #[must_use]
    pub const fn new(
        hooks: &'a H,
        capability: &'a CapabilityPolicyEvaluator,
        claims: ClaimPolicy,
    ) -> Self {
        Self {
            hooks,
            capability,
            claims,
        }
    }

    /// Evaluates every gate in deterministic fail-closed order.
    #[must_use]
    pub fn assess(&self, request: ValidatedInferenceRequest<'_>) -> ValidatedInferenceOutcome {
        if request.construct.is_baseline() {
            return ValidatedInferenceOutcome::Baseline(BaselineNoopTrace {
                assessment_id: request.assessment_id,
                evidence: request.evidence,
                scope: request.scope,
                claim_emitted: false,
            });
        }
        let construct_reference = request.construct.reference();
        let promoted = match self.hooks.construct_is_promoted(&construct_reference) {
            Ok(value) => value,
            Err(_) => {
                return abstain(
                    &request.assessment_id,
                    ValidationAbstention::HookUnavailable,
                );
            }
        };
        if !promoted {
            return abstain(
                &request.assessment_id,
                ValidationAbstention::ConstructNotPromoted,
            );
        }
        let physiology_valid = match self
            .hooks
            .physiology_is_valid(&request.evidence.physiology_id)
        {
            Ok(value) => value,
            Err(_) => {
                return abstain(
                    &request.assessment_id,
                    ValidationAbstention::HookUnavailable,
                );
            }
        };
        if !physiology_valid {
            return abstain(
                &request.assessment_id,
                ValidationAbstention::InvalidPhysiology,
            );
        }
        let profile_valid = match self.hooks.profile_is_valid(
            &request.scope.participant,
            &request.evidence.provenance.profile_version,
        ) {
            Ok(value) => value,
            Err(_) => {
                return abstain(
                    &request.assessment_id,
                    ValidationAbstention::HookUnavailable,
                );
            }
        };
        if !profile_valid {
            return abstain(&request.assessment_id, ValidationAbstention::InvalidProfile);
        }
        let inside_envelope = match self.hooks.operating_envelope_holds(&request.scope) {
            Ok(value) => value,
            Err(_) => {
                return abstain(
                    &request.assessment_id,
                    ValidationAbstention::HookUnavailable,
                );
            }
        };
        if !inside_envelope {
            return abstain(
                &request.assessment_id,
                ValidationAbstention::OutsideOperatingEnvelope,
            );
        }
        if !self.claims.task_specific_workload_enabled {
            return abstain(
                &request.assessment_id,
                ValidationAbstention::ClaimPolicyDisabled,
            );
        }

        let probability = match evaluate_uncertainty(
            request.model_digest,
            request.raw_score,
            request.calibration,
            request.ood,
            request.inference_evidence,
        ) {
            UncertaintyDecision::Serve(probability) => probability,
            UncertaintyDecision::Abstain(reason) => {
                return abstain(
                    &request.assessment_id,
                    ValidationAbstention::Uncertainty(reason),
                );
            }
        };
        let capability = match self
            .capability
            .evaluate(request.capability_policy, request.capability_context)
        {
            Ok(grant) => grant,
            Err(reason) => {
                return abstain(
                    &request.assessment_id,
                    ValidationAbstention::Capability(reason),
                );
            }
        };
        let domain_probability = match CalibratedProbability::new(probability.get()) {
            Ok(probability) => probability,
            Err(_) => {
                return abstain(
                    &request.assessment_id,
                    ValidationAbstention::DomainInvariant,
                );
            }
        };
        if let CapabilityGrant::Experimental(isolation) = capability {
            return ValidatedInferenceOutcome::Experimental(ExperimentalInference {
                assessment_id: request.assessment_id,
                evidence: request.evidence,
                scope: request.scope,
                probability: domain_probability,
                isolation,
            });
        }
        let command = match AssessLatentState::new(
            &request.assessment_id,
            request.construct,
            request.evidence,
            request.scope,
            domain_probability,
            GateEvidence::all_passed(),
        ) {
            Ok(command) => command,
            Err(_) => {
                return abstain(
                    &request.assessment_id,
                    ValidationAbstention::DomainInvariant,
                );
            }
        };
        ValidatedInferenceOutcome::Production(StateAssessment::assess(command))
    }
}

fn abstain(assessment_id: &str, reason: ValidationAbstention) -> ValidatedInferenceOutcome {
    ValidatedInferenceOutcome::Abstained {
        assessment_id: assessment_id.to_owned(),
        reason,
    }
}
