//! Participant-scoped persistence and vector-memory ports.
use crate::domain::{DomainBoundary, EmbeddingVector, ParticipantPseudonym, PatternProfile};
/// Returns the public domain-boundary marker.
#[must_use]
pub const fn boundary() -> DomainBoundary {
    DomainBoundary
}
/// Aggregate metadata repository.
pub trait PatternProfileRepository {
    /// Persistence failure.
    type Error;
    /// Saves an aggregate with optimistic versioning.
    fn save(&mut self, profile: &PatternProfile) -> Result<(), Self::Error>;
}
/// Participant-bound vector key; cross-user keys cannot be constructed without an explicit scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedVectorKey {
    /// Participant scope.
    pub participant: ParticipantPseudonym,
    /// Anchor identity.
    pub anchor_id: String,
}
/// Participant-bound similarity query.
pub struct ScopedVectorQuery<'a> {
    /// Required participant scope.
    pub participant: &'a ParticipantPseudonym,
    /// Query embedding.
    pub embedding: &'a EmbeddingVector,
    /// Bounded result count.
    pub limit: usize,
}
/// Vector-memory adapter, implemented by Rust RuVector/RVF first.
pub trait VectorMemory {
    /// Adapter failure.
    type Error;
    /// Similarity result.
    type Match;
    /// Inserts only under an explicit participant key.
    fn insert(
        &mut self,
        key: &ScopedVectorKey,
        embedding: &EmbeddingVector,
    ) -> Result<(), Self::Error>;
    /// Queries only within one participant scope.
    fn search(&self, query: ScopedVectorQuery<'_>) -> Result<Vec<Self::Match>, Self::Error>;
    /// Cryptographically erases a participant namespace and rebuilds indexes.
    fn erase_participant(&mut self, participant: &ParticipantPseudonym) -> Result<(), Self::Error>;
}
