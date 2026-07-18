//! Participant-scoped append-only personalization aggregate.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

/// Compatibility marker for bounded-context tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainBoundary;
fn required(value: impl Into<String>, label: &'static str) -> Result<String, DomainError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 256 {
        Err(DomainError::InvalidValue(label))
    } else {
        Ok(value)
    }
}

/// Opaque participant scope required by every vector-memory operation.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ParticipantPseudonym(String);
impl ParticipantPseudonym {
    /// Creates a bounded pseudonym.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(required(value, "participant pseudonym")?))
    }
    /// Opaque stable value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Explicit user-provided anchor label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchorLabel(String);
impl AnchorLabel {
    /// Creates a bounded label.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(required(value, "anchor label")?))
    }
}

/// Finite, non-empty embedding owned by one participant.
#[derive(Clone, Debug, PartialEq)]
pub struct EmbeddingVector(Vec<f32>);
impl EmbeddingVector {
    /// Creates a checked vector.
    pub fn new(values: Vec<f32>) -> Result<Self, DomainError> {
        if values.is_empty() || values.len() > 4096 || values.iter().any(|v| !v.is_finite()) {
            Err(DomainError::InvalidEmbedding)
        } else {
            Ok(Self(values))
        }
    }
    /// Read-only components.
    #[must_use]
    pub fn values(&self) -> &[f32] {
        &self.0
    }
}

/// Dataset role controlling whether memory mutation is legal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PartitionRole {
    /// Model fitting data.
    Training,
    /// Iterative development data.
    Development,
    /// Prospective held-out evaluation data.
    Evaluation,
    /// Locked confirmatory data.
    Test,
}
impl PartitionRole {
    fn is_mutable(self) -> bool {
        matches!(self, Self::Training | Self::Development)
    }
}

/// Exact assessment, observation, and session source lineage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    assessment_id: String,
    observation_id: String,
    session_id: String,
}
impl Provenance {
    /// Creates complete provenance.
    pub fn new(
        assessment: impl Into<String>,
        observation: impl Into<String>,
        session: impl Into<String>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            assessment_id: required(assessment, "assessment identifier")?,
            observation_id: required(observation, "observation identifier")?,
            session_id: required(session, "session identifier")?,
        })
    }
}

/// Append-only participant anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct Anchor {
    id: String,
    participant: ParticipantPseudonym,
    label: AnchorLabel,
    embedding: EmbeddingVector,
    provenance: Provenance,
    partition: PartitionRole,
}
/// Append-only feedback; corrections link rather than overwrite.
#[derive(Clone, Debug, PartialEq)]
pub struct FeedbackRecord {
    id: String,
    anchor_id: String,
    reward: f32,
    corrects: Option<String>,
    partition: PartitionRole,
}
impl FeedbackRecord {
    /// Returns the immutable finite reward.
    #[must_use]
    pub const fn reward(&self) -> f32 {
        self.reward
    }
}

/// Versioned candidate adaptation with exact feedback lineage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptationVersion {
    id: String,
    parent: Option<String>,
    feedback_ids: Vec<String>,
    improved: bool,
}
/// Held-out evidence recorded prospectively and never used to update memory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProspectiveEvaluation {
    id: String,
    adaptation_id: String,
}

