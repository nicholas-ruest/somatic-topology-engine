use crate::WorkflowEvent;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Mutex};

/// Checksummed event envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredEvent {
    pub sequence: u64,
    pub previous_checksum: String,
    pub checksum: String,
    pub event: WorkflowEvent,
}

/// Atomic optimistic append journal.
pub trait Journal {
    fn load(&self, id: &str) -> Result<Vec<StoredEvent>, JournalError>;
    fn append(
        &self,
        id: &str,
        expected: u64,
        events: &[WorkflowEvent],
    ) -> Result<Vec<StoredEvent>, JournalError>;
}

/// Journal integrity or concurrency failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JournalError {
    Conflict,
    Corrupt,
    Serialization,
}

/// Thread-safe reference journal used by tests and embedded deployments.
#[derive(Default)]
pub struct InMemoryJournal {
    inner: Mutex<HashMap<String, Vec<StoredEvent>>>,
}

fn checksum(seq: u64, previous: &str, event: &WorkflowEvent) -> Result<String, JournalError> {
    let bytes = serde_json::to_vec(event).map_err(|_| JournalError::Serialization)?;
    let mut hash = Sha256::new();
    hash.update(seq.to_le_bytes());
    hash.update(previous.as_bytes());
    hash.update(bytes);
    Ok(format!("{:x}", hash.finalize()))
}

impl Journal for InMemoryJournal {
    fn load(&self, id: &str) -> Result<Vec<StoredEvent>, JournalError> {
        let events = self
            .inner
            .lock()
            .map_err(|_| JournalError::Corrupt)?
            .get(id)
            .cloned()
            .unwrap_or_default();
        let mut previous = String::new();
        for (index, event) in events.iter().enumerate() {
            if event.sequence != index as u64 + 1
                || event.previous_checksum != previous
                || checksum(event.sequence, &previous, &event.event)? != event.checksum
            {
                return Err(JournalError::Corrupt);
            }
            previous.clone_from(&event.checksum);
        }
        Ok(events)
    }
    fn append(
        &self,
        id: &str,
        expected: u64,
        new: &[WorkflowEvent],
    ) -> Result<Vec<StoredEvent>, JournalError> {
        let mut all = self.inner.lock().map_err(|_| JournalError::Corrupt)?;
        let stream = all.entry(id.to_owned()).or_default();
        if stream.len() as u64 != expected {
            return Err(JournalError::Conflict);
        }
        let mut previous = stream
            .last()
            .map_or_else(String::new, |e| e.checksum.clone());
        let mut stored = Vec::with_capacity(new.len());
        for event in new {
            let sequence = stream.len() as u64 + 1;
            let sum = checksum(sequence, &previous, event)?;
            let envelope = StoredEvent {
                sequence,
                previous_checksum: previous,
                checksum: sum.clone(),
                event: event.clone(),
            };
            stream.push(envelope.clone());
            stored.push(envelope);
            previous = sum;
        }
        Ok(stored)
    }
}
