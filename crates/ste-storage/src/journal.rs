//! Checksummed journal, recovery, compaction, chunk, and storage extension ports.

use std::{collections::BTreeMap, error::Error, fmt, fmt::Write as _};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Canonical storage data classes shared by journal, crypto, and lifecycle adapters.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[repr(u8)]
pub enum DataClass {
    /// Raw channel-state information.
    RawCsi,
    /// Derived signal observations.
    Observation,
    /// Physiological estimates.
    Physiology,
    /// Latent-state assessments.
    LatentState,
    /// Participant personalization anchors.
    Anchor,
    /// Consent and policy authority.
    ConsentPolicy,
    /// Domain audit records.
    Audit,
    /// Security events.
    Security,
    /// Redacted diagnostics.
    Diagnostics,
    /// Scientific provenance.
    Provenance,
}

/// Compatibility name emphasizing physical journal partitioning by data class.
pub type DataPartition = DataClass;

/// A verified journal record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JournalRecord {
    /// Monotonic sequence within one data-class partition.
    pub sequence: u64,
    /// Event schema major version.
    pub schema_version: u16,
    /// Physical/logical journal partition.
    pub data_class: DataClass,
    /// Bounded event payload.
    pub payload: Vec<u8>,
    checksum: String,
}

impl JournalRecord {
    /// Constructs a record with a checksum covering metadata and payload.
    #[must_use]
    pub fn verified(
        sequence: u64,
        schema_version: u16,
        data_class: DataClass,
        payload: Vec<u8>,
    ) -> Self {
        let checksum = checksum(sequence, schema_version, data_class, &payload);
        Self {
            sequence,
            schema_version,
            data_class,
            payload,
            checksum,
        }
    }

    fn verify(&self) -> bool {
        self.checksum
            == checksum(
                self.sequence,
                self.schema_version,
                self.data_class,
                &self.payload,
            )
    }
}

fn checksum(sequence: u64, version: u16, class: DataClass, payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sequence.to_be_bytes());
    hasher.update(version.to_be_bytes());
    hasher.update([class as u8]);
    hasher.update((payload.len() as u64).to_be_bytes());
    hasher.update(payload);
    encode_hex(&hasher.finalize())
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

/// Upcast result preserving an explicit resulting schema version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpcastEvent {
    /// Result schema version.
    pub schema_version: u16,
    /// Migrated payload.
    pub payload: Vec<u8>,
}

/// Port for deterministic, side-effect-free event schema migration.
pub trait EventUpcaster {
    /// Upcasts one verified payload or returns a typed migration error.
    fn upcast(&self, version: u16, payload: &[u8]) -> Result<UpcastEvent, JournalError>;
}

/// Atomic byte persistence required by the journal core.
pub trait JournalIo {
    /// Returns all bytes in one partition.
    fn read(&self, class: DataClass) -> Result<Vec<u8>, JournalError>;
    /// Appends all bytes or none of them.
    fn append_atomic(&mut self, class: DataClass, bytes: &[u8]) -> Result<(), JournalError>;
    /// Atomically replaces a partition during verified compaction.
    fn replace_atomic(&mut self, class: DataClass, bytes: &[u8]) -> Result<(), JournalError>;
}

/// Injectable storage failures used by deterministic fault tests and adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fault {
    /// Capacity exhausted before an atomic operation.
    DiskFull,
    /// Atomic replacement was interrupted before commit.
    Interrupted,
}

/// Deterministic in-memory I/O implementation and fault harness.
#[derive(Clone, Debug, Default)]
pub struct InMemoryJournalIo {
    partitions: BTreeMap<DataClass, Vec<u8>>,
    next_fault: Option<Fault>,
}

impl InMemoryJournalIo {
    /// Injects a failure into the next atomic mutation.
    pub fn fail_next(&mut self, fault: Fault) {
        self.next_fault = Some(fault);
    }

    /// Appends incomplete/raw bytes to simulate a process tear.
    pub fn append_raw(&mut self, class: DataClass, bytes: &[u8]) -> Result<(), JournalError> {
        self.partitions.entry(class).or_default().extend(bytes);
        Ok(())
    }