/// Pattern-profile aggregate and its retained audit events.
#[derive(Clone, Debug, PartialEq)]
pub struct PatternProfile {
    id: String,
    participant: ParticipantPseudonym,
    anchors: Vec<Anchor>,
    feedback: Vec<FeedbackRecord>,
    adaptations: Vec<AdaptationVersion>,
    evaluations: Vec<ProspectiveEvaluation>,
    events: Vec<PatternProfileEvent>,
    forgotten: bool,
}
impl PatternProfile {
    /// Creates an empty participant-scoped profile.
    pub fn create(
        id: impl Into<String>,
        participant: ParticipantPseudonym,
    ) -> Result<Self, DomainError> {
        let id = required(id, "profile identifier")?;
        Ok(Self {
            events: vec![PatternProfileEvent::Created {
                profile_id: id.clone(),
                participant: participant.clone(),
            }],
            id,
            participant,
            anchors: Vec::new(),
            feedback: Vec::new(),
            adaptations: Vec::new(),
            evaluations: Vec::new(),
            forgotten: false,
        })
    }
    fn ensure_active(&self) -> Result<(), DomainError> {
        if self.forgotten {
            Err(DomainError::ProfileForgotten)
        } else {
            Ok(())
        }
    }
    /// Appends an anchor only from a mutable partition.
    pub fn record_anchor(
        &mut self,
        id: impl Into<String>,
        label: AnchorLabel,
        embedding: EmbeddingVector,
        provenance: Provenance,
        partition: PartitionRole,
    ) -> Result<String, DomainError> {
        self.ensure_active()?;
        if !partition.is_mutable() {
            return Err(DomainError::ReadOnlyPartition);
        }
        let id = required(id, "anchor identifier")?;
        if self.anchors.iter().any(|a| a.id == id) {
            return Err(DomainError::DuplicateIdentity);
        }
        self.anchors.push(Anchor {
            id: id.clone(),
            participant: self.participant.clone(),
            label,
            embedding,
            provenance,
            partition,
        });
        self.events.push(PatternProfileEvent::AnchorRecorded {
            anchor_id: id.clone(),
        });
        Ok(id)
    }
    /// Appends feedback or a linked correction without rewriting prior records.
    pub fn record_feedback(
        &mut self,
        id: impl Into<String>,
        anchor: impl AsRef<str>,
        reward: f32,
        corrects: Option<&str>,
        partition: PartitionRole,
    ) -> Result<(), DomainError> {
        self.ensure_active()?;
        if !partition.is_mutable() {
            return Err(DomainError::ReadOnlyPartition);
        }
        if !reward.is_finite() || !(-1.0..=1.0).contains(&reward) {
            return Err(DomainError::InvalidReward);
        }
        let anchor = anchor.as_ref();
        if !self.anchors.iter().any(|a| a.id == anchor) {
            return Err(DomainError::UnknownLineage);
        }
        if corrects.is_some_and(|prior| !self.feedback.iter().any(|f| f.id == prior)) {
            return Err(DomainError::UnknownLineage);
        }
        let id = required(id, "feedback identifier")?;
        if self.feedback.iter().any(|f| f.id == id) {
            return Err(DomainError::DuplicateIdentity);
        }
        self.feedback.push(FeedbackRecord {
            id: id.clone(),
            anchor_id: anchor.into(),
            reward,
            corrects: corrects.map(str::to_owned),
            partition,
        });
        self.events
            .push(PatternProfileEvent::FeedbackRecorded { feedback_id: id });
        Ok(())
    }
    /// Builds an append-only candidate with exact parent and feedback set.
    pub fn build_adaptation(
        &mut self,
        id: impl Into<String>,
        parent: Option<&str>,
        feedback_ids: Vec<String>,
        improved: bool,
    ) -> Result<(), DomainError> {
        self.ensure_active()?;
        let id = required(id, "adaptation identifier")?;
        if feedback_ids.is_empty()
            || feedback_ids.iter().collect::<BTreeSet<_>>().len() != feedback_ids.len()
            || feedback_ids
                .iter()
                .any(|id| !self.feedback.iter().any(|f| &f.id == id))
        {
            return Err(DomainError::UnknownLineage);
        }
        if let Some(parent) = parent {
            if !self.adaptations.iter().any(|v| v.id == parent) {
                return Err(DomainError::UnknownLineage);
            }
            if improved && !self.evaluations.iter().any(|e| e.adaptation_id == parent) {
                return Err(DomainError::ProspectiveEvidenceRequired);
            }
        } else if !self.adaptations.is_empty() {
            return Err(DomainError::UnknownLineage);
        }
        self.adaptations.push(AdaptationVersion {
            id: id.clone(),
            parent: parent.map(str::to_owned),
            feedback_ids,
            improved,
        });
        self.events
            .push(PatternProfileEvent::AdaptationBuilt { adaptation_id: id });
        Ok(())
    }
    /// Records prospective held-out evidence without updating payloads.
    pub fn record_prospective_evaluation(
        &mut self,
        id: impl Into<String>,
        adaptation: impl Into<String>,
    ) -> Result<(), DomainError> {
        self.ensure_active()?;
        let adaptation = adaptation.into();
        if !self.adaptations.iter().any(|v| v.id == adaptation) {
            return Err(DomainError::UnknownLineage);
        }
        self.evaluations.push(ProspectiveEvaluation {
            id: required(id, "evaluation identifier")?,
            adaptation_id: adaptation,
        });
        Ok(())
    }
    /// Tombstones the profile and removes all retrievable payloads while retaining audit events.
    pub fn forget(&mut self, erasure_proof: impl Into<String>) -> Result<(), DomainError> {
        self.ensure_active()?;
        let proof = required(erasure_proof, "erasure proof")?;
        self.anchors.clear();
        self.feedback.clear();
        self.adaptations.clear();
        self.evaluations.clear();
        self.forgotten = true;
        self.events.push(PatternProfileEvent::Forgotten {
            erasure_proof: proof,
        });
        Ok(())
    }
    /// Stable profile identity.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Participant scope.
    #[must_use]
    pub const fn participant(&self) -> &ParticipantPseudonym {
        &self.participant
    }
    /// Anchors.
    #[must_use]
    pub fn anchors(&self) -> &[Anchor] {
        &self.anchors
    }
    /// Feedback history.
    #[must_use]
    pub fn feedback(&self) -> &[FeedbackRecord] {
        &self.feedback
    }
    /// Adaptation history.
    #[must_use]
    pub fn adaptations(&self) -> &[AdaptationVersion] {
        &self.adaptations
    }
    /// Tombstone state.
    #[must_use]
    pub const fn is_forgotten(&self) -> bool {
        self.forgotten
    }
}

/// Append-only aggregate events with payload-minimized deletion evidence.
#[derive(Clone, Debug, PartialEq)]
pub enum PatternProfileEvent {
    /// Profile created.
    Created {
        /// Profile identity.
        profile_id: String,
        /// Participant scope.
        participant: ParticipantPseudonym,
    },
    /// Anchor appended.
    AnchorRecorded {
        /// Anchor identity.
        anchor_id: String,
    },
    /// Feedback appended.
    FeedbackRecorded {
        /// Feedback identity.
        feedback_id: String,
    },
    /// Candidate adaptation appended.
    AdaptationBuilt {
        /// Adaptation identity.
        adaptation_id: String,
    },
    /// Payloads cryptographically erased.
    Forgotten {
        /// Payload-minimized proof.
        erasure_proof: String,
    },
}

/// Stable invariant failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    /// Invalid bounded value.
    InvalidValue(&'static str),
    /// Embedding invalid.
    InvalidEmbedding,
    /// Reward invalid.
    InvalidReward,
    /// Evaluation/test partition attempted mutation.
    ReadOnlyPartition,
    /// Duplicate append identity.
    DuplicateIdentity,
    /// Parent/anchor/feedback lineage absent.
    UnknownLineage,
    /// Improvement lacks prospective evidence.
    ProspectiveEvidenceRequired,
    /// Participant data was forgotten.
    ProfileForgotten,
}
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "personalization invariant failed: {self:?}")
    }
}
impl Error for DomainError {}
