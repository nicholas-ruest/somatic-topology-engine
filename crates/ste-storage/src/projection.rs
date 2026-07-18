//! Idempotent projection application with atomic snapshot/checkpoint commit.

use crate::{JournalError, JournalRecord};

/// Deterministic handler that applies one journal record to a candidate snapshot.
pub trait ProjectionHandler<State> {
    /// Applies a record. The candidate is discarded if this returns an error.
    fn apply(&self, state: &mut State, record: &JournalRecord) -> Result<(), JournalError>;
}

/// Atomic projection persistence port.
pub trait ProjectionStore<State> {
    /// Last committed sequence.
    fn checkpoint(&self) -> Option<u64>;
    /// Current committed snapshot.
    fn snapshot(&self) -> &State;
    /// Atomically commits both snapshot and checkpoint.
    fn commit(&mut self, checkpoint: u64, state: State) -> Result<(), JournalError>;
}

/// In-memory atomic projection store with an interruption harness.
#[derive(Clone, Debug)]
pub struct InMemoryProjectionStore<State> {
    checkpoint: Option<u64>,
    snapshot: State,
    fail_next: bool,
}

impl<State: Default> Default for InMemoryProjectionStore<State> {
    fn default() -> Self {
        Self {
            checkpoint: None,
            snapshot: State::default(),
            fail_next: false,
        }
    }
}

impl<State> InMemoryProjectionStore<State> {
    /// Makes the next commit fail before either value changes.
    pub fn fail_next_commit(&mut self) {
        self.fail_next = true;
    }

    /// Returns the committed checkpoint.
    #[must_use]
    pub const fn checkpoint(&self) -> Option<u64> {
        self.checkpoint
    }

    /// Returns the committed snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &State {
        &self.snapshot
    }
}

impl<State> ProjectionStore<State> for InMemoryProjectionStore<State> {
    fn checkpoint(&self) -> Option<u64> {
        self.checkpoint
    }

    fn snapshot(&self) -> &State {
        &self.snapshot
    }

    fn commit(&mut self, checkpoint: u64, state: State) -> Result<(), JournalError> {
        if self.fail_next {
            self.fail_next = false;
            return Err(JournalError::CheckpointInterrupted);
        }
        self.snapshot = state;
        self.checkpoint = Some(checkpoint);
        Ok(())
    }
}

/// Applies ordered records idempotently through an atomic store.
#[derive(Clone, Debug)]
pub struct ProjectionEngine<Store, Handler> {
    store: Store,
    handler: Handler,
}

impl<Store, Handler> ProjectionEngine<Store, Handler> {
    /// Creates an engine over a store and deterministic handler.
    #[must_use]
    pub const fn new(store: Store, handler: Handler) -> Self {
        Self { store, handler }
    }

    /// Borrows the projection store.
    #[must_use]
    pub const fn store(&self) -> &Store {
        &self.store
    }

    /// Mutably borrows the projection store for controlled fault injection.
    pub fn store_mut(&mut self) -> &mut Store {
        &mut self.store
    }
}

impl<Store, Handler> ProjectionEngine<Store, Handler> {
    /// Applies records after the committed checkpoint and atomically advances it.
    pub fn apply<State>(&mut self, records: &[JournalRecord]) -> Result<(), JournalError>
    where
        State: Clone,
        Store: ProjectionStore<State>,
        Handler: ProjectionHandler<State>,
    {
        let checkpoint = self.store.checkpoint().unwrap_or(0);
        let pending: Vec<_> = records
            .iter()
            .filter(|record| record.sequence > checkpoint)
            .collect();
        let Some(last) = pending.last() else {
            return Ok(());
        };
        let mut candidate = self.store.snapshot().clone();
        for record in &pending {
            self.handler.apply(&mut candidate, record)?;
        }
        self.store.commit(last.sequence, candidate)
    }
}
