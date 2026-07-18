//! Deterministic participant-scoped similarity retrieval.

use std::cmp::Ordering;

/// Immutable role assigned before an anchor is observed by adaptation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PartitionRole {
    /// May be retrieved and used to train a candidate.
    Training,
    /// Read-only prospective comparison data.
    ProspectiveEvaluation,
    /// Read-only final evaluation data.
    Test,
}

/// Participant-owned vector with exact source provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct Anchor {
    /// Stable anchor identity.
    pub id: String,
    /// Participant scope; never inferred from vector content.
    pub participant_id: String,
    /// Source artifact identity.
    pub source_id: String,
    /// Immutable partition role.
    pub partition: PartitionRole,
    /// Finite embedding.
    pub embedding: Vec<f32>,
    /// False after correction/deletion; history remains append-only elsewhere.
    pub active: bool,
}

/// Retrieval match without cross-participant payload.
#[derive(Clone, Debug, PartialEq)]
pub struct SimilarityMatch {
    /// Anchor identity.
    pub anchor_id: String,
    /// Source artifact identity for provenance display.
    pub source_id: String,
    /// Cosine similarity.
    pub similarity: f32,
}

/// In-memory deterministic reference implementation of the vector port.
#[derive(Clone, Debug, Default)]
pub struct ParticipantVectorMemory {
    anchors: Vec<Anchor>,
}

impl ParticipantVectorMemory {
    /// Builds a validated memory snapshot.
    pub fn new(anchors: Vec<Anchor>) -> Result<Self, RetrievalError> {
        if anchors.iter().any(|anchor| {
            anchor.id.trim().is_empty()
                || anchor.participant_id.trim().is_empty()
                || anchor.source_id.trim().is_empty()
                || !valid_vector(&anchor.embedding)
        }) {
            return Err(RetrievalError::InvalidAnchor);
        }
        let mut ids: Vec<&str> = anchors.iter().map(|anchor| anchor.id.as_str()).collect();
        ids.sort_unstable();
        if ids.windows(2).any(|ids| ids[0] == ids[1]) {
            return Err(RetrievalError::DuplicateAnchor);
        }
        Ok(Self { anchors })
    }

    /// Retrieves training anchors for exactly one participant with stable ordering.
    pub fn retrieve(
        &self,
        participant_id: &str,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<SimilarityMatch>, RetrievalError> {
        if participant_id.trim().is_empty() || !valid_vector(query) || limit == 0 {
            return Err(RetrievalError::InvalidQuery);
        }
        let mut matches = Vec::new();
        for anchor in self.anchors.iter().filter(|anchor| {
            anchor.active
                && anchor.partition == PartitionRole::Training
                && anchor.participant_id == participant_id
        }) {
            if anchor.embedding.len() != query.len() {
                return Err(RetrievalError::DimensionMismatch);
            }
            matches.push(SimilarityMatch {
                anchor_id: anchor.id.clone(),
                source_id: anchor.source_id.clone(),
                similarity: cosine(query, &anchor.embedding),
            });
        }
        matches.sort_by(|left, right| {
            right
                .similarity
                .partial_cmp(&left.similarity)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.anchor_id.cmp(&right.anchor_id))
        });
        matches.truncate(limit);
        Ok(matches)
    }
}

fn valid_vector(vector: &[f32]) -> bool {
    !vector.is_empty() && vector.iter().all(|value| value.is_finite())
}

fn cosine(left: &[f32], right: &[f32]) -> f32 {
    let dot: f32 = left.iter().zip(right).map(|(a, b)| a * b).sum();
    let left_norm: f32 = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm: f32 = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

/// Retrieval validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrievalError {
    /// Anchor metadata or vector is malformed.
    InvalidAnchor,
    /// Anchor identity was reused.
    DuplicateAnchor,
    /// Query or limit is malformed.
    InvalidQuery,
    /// A participant anchor used an incompatible embedding schema.
    DimensionMismatch,
}