    /// Corrupts one existing byte for recovery tests.
    pub fn corrupt_byte(
        &mut self,
        class: DataClass,
        offset: usize,
        value: u8,
    ) -> Result<(), JournalError> {
        let byte = self
            .partitions
            .get_mut(&class)
            .and_then(|bytes| bytes.get_mut(offset))
            .ok_or(JournalError::Io("corruption offset is out of range"))?;
        *byte = value;
        Ok(())
    }

    /// Borrows raw bytes for immutable recovery assertions.
    #[must_use]
    pub fn bytes(&self, class: DataClass) -> &[u8] {
        self.partitions.get(&class).map_or(&[], Vec::as_slice)
    }

    fn take_fault(&mut self) -> Result<(), JournalError> {
        match self.next_fault.take() {
            Some(Fault::DiskFull) => Err(JournalError::DiskFull),
            Some(Fault::Interrupted) => Err(JournalError::Io("atomic operation interrupted")),
            None => Ok(()),
        }
    }
}

impl JournalIo for InMemoryJournalIo {
    fn read(&self, class: DataClass) -> Result<Vec<u8>, JournalError> {
        Ok(self.bytes(class).to_vec())
    }

    fn append_atomic(&mut self, class: DataClass, bytes: &[u8]) -> Result<(), JournalError> {
        self.take_fault()?;
        self.partitions.entry(class).or_default().extend(bytes);
        Ok(())
    }

    fn replace_atomic(&mut self, class: DataClass, bytes: &[u8]) -> Result<(), JournalError> {
        self.take_fault()?;
        self.partitions.insert(class, bytes.to_vec());
        Ok(())
    }
}

/// Append-only journal over an atomic storage port.
#[derive(Clone, Debug)]
pub struct Journal<I> {
    io: I,
    max_record_bytes: usize,
}

impl<I: JournalIo> Journal<I> {
    /// Creates a journal with a strict per-record payload bound.
    #[must_use]
    pub const fn new(io: I, max_record_bytes: usize) -> Self {
        Self {
            io,
            max_record_bytes,
        }
    }

    /// Borrows the backing extension port implementation.
    #[must_use]
    pub const fn io(&self) -> &I {
        &self.io
    }

    /// Mutably borrows backing I/O, primarily for controlled fault injection.
    pub fn io_mut(&mut self) -> &mut I {
        &mut self.io
    }

    /// Appends a checksummed record atomically and returns its partition sequence.
    pub fn append(
        &mut self,
        class: DataClass,
        schema_version: u16,
        payload: &[u8],
    ) -> Result<u64, JournalError> {
        if payload.len() > self.max_record_bytes {
            return Err(JournalError::RecordTooLarge {
                actual: payload.len(),
                maximum: self.max_record_bytes,
            });
        }
        if schema_version == 0 {
            return Err(JournalError::UnsupportedSchema(0));
        }
        let inspected = inspect_bytes(class, &self.io.read(class)?)?;
        let sequence = inspected
            .last_sequence
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(JournalError::Io("journal sequence exhausted"))?;
        let record = JournalRecord::verified(sequence, schema_version, class, payload.to_vec());
        let mut encoded = serde_json::to_vec(&record)
            .map_err(|_| JournalError::Io("record serialization failed"))?;
        encoded.push(b'\n');
        self.io.append_atomic(class, &encoded)?;
        Ok(sequence)
    }

    /// Verifies and upcasts all committed records, ignoring only an incomplete final line.
    pub fn recover(
        &self,
        class: DataClass,
        upcaster: &dyn EventUpcaster,
    ) -> Result<RecoveryReport, JournalError> {
        recover_bytes(class, &self.io.read(class)?, upcaster)
    }

