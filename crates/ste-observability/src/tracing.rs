//! Bounded local trace spans.
use serde::Serialize;
use std::collections::VecDeque;
/// Structured local span.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LocalSpan {
    /// Trace identifier.
    pub trace_id: String,
    /// Stable operation.
    pub operation: String,
    /// Start.
    pub start_ns: u64,
    /// End.
    pub end_ns: u64,
}
/// Rotating trace store with drops observable.
pub struct TraceStore {
    cap: usize,
    spans: VecDeque<LocalSpan>,
    dropped: u64,
}
impl TraceStore {
    /// Creates bounded store.
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            spans: VecDeque::new(),
            dropped: 0,
        }
    }
    /// Adds a span.
    pub fn push(&mut self, span: LocalSpan) {
        if self.spans.len() >= self.cap {
            self.spans.pop_front();
            self.dropped += 1;
        }
        self.spans.push_back(span);
    }
    /// Dropped spans.
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }
}
