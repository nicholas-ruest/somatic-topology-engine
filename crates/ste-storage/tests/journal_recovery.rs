//! Recovery and fault semantics for the append-only storage core.

use std::collections::BTreeMap;

use ste_storage::{
    ChunkStore, DataPartition, EventUpcaster, Fault, InMemoryChunkStore, InMemoryJournalIo,
    InMemoryProjectionStore, Journal, JournalError, JournalRecord, ProjectionEngine,
    ProjectionHandler, RecoveryMode, UpcastEvent,
};

#[derive(Default)]
struct RenameV1;

impl EventUpcaster for RenameV1 {
    fn upcast(&self, version: u16, payload: &[u8]) -> Result<UpcastEvent, JournalError> {
        match version {
            1 => Ok(UpcastEvent {
                schema_version: 2,
                payload: [b"v2:".as_slice(), payload].concat(),
            }),
            2 => Ok(UpcastEvent {
                schema_version: 2,
                payload: payload.to_vec(),
            }),
            other => Err(JournalError::UnsupportedSchema(other)),
        }
    }
}

#[test]
fn partitioned_journal_rebuild_is_deterministic_and_upcasted() {
    let io = InMemoryJournalIo::default();
    let mut journal = Journal::new(io, 1024);
    journal.append(DataPartition::Audit, 1, b"one").unwrap();
    journal.append(DataPartition::Audit, 2, b"two").unwrap();
    journal
        .append(DataPartition::Security, 2, b"isolated")
        .unwrap();

    let first = journal.recover(DataPartition::Audit, &RenameV1).unwrap();
    let second = journal.recover(DataPartition::Audit, &RenameV1).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.records.len(), 2);
    assert_eq!(first.records[0].payload, b"v2:one");
    assert_eq!(first.records[1].payload, b"two");
    assert_eq!(
        journal
            .recover(DataPartition::Security, &RenameV1)
            .unwrap()
            .records
            .len(),
        1
    );
}

#[test]
fn torn_tail_recovers_only_last_verified_record_and_reports_loss() {
    let io = InMemoryJournalIo::default();
    let mut journal = Journal::new(io, 1024);
    journal
        .append(DataPartition::Audit, 2, b"committed")
        .unwrap();
    journal
        .io_mut()
        .append_raw(DataPartition::Audit, br#"{"sequence":2"#)
        .unwrap();

    let recovered = journal.recover(DataPartition::Audit, &RenameV1).unwrap();
    assert_eq!(recovered.records.len(), 1);
    assert_eq!(recovered.mode, RecoveryMode::TornTailIgnored);
    assert_eq!(recovered.last_verified_sequence, Some(1));
}

#[test]
fn checksum_corruption_is_explicit_and_never_silently_skipped() {
    let io = InMemoryJournalIo::default();
    let mut journal = Journal::new(io, 1024);
    journal
        .append(DataPartition::Audit, 2, b"sensitive")
        .unwrap();
    journal
        .io_mut()
        .corrupt_byte(DataPartition::Audit, 20, b'X')
        .unwrap();

    assert!(matches!(
        journal.recover(DataPartition::Audit, &RenameV1),
        Err(JournalError::CorruptRecord { .. }) | Err(JournalError::MalformedRecord { .. })
    ));
}

#[test]
fn disk_full_append_is_atomic_and_does_not_consume_sequence() {
    let io = InMemoryJournalIo::default();
    let mut journal = Journal::new(io, 1024);
    journal.io_mut().fail_next(Fault::DiskFull);
    assert_eq!(
        journal.append(DataPartition::RawCsi, 2, b"lost"),
        Err(JournalError::DiskFull)
    );
    let sequence = journal.append(DataPartition::RawCsi, 2, b"kept").unwrap();
    assert_eq!(sequence, 1);
    let recovered = journal.recover(DataPartition::RawCsi, &RenameV1).unwrap();
    assert_eq!(recovered.records.len(), 1);
    assert_eq!(recovered.records[0].payload, b"kept");
}

#[derive(Default)]
struct CountProjection;

impl ProjectionHandler<BTreeMap<String, u64>> for CountProjection {
    fn apply(
        &self,
        state: &mut BTreeMap<String, u64>,
        record: &JournalRecord,
    ) -> Result<(), JournalError> {
        *state
            .entry(String::from_utf8_lossy(&record.payload).into())
            .or_default() += 1;
        Ok(())
    }
}

#[test]
fn checkpoint_commit_is_atomic_and_replay_is_idempotent() {
    let store = InMemoryProjectionStore::<BTreeMap<String, u64>>::default();
    let mut engine = ProjectionEngine::new(store, CountProjection);
    let records = vec![
        JournalRecord::verified(1, 2, DataPartition::Audit, b"a".to_vec()),
        JournalRecord::verified(2, 2, DataPartition::Audit, b"b".to_vec()),
    ];
    engine.apply(&records).unwrap();
    engine.apply(&records).unwrap();
    assert_eq!(engine.store().checkpoint(), Some(2));
    assert_eq!(engine.store().snapshot().get("a"), Some(&1));

    engine.store_mut().fail_next_commit();
    let third = JournalRecord::verified(3, 2, DataPartition::Audit, b"c".to_vec());
    assert_eq!(
        engine.apply(&[third]),
        Err(JournalError::CheckpointInterrupted)
    );
    assert_eq!(engine.store().checkpoint(), Some(2));
    assert!(!engine.store().snapshot().contains_key("c"));
}

struct FailingMigration;

impl EventUpcaster for FailingMigration {
    fn upcast(&self, _: u16, _: &[u8]) -> Result<UpcastEvent, JournalError> {
        Err(JournalError::MigrationInterrupted)
    }
}

#[test]
fn interrupted_migration_preserves_original_bytes() {
    let io = InMemoryJournalIo::default();
    let mut journal = Journal::new(io, 1024);
    journal.append(DataPartition::Audit, 1, b"old").unwrap();
    let before = journal.io().bytes(DataPartition::Audit).to_vec();
    assert_eq!(
        journal.recover(DataPartition::Audit, &FailingMigration),
        Err(JournalError::MigrationInterrupted)
    );
    assert_eq!(journal.io().bytes(DataPartition::Audit), before);
}

#[test]
fn bounded_chunks_are_content_addressed_and_reject_overflow() {
    let mut chunks = InMemoryChunkStore::new(8);
    let reference = chunks.put(b"1234").unwrap();
    assert_eq!(chunks.get(&reference).unwrap(), b"1234");
    assert_eq!(chunks.put(b"12345"), Err(JournalError::ChunkBudgetExceeded));
    assert_eq!(chunks.get(&reference).unwrap(), b"1234");
}

#[test]
fn verified_compaction_is_atomic_and_retains_requested_suffix() {
    let io = InMemoryJournalIo::default();
    let mut journal = Journal::new(io, 1024);
    for payload in [b"one".as_slice(), b"two", b"three"] {
        journal
            .append(DataPartition::Diagnostics, 2, payload)
            .unwrap();
    }
    journal
        .compact_before(DataPartition::Diagnostics, 3, &RenameV1)
        .unwrap();
    let recovered = journal
        .recover(DataPartition::Diagnostics, &RenameV1)
        .unwrap();
    assert_eq!(recovered.records.len(), 1);
    assert_eq!(recovered.records[0].sequence, 3);
}