    /// Atomically removes records before `retain_from`, after full verification/upcast.
    pub fn compact_before(
        &mut self,
        class: DataClass,
        retain_from: u64,
        upcaster: &dyn EventUpcaster,
    ) -> Result<InspectionReport, JournalError> {
        let recovered = self.recover(class, upcaster)?;
        let retained: Vec<_> = recovered
            .records
            .into_iter()
            .filter(|record| record.sequence >= retain_from)
            .collect();
        let mut bytes = Vec::new();
        for record in retained {
            let normalized = JournalRecord::verified(
                record.sequence,
                record.schema_version,
                class,
                record.payload,
            );
            bytes.extend(
                serde_json::to_vec(&normalized)
                    .map_err(|_| JournalError::Io("record serialization failed"))?,
            );
            bytes.push(b'\n');
        }
        self.io.replace_atomic(class, &bytes)?;
        inspect_bytes(class, &bytes)
    }
}

/// CLI/operations-facing typed inspection and recovery contract.
pub trait JournalStore {
    /// Verifies checksums and sequence continuity without decoding event meaning.
    fn inspect(&self, class: DataClass) -> Result<InspectionReport, JournalError>;
    /// Rebuilds a typed sequence through the supplied deterministic upcaster.
    fn rebuild(
        &self,
        class: DataClass,
        upcaster: &dyn EventUpcaster,
    ) -> Result<RecoveryReport, JournalError>;
}

impl<I: JournalIo> JournalStore for Journal<I> {
    fn inspect(&self, class: DataClass) -> Result<InspectionReport, JournalError> {
        inspect_bytes(class, &self.io.read(class)?)
    }

    fn rebuild(
        &self,
        class: DataClass,
        upcaster: &dyn EventUpcaster,
    ) -> Result<RecoveryReport, JournalError> {
        self.recover(class, upcaster)
    }
}

/// Result of structural journal inspection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectionReport {
    /// Partition inspected.
    pub data_class: DataClass,
    /// Number of verified records.
    pub verified_records: usize,
    /// Last verified sequence, if non-empty.
    pub last_sequence: Option<u64>,
    /// Whether an incomplete final line was ignored.
    pub torn_tail: bool,
}

/// Recovery disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryMode {
    /// All stored bytes were verified.
    Complete,
    /// Only an incomplete final line was discarded.
    TornTailIgnored,
}

/// Deterministic recovery result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    /// Verified, upcasted records in sequence order.
    pub records: Vec<JournalRecord>,
    /// Last verified committed sequence.
    pub last_verified_sequence: Option<u64>,
    /// Recovery disposition.
    pub mode: RecoveryMode,
}

fn inspect_bytes(class: DataClass, bytes: &[u8]) -> Result<InspectionReport, JournalError> {
    let (records, torn_tail) = decode_verified(class, bytes)?;
    Ok(InspectionReport {
        data_class: class,
        verified_records: records.len(),
        last_sequence: records.last().map(|record| record.sequence),
        torn_tail,
    })
}

fn recover_bytes(
    class: DataClass,
    bytes: &[u8],
    upcaster: &dyn EventUpcaster,
) -> Result<RecoveryReport, JournalError> {
    let (records, torn_tail) = decode_verified(class, bytes)?;
    let mut upcasted = Vec::with_capacity(records.len());
    for record in records {
        let event = upcaster.upcast(record.schema_version, &record.payload)?;
        upcasted.push(JournalRecord::verified(
            record.sequence,
            event.schema_version,
            class,
            event.payload,
        ));
    }
    Ok(RecoveryReport {
        last_verified_sequence: upcasted.last().map(|record| record.sequence),
        records: upcasted,
        mode: if torn_tail {
            RecoveryMode::TornTailIgnored
        } else {
            RecoveryMode::Complete
        },
    })
}

