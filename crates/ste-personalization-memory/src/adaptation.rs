//! Sandboxed candidate adaptation, prospective comparison, promotion, and rollback.

use crate::retrieval::PartitionRole;
use std::collections::BTreeSet;

/// Explicit participant correction admitted to candidate training.
#[derive(Clone, Debug, PartialEq)]
pub struct QualifiedFeedback {
    /// Feedback identity.
    pub id: String,
    /// Participant owner.
    pub participant_id: String,
    /// Training anchor identity.
    pub anchor_id: String,
    /// Immutable partition role.
    pub partition: PartitionRole,
    /// Quality score in `[0, 1]`.
    pub quality: f32,
}

/// Exact, sorted training lineage retained with a sandbox candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptationLineage {
    /// Participant owner.
    pub participant_id: String,
    /// Unique sorted feedback identities.
    pub feedback_ids: Vec<String>,
    /// Unique sorted anchor identities.
    pub anchor_ids: Vec<String>,
}

/// Candidate lifecycle, intentionally separate from the active profile.
#[derive(Clone, Debug, PartialEq)]
pub enum CandidateState {
    /// Candidate can only run in a sandbox.
    Sandboxed,
    /// Prospective held-out evidence supports the stated improvement.
    ProspectivelyValidated {
        /// Baseline metric.
        baseline_score: f32,
        /// Candidate metric.
        candidate_score: f32,
        /// Exact evaluation observations.
        evaluation_ids: Vec<String>,
    },
    /// Candidate is active for the participant.
    Promoted,
    /// Candidate was removed from service.
    RolledBack {
        /// Auditable operator/system reason.
        reason: String,
    },
}

/// Sandboxed adaptation candidate.
#[derive(Clone, Debug, PartialEq)]
pub struct AdaptationCandidate {
    /// Stable candidate identity.
    pub id: String,
    /// Exact training lineage.
    pub lineage: AdaptationLineage,
    /// Lifecycle state.
    pub state: CandidateState,
}

/// Bounded candidate admission policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdaptationPolicy {
    /// Minimum unique qualified feedback items.
    pub minimum_evidence: usize,
    /// Minimum accepted quality.
    pub minimum_quality: f32,
    /// Maximum candidates per participant in one window.
    pub maximum_candidates_per_window: usize,
    /// Minimum prospective improvement.
    pub minimum_improvement: f32,
}

/// Deterministic per-participant fixed-window admission counter.
#[derive(Clone, Debug)]
pub struct CandidateFactory {
    policy: AdaptationPolicy,
    window_id: u64,
    admitted: std::collections::BTreeMap<String, usize>,
}

impl CandidateFactory {
    /// Creates a factory from a valid policy and explicit window identity.
    pub fn new(policy: AdaptationPolicy, window_id: u64) -> Result<Self, AdaptationError> {
        if policy.minimum_evidence == 0
            || policy.maximum_candidates_per_window == 0
            || !valid_unit(policy.minimum_quality)
            || !valid_unit(policy.minimum_improvement)
        {
            return Err(AdaptationError::InvalidPolicy);
        }
        Ok(Self {
            policy,
            window_id,
            admitted: std::collections::BTreeMap::new(),
        })
    }

