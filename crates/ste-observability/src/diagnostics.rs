//! Separated records, schema redaction, health, and rotating storage.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
/// Record security/lifecycle class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum RecordClass {
    /// Redacted diagnostics.
    Diagnostic,
    /// Security event.
    Security,
    /// Domain audit.
    Audit,
}
/// Structured record; free-form payload dumping is not supported.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// Class.
    pub class: RecordClass,
    /// Stable event code.
    pub code: String,
    /// UTC correlation time.
    pub time_ns: u64,
    /// Structured fields.
    pub fields: BTreeMap<String, String>,
}
/// Field allow-list per record code; non-allowed fields are removed, not key-pattern guessed.
#[derive(Default)]
pub struct RedactionSchema {
    allowed: BTreeMap<(RecordClass, String), BTreeSet<String>>,
}
impl RedactionSchema {
    /// Registers exact exportable fields.
    pub fn allow(
        &mut self,
        class: RecordClass,
        code: impl Into<String>,
        fields: impl IntoIterator<Item = String>,
    ) {
        self.allowed
            .insert((class, code.into()), fields.into_iter().collect());
    }
    /// Produces schema-safe derivative.
    #[must_use]
    pub fn redact(&self, record: &Record) -> Record {
        let allowed = self.allowed.get(&(record.class, record.code.clone()));
        let fields = record
            .fields
            .iter()
            .filter(|(k, _)| allowed.is_some_and(|set| set.contains(*k)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Record {
            class: record.class,
            code: record.code.clone(),
            time_ns: record.time_ns,
            fields,
        }
    }
}
/// Bounded rotating stores kept separately per class.
pub struct RecordStore {
    capacity: usize,
    records: BTreeMap<RecordClass, VecDeque<Record>>,
    dropped: BTreeMap<RecordClass, u64>,
}
impl RecordStore {
    /// Creates a per-class record cap.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            records: BTreeMap::new(),
            dropped: BTreeMap::new(),
        }
    }
    /// Inserts and rotates oldest within only that class.
    pub fn push(&mut self, record: Record) {
        let class = record.class;
        let queue = self.records.entry(class).or_default();
        if queue.len() >= self.capacity {
            queue.pop_front();
            *self.dropped.entry(class).or_default() += 1;
        }
        queue.push_back(record);
    }
    /// Returns class-isolated records.
    pub fn records(&self, class: RecordClass) -> impl Iterator<Item = &Record> {
        self.records.get(&class).into_iter().flatten()
    }
    /// Rotation count.
    #[must_use]
    pub fn dropped(&self, class: RecordClass) -> u64 {
        self.dropped.get(&class).copied().unwrap_or(0)
    }
}
/// Local health summary without sensitive payloads.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HealthSnapshot {
    /// Overall state code.
    pub state: String,
    /// Queue saturation count.
    pub saturated_queues: u64,
    /// Dropped diagnostic count.
    pub dropped_records: u64,
}