fn decode_verified(
    expected_class: DataClass,
    bytes: &[u8],
) -> Result<(Vec<JournalRecord>, bool), JournalError> {
    let torn_tail = !bytes.is_empty() && !bytes.ends_with(b"\n");
    let committed = if torn_tail {
        let end = bytes.iter().rposition(|byte| *byte == b'\n').unwrap_or(0);
        &bytes[..end]
    } else {
        bytes.strip_suffix(b"\n").unwrap_or(bytes)
    };
    if committed.is_empty() {
        return Ok((Vec::new(), torn_tail));
    }
    let mut records: Vec<JournalRecord> = Vec::new();
    for (index, line) in committed.split(|byte| *byte == b'\n').enumerate() {
        let record: JournalRecord =
            serde_json::from_slice(line).map_err(|_| JournalError::MalformedRecord {
                record_index: index,
            })?;
        let sequence_valid = if let Some(previous) = records.last() {
            previous.sequence.checked_add(1) == Some(record.sequence)
        } else {
            record.sequence > 0
        };
        if !sequence_valid || record.data_class != expected_class || !record.verify() {
            return Err(JournalError::CorruptRecord {
                sequence: record.sequence,
            });
        }
        records.push(record);
    }
    Ok((records, torn_tail))
}

/// Content-addressed reference to a bounded large payload.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ChunkRef {
    /// SHA-256 content digest.
    pub digest: String,
    /// Payload size in bytes.
    pub size: usize,
}

/// Extension port for bounded, content-addressed large payload storage.
pub trait ChunkStore {
    /// Stores a chunk if the total budget permits.
    fn put(&mut self, payload: &[u8]) -> Result<ChunkRef, JournalError>;
    /// Retrieves and verifies a chunk.
    fn get(&self, reference: &ChunkRef) -> Result<Vec<u8>, JournalError>;
}

/// In-memory bounded chunk implementation.
#[derive(Clone, Debug)]
pub struct InMemoryChunkStore {
    maximum_bytes: usize,
    used_bytes: usize,
    chunks: BTreeMap<String, Vec<u8>>,
}

impl InMemoryChunkStore {
    /// Creates a store with a strict aggregate byte budget.
    #[must_use]
    pub const fn new(maximum_bytes: usize) -> Self {
        Self {
            maximum_bytes,
            used_bytes: 0,
            chunks: BTreeMap::new(),
        }
    }
}

impl ChunkStore for InMemoryChunkStore {
    fn put(&mut self, payload: &[u8]) -> Result<ChunkRef, JournalError> {
        let digest = encode_hex(&Sha256::digest(payload));
        if !self.chunks.contains_key(&digest)
            && self.used_bytes.saturating_add(payload.len()) > self.maximum_bytes
        {
            return Err(JournalError::ChunkBudgetExceeded);
        }
        if !self.chunks.contains_key(&digest) {
            self.used_bytes += payload.len();
            self.chunks.insert(digest.clone(), payload.to_vec());
        }
        Ok(ChunkRef {
            digest,
            size: payload.len(),
        })
    }

    fn get(&self, reference: &ChunkRef) -> Result<Vec<u8>, JournalError> {
        let payload = self
            .chunks
            .get(&reference.digest)
            .ok_or(JournalError::ChunkMissing)?;
        if payload.len() != reference.size
            || encode_hex(&Sha256::digest(payload)) != reference.digest
        {
            return Err(JournalError::ChunkCorrupt);
        }
        Ok(payload.clone())
    }
}

/// Typed storage/recovery failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JournalError {
    /// Atomic write could not begin because storage is full.
    DiskFull,
    /// Payload exceeds the configured bound.
    RecordTooLarge {
        /// Attempted payload bytes.
        actual: usize,
        /// Configured payload bound.
        maximum: usize,
    },
    /// A record was syntactically invalid.
    MalformedRecord {
        /// Zero-based committed line index.
        record_index: usize,
    },
    /// Sequence, partition, or checksum verification failed.
    CorruptRecord {
        /// Parsed sequence, when available.
        sequence: u64,
    },
    /// Event schema is unsupported.
    UnsupportedSchema(u16),
    /// Upcast did not complete.
    MigrationInterrupted,
    /// Projection snapshot/checkpoint atomic commit did not complete.
    CheckpointInterrupted,
    /// Chunk budget would be exceeded.
    ChunkBudgetExceeded,
    /// Chunk reference does not exist.
    ChunkMissing,
    /// Chunk content does not match its reference.
    ChunkCorrupt,
    /// Adapter-level non-sensitive I/O reason.
    Io(&'static str),
}

impl fmt::Display for JournalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for JournalError {}
