//! Append-only, checksummed persistence and deterministic projection primitives.
#![forbid(unsafe_code)]

/// Versioned envelope encryption and managed-key extension points.
pub mod crypto;
mod journal;
/// Retention, export, deletion, reset, and decommission orchestration.
pub mod lifecycle;
mod projection;

pub use journal::{
    ChunkRef, ChunkStore, DataClass, DataPartition, EventUpcaster, Fault, InMemoryChunkStore,
    InMemoryJournalIo, InspectionReport, Journal, JournalError, JournalIo, JournalRecord,
    JournalStore, RecoveryMode, RecoveryReport, UpcastEvent,
};
pub use projection::{
    InMemoryProjectionStore, ProjectionEngine, ProjectionHandler, ProjectionStore,
};