    /// Opens a sandbox candidate using only qualified training feedback.
    pub fn create(
        &mut self,
        candidate_id: &str,
        participant_id: &str,
        window_id: u64,
        feedback: &[QualifiedFeedback],
    ) -> Result<AdaptationCandidate, AdaptationError> {
        if candidate_id.trim().is_empty()
            || participant_id.trim().is_empty()
            || window_id != self.window_id
        {
            return Err(AdaptationError::InvalidRequest);
        }
        if self.admitted.get(participant_id).copied().unwrap_or(0)
            >= self.policy.maximum_candidates_per_window
        {
            return Err(AdaptationError::RateLimited);
        }
        if feedback
            .iter()
            .any(|item| item.participant_id != participant_id)
        {
            return Err(AdaptationError::CrossParticipantEvidence);
        }
        if feedback
            .iter()
            .any(|item| item.partition != PartitionRole::Training)
        {
            return Err(AdaptationError::PartitionLeakage);
        }
        if feedback
            .iter()
            .any(|item| !valid_unit(item.quality) || item.quality < self.policy.minimum_quality)
        {
            return Err(AdaptationError::UnqualifiedFeedback);
        }
        let feedback_ids: BTreeSet<_> = feedback.iter().map(|item| item.id.clone()).collect();
        let anchor_ids: BTreeSet<_> = feedback.iter().map(|item| item.anchor_id.clone()).collect();
        if feedback_ids.len() != feedback.len() || anchor_ids.len() != feedback.len() {
            return Err(AdaptationError::DuplicateEvidence);
        }
        if feedback.len() < self.policy.minimum_evidence {
            return Err(AdaptationError::InsufficientEvidence);
        }
        *self.admitted.entry(participant_id.to_owned()).or_default() += 1;
        Ok(AdaptationCandidate {
            id: candidate_id.to_owned(),
            lineage: AdaptationLineage {
                participant_id: participant_id.to_owned(),
                feedback_ids: feedback_ids.into_iter().collect(),
                anchor_ids: anchor_ids.into_iter().collect(),
            },
            state: CandidateState::Sandboxed,
        })
    }

    /// Records a prospective held-out comparison; training IDs cannot be reused.
    pub fn compare_prospectively(
        &self,
        candidate: &mut AdaptationCandidate,
        baseline_score: f32,
        candidate_score: f32,
        evaluation_ids: Vec<String>,
    ) -> Result<(), AdaptationError> {
        if candidate.state != CandidateState::Sandboxed
            || !baseline_score.is_finite()
            || !candidate_score.is_finite()
            || evaluation_ids.is_empty()
        {
            return Err(AdaptationError::InvalidComparison);
        }
        let unique: BTreeSet<_> = evaluation_ids.iter().cloned().collect();
        if unique.len() != evaluation_ids.len()
            || unique
                .iter()
                .any(|id| candidate.lineage.feedback_ids.binary_search(id).is_ok())
        {
            return Err(AdaptationError::PartitionLeakage);
        }
        if candidate_score - baseline_score < self.policy.minimum_improvement {
            return Err(AdaptationError::NoProspectiveImprovement);
        }
        candidate.state = CandidateState::ProspectivelyValidated {
            baseline_score,
            candidate_score,
            evaluation_ids: unique.into_iter().collect(),
        };
        Ok(())
    }
}

impl AdaptationCandidate {
    /// Promotes only a prospectively validated candidate.
    pub fn promote(&mut self) -> Result<(), AdaptationError> {
        if !matches!(self.state, CandidateState::ProspectivelyValidated { .. }) {
            return Err(AdaptationError::NotProspectivelyValidated);
        }
        self.state = CandidateState::Promoted;
        Ok(())
    }

    /// Rolls back a promoted candidate with an auditable reason.
    pub fn rollback(&mut self, reason: &str) -> Result<(), AdaptationError> {
        if self.state != CandidateState::Promoted || reason.trim().is_empty() {
            return Err(AdaptationError::InvalidRollback);
        }
        self.state = CandidateState::RolledBack {
            reason: reason.to_owned(),
        };
        Ok(())
    }
}

fn valid_unit(value: f32) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

/// Fail-closed candidate workflow error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdaptationError {
    /// Policy values are unsafe or malformed.
    InvalidPolicy,
    /// Candidate identity, participant, or window is invalid.
    InvalidRequest,
    /// Per-participant candidate budget was exhausted.
    RateLimited,
    /// Evidence belongs to another participant.
    CrossParticipantEvidence,
    /// Read-only held-out data was offered for training or reused.
    PartitionLeakage,
    /// Feedback failed explicit quality qualification.
    UnqualifiedFeedback,
    /// Evidence identities were duplicated.
    DuplicateEvidence,
    /// Too few unique feedback items were supplied.
    InsufficientEvidence,
    /// Comparison metrics or lifecycle state are invalid.
    InvalidComparison,
    /// Prospective evidence did not exceed the configured margin.
    NoProspectiveImprovement,
    /// Promotion was attempted before prospective validation.
    NotProspectivelyValidated,
    /// Rollback state or reason is invalid.
    InvalidRollback,
}
