//! Append-only authority and deterministic Rust reference vector adapter.
use crate::{
    application::{PatternProfileRepository, ScopedVectorKey, ScopedVectorQuery, VectorMemory},
    domain::{EmbeddingVector, ParticipantPseudonym, PatternProfile},
};
use chacha20poly1305::{
    ChaCha20Poly1305, KeyInit,
    aead::{Aead, Payload},
};
use rand_core::{OsRng, RngCore};
use std::{cmp::Ordering, collections::BTreeMap, error::Error, fmt};

/// Append-only authoritative profile snapshots; corrections never replace history.
#[derive(Default)]
pub struct AppendOnlyPatternRepository {
    history: BTreeMap<String, Vec<PatternProfile>>,
}
impl AppendOnlyPatternRepository {
    /// Entire immutable profile history.
    #[must_use]
    pub fn history(&self, id: &str) -> &[PatternProfile] {
        self.history.get(id).map(Vec::as_slice).unwrap_or_default()
    }
    /// Latest profile state.
    #[must_use]
    pub fn latest(&self, id: &str) -> Option<&PatternProfile> {
        self.history.get(id).and_then(|v| v.last())
    }
}
impl PatternProfileRepository for AppendOnlyPatternRepository {
    type Error = VectorAdapterError;
    fn save(&mut self, profile: &PatternProfile) -> Result<(), Self::Error> {
        let history = self.history.entry(profile.id().to_owned()).or_default();
        if history.last() == Some(profile) {
            return Ok(());
        }
        if history.last().is_some_and(|prior| prior.is_forgotten()) {
            return Err(VectorAdapterError::Forgotten);
        }
        history.push(profile.clone());
        Ok(())
    }
}

struct CipherRecord {
    participant: ParticipantPseudonym,
    anchor_id: String,
    nonce: [u8; 12],
    ciphertext: Vec<u8>,
}
/// Deterministic similarity result with no cross-participant payload.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceVectorMatch {
    /// Anchor identity.
    pub anchor_id: String,
    /// Cosine similarity.
    pub similarity: f32,
}

/// Bounded Rust reference adapter. Journal payloads are encrypted; the index is derived.
#[derive(Default)]
pub struct ReferenceVectorMemory {
    journal: Vec<CipherRecord>,
    keys: BTreeMap<ParticipantPseudonym, [u8; 32]>,
    counters: BTreeMap<ParticipantPseudonym, u64>,
    index: BTreeMap<ParticipantPseudonym, BTreeMap<String, Vec<f32>>>,
    tombstones: Vec<ParticipantPseudonym>,
}
impl ReferenceVectorMemory {
    /// Whether participant encryption-key material remains.
    #[must_use]
    pub fn has_key(&self, participant: &ParticipantPseudonym) -> bool {
        self.keys.contains_key(participant)
    }
    /// Encrypted authoritative journal record count.
    #[must_use]
    pub fn journal_len(&self) -> usize {
        self.journal.len()
    }
    /// Rebuilds the derived index only from decryptable, non-tombstoned authority.
    pub fn rebuild(&mut self) -> Result<(), VectorAdapterError> {
        self.index.clear();
        for record in &self.journal {
            if self.tombstones.contains(&record.participant) {
                continue;
            }
            let Some(key) = self.keys.get(&record.participant) else {
                continue;
            };
            let cipher = ChaCha20Poly1305::new(key.into());
            let plaintext = cipher
                .decrypt(
                    (&record.nonce).into(),
                    Payload {
                        msg: &record.ciphertext,
                        aad: record.anchor_id.as_bytes(),
                    },
                )
                .map_err(|_| VectorAdapterError::Integrity)?;
            if plaintext.len() % 4 != 0 {
                return Err(VectorAdapterError::Integrity);
            }
            let values = plaintext
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes(b.try_into().expect("four bytes")))
                .collect();
            self.index
                .entry(record.participant.clone())
                .or_default()
                .insert(record.anchor_id.clone(), values);
        }
        Ok(())
    }
}
impl VectorMemory for ReferenceVectorMemory {
    type Error = VectorAdapterError;
    type Match = ReferenceVectorMatch;
    fn insert(
        &mut self,
        key: &ScopedVectorKey,
        embedding: &EmbeddingVector,
    ) -> Result<(), Self::Error> {
        if self.tombstones.contains(&key.participant) || key.anchor_id.trim().is_empty() {
            return Err(VectorAdapterError::Forgotten);
        }
        if self
            .index
            .get(&key.participant)
            .is_some_and(|m| m.contains_key(&key.anchor_id))
        {
            return Err(VectorAdapterError::Duplicate);
        }
        let secret = self.keys.entry(key.participant.clone()).or_insert_with(|| {
            let mut k = [0; 32];
            OsRng.fill_bytes(&mut k);
            k
        });
        let counter = self.counters.entry(key.participant.clone()).or_default();
        *counter = counter.checked_add(1).ok_or(VectorAdapterError::Capacity)?;
        let mut nonce = [0; 12];
        nonce[4..].copy_from_slice(&counter.to_be_bytes());
        let bytes = embedding
            .values()
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        let ciphertext = ChaCha20Poly1305::new((&*secret).into())
            .encrypt(
                (&nonce).into(),
                Payload {
                    msg: &bytes,
                    aad: key.anchor_id.as_bytes(),
                },
            )
            .map_err(|_| VectorAdapterError::Integrity)?;
        self.journal.push(CipherRecord {
            participant: key.participant.clone(),
            anchor_id: key.anchor_id.clone(),
            nonce,
            ciphertext,
        });
        self.index
            .entry(key.participant.clone())
            .or_default()
            .insert(key.anchor_id.clone(), embedding.values().to_vec());
        Ok(())
    }
    fn search(&self, query: ScopedVectorQuery<'_>) -> Result<Vec<Self::Match>, Self::Error> {
        if query.limit == 0 || query.limit > 1024 {
            return Err(VectorAdapterError::Capacity);
        }
        let mut matches = self
            .index
            .get(query.participant)
            .into_iter()
            .flat_map(|m| m.iter())
            .filter_map(|(id, v)| {
                cosine(v, query.embedding.values()).map(|similarity| ReferenceVectorMatch {
                    anchor_id: id.clone(),
                    similarity,
                })
            })
            .collect::<Vec<_>>();
        matches.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.anchor_id.cmp(&b.anchor_id))
        });
        matches.truncate(query.limit);
        Ok(matches)
    }
    fn erase_participant(&mut self, participant: &ParticipantPseudonym) -> Result<(), Self::Error> {
        self.keys.remove(participant);
        self.index.remove(participant);
        self.tombstones.push(participant.clone());
        self.rebuild()
    }
}
fn cosine(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() {
        return None;
    }
    let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
    let an = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let bn = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    (an > 0.0 && bn > 0.0).then_some(dot / (an * bn))
}
/// Stable reference-adapter failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorAdapterError {
    /// Erased namespace or unsafe key.
    Forgotten,
    /// Duplicate immutable key.
    Duplicate,
    /// Invalid bound.
    Capacity,
    /// Ciphertext integrity/decryption failure.
    Integrity,
}
impl fmt::Display for VectorAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vector adapter rejected operation: {self:?}")
    }
}
impl Error for VectorAdapterError {}
