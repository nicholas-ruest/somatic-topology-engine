#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Bounded, typed and fail-closed telemetry query plane.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Wire schema version.
pub const SCHEMA_VERSION: u16 = 1;
/// Hard server query point limit.
pub const MAX_POINTS: usize = 10_000;
/// Hard server query duration limit (seven days).
pub const MAX_RANGE_MS: u64 = 604_800_000;

/// Distinguishes clocks and truth semantics.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Live,
    History,
    Replay,
}

/// Allowlisted series; arbitrary topic subscription is impossible.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SeriesKind {
    CaptureHealth,
    PacketContinuity,
    CsiDiagnostic,
    ObservationWindow,
    ApprovedPhysiology,
    ApprovedState,
    DeviceHealth,
    RuntimeMetrics,
    WorkflowProgress,
    SecurityStatus,
}

/// Scope carried by every response and checked server-side.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Aggregate,
    Operator,
    Diagnostic,
}

/// Explicit diagnostic lease. It is narrow, expiring, and auditable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticLease {
    pub purpose: String,
    pub expires_at_ms: u64,
    pub series: BTreeSet<SeriesKind>,
}

/// Authorization context supplied by the trusted gateway.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authorization {
    pub scope: Scope,
    pub now_ms: u64,
    pub diagnostic: Option<DiagnosticLease>,
}

/// Quality flags retained through aggregation.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Quality {
    pub score: f32,
    pub contaminated: bool,
    pub gap: bool,
    pub stale: bool,
}

/// Versioned scalar sample. The closed enum prevents prohibited arbitrary fields.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Sample {
    pub schema_version: u16,
    pub source: Source,
    pub stream_id: String,
    pub series: SeriesKind,
    pub sequence: u64,
    pub event_time_ms: u64,
    pub emitted_time_ms: u64,
    pub unit: String,
    pub algorithm_version: String,
    pub configuration_version: String,
    pub provenance: String,
    pub scope: Scope,
    pub retention_class: String,
    pub quality: Quality,
    pub value: f64,
}

impl Sample {
    /// Validates boundary metadata and floating-point safety.
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema_version != SCHEMA_VERSION
            || self.stream_id.is_empty()
            || self.sequence == 0
            || !self.value.is_finite()
            || !(0.0..=1.0).contains(&self.quality.score)
            || self.unit.len() > 32
            || self.provenance.is_empty()
        {
            return Err(Error::InvalidSample);
        }
        Ok(())
    }
}

/// Explicit gap returned to a slow or resumed consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Gap {
    pub requested_after: u64,
    pub first_available: u64,
    pub dropped: u64,
}

/// Snapshot from a fixed-capacity, nonblocking projection ring.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub samples: Vec<Sample>,
    pub gap: Option<Gap>,
    pub next_sequence: u64,
}

/// Fixed-capacity stream; producers never wait for browsers.
pub struct LiveRing {
    capacity: usize,
    next: u64,
    samples: VecDeque<Sample>,
}

impl LiveRing {
    /// Creates a bounded ring.
    pub fn new(capacity: usize) -> Result<Self, Error> {
        if capacity == 0 || capacity > MAX_POINTS {
            return Err(Error::Bounds);
        }
        Ok(Self {
            capacity,
            next: 1,
            samples: VecDeque::with_capacity(capacity),
        })
    }
    /// Appends, assigning the authoritative sequence and evicting oldest data.
    pub fn push(&mut self, mut sample: Sample) -> Result<u64, Error> {
        sample.validate()?;
        sample.source = Source::Live;
        sample.sequence = self.next;
        self.next = self.next.checked_add(1).ok_or(Error::Bounds)?;
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        let seq = sample.sequence;
        self.samples.push_back(sample);
        Ok(seq)
    }
    /// Resumes strictly after a sequence; reports loss rather than concealing it.
    pub fn resume(
        &self,
        after: u64,
        limit: usize,
        auth: &Authorization,
    ) -> Result<Snapshot, Error> {
        if limit == 0 || limit > self.capacity {
            return Err(Error::Bounds);
        }
        let first = self.samples.front().map_or(self.next, |s| s.sequence);
        let gap = (after.saturating_add(1) < first).then(|| Gap {
            requested_after: after,
            first_available: first,
            dropped: first - after - 1,
        });
        let samples = self
            .samples
            .iter()
            .filter(|s| s.sequence > after)
            .take(limit)
            .map(|s| {
                authorize(s.series, auth)?;
                Ok(s.clone())
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Snapshot {
            samples,
            gap,
            next_sequence: self.next,
        })
    }
}

fn authorize(series: SeriesKind, auth: &Authorization) -> Result<(), Error> {
    if series != SeriesKind::CsiDiagnostic {
        return Ok(());
    }
    let lease = auth.diagnostic.as_ref().ok_or(Error::Forbidden)?;
    if auth.scope != Scope::Diagnostic
        || lease.purpose.trim().is_empty()
        || auth.now_ms >= lease.expires_at_ms
        || !lease.series.contains(&series)
    {
        return Err(Error::Forbidden);
    }
    Ok(())
}

/// Typed, bounded history query.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HistoryQuery {
    pub series: SeriesKind,
    pub start_ms: u64,
    pub end_ms: u64,
    pub limit: usize,
    pub bucket_ms: u64,
    pub cursor_after: Option<u64>,
    pub min_quality_milli: u16,
}

/// Aggregate that preserves extrema, count, contamination, and gaps.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Bucket {
    pub start_ms: u64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub count: u64,
    pub min_quality: f32,
    pub contaminated: bool,
    pub gap: bool,
}

/// History response with opaque monotonic cursor semantics.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HistoryPage {
    pub buckets: Vec<Bucket>,
    pub next_cursor: Option<u64>,
}

/// Executes a typed read-only in-memory projection query.
pub fn query_history(
    data: &[Sample],
    query: &HistoryQuery,
    auth: &Authorization,
) -> Result<HistoryPage, Error> {
    authorize(query.series, auth)?;
    if query.start_ms >= query.end_ms
        || query.end_ms - query.start_ms > MAX_RANGE_MS
        || query.limit == 0
        || query.limit > MAX_POINTS
        || query.bucket_ms == 0
        || query.min_quality_milli > 1000
    {
        return Err(Error::Bounds);
    }
    let mut grouped: BTreeMap<u64, Vec<&Sample>> = BTreeMap::new();
    for s in data {
        s.validate()?;
        if s.series == query.series
            && s.event_time_ms >= query.start_ms
            && s.event_time_ms < query.end_ms
            && s.sequence > query.cursor_after.unwrap_or(0)
            && s.quality.score * 1000.0 >= f32::from(query.min_quality_milli)
        {
            let key = query.start_ms
                + ((s.event_time_ms - query.start_ms) / query.bucket_ms) * query.bucket_ms;
            grouped.entry(key).or_default().push(s);
        }
    }
    let mut buckets = Vec::new();
    let mut last = None;
    for (start_ms, values) in grouped.into_iter().take(query.limit) {
        let min = values.iter().map(|s| s.value).fold(f64::INFINITY, f64::min);
        let max = values
            .iter()
            .map(|s| s.value)
            .fold(f64::NEG_INFINITY, f64::max);
        let mean = values.iter().map(|s| s.value).sum::<f64>() / values.len() as f64;
        let min_quality = values
            .iter()
            .map(|s| s.quality.score)
            .fold(1.0_f32, f32::min);
        let contaminated = values.iter().any(|s| s.quality.contaminated);
        let gap = values.iter().any(|s| s.quality.gap);
        last = values.iter().map(|s| s.sequence).max();
        buckets.push(Bucket {
            start_ms,
            min,
            max,
            mean,
            count: values.len() as u64,
            min_quality,
            contaminated,
            gap,
        });
    }
    Ok(HistoryPage {
        next_cursor: last,
        buckets,
    })
}

/// Immutable replay event.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayEvent {
    pub event_time_ms: u64,
    pub track: String,
    pub value: f64,
}
/// Verified checkpoint for deterministic seeking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub event_time_ms: u64,
    pub digest: String,
}
/// Playback state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Playback {
    Playing,
    Paused,
}
/// Replay-only command sink. No production-write API is exposed.
pub trait ReplayProjection {
    fn reset(&mut self);
    fn apply(&mut self, event: &ReplayEvent);
}

/// Isolated deterministic replay session.
pub struct ReplaySession {
    artifact_digest: String,
    configuration_digest: String,
    model_digest: String,
    events: Vec<ReplayEvent>,
    checkpoints: Vec<Checkpoint>,
    cursor: usize,
    virtual_time_ms: u64,
    state: Playback,
    speed_milli: u32,
    bookmarks: BTreeSet<u64>,
    tracks: BTreeSet<String>,
}

impl ReplaySession {
    /// Creates a session bound to immutable digests and sorted events.
    pub fn new(
        artifact: String,
        configuration: String,
        model: String,
        events: Vec<ReplayEvent>,
        checkpoints: Vec<Checkpoint>,
    ) -> Result<Self, Error> {
        if artifact.is_empty()
            || configuration.is_empty()
            || model.is_empty()
            || events
                .iter()
                .any(|e| !e.value.is_finite() || e.track.is_empty())
            || events
                .windows(2)
                .any(|w| w[0].event_time_ms > w[1].event_time_ms)
            || checkpoints.iter().any(|c| c.digest.is_empty())
        {
            return Err(Error::InvalidReplay);
        }
        let tracks = events.iter().map(|e| e.track.clone()).collect();
        Ok(Self {
            artifact_digest: artifact,
            configuration_digest: configuration,
            model_digest: model,
            events,
            checkpoints,
            cursor: 0,
            virtual_time_ms: 0,
            state: Playback::Paused,
            speed_milli: 1000,
            bookmarks: BTreeSet::new(),
            tracks,
        })
    }
    /// Immutable identity binding.
    pub fn binding(&self) -> (&str, &str, &str) {
        (
            &self.artifact_digest,
            &self.configuration_digest,
            &self.model_digest,
        )
    }
    /// Starts playback.
    pub fn play(&mut self) {
        self.state = Playback::Playing;
    }
    /// Pauses playback.
    pub fn pause(&mut self) {
        self.state = Playback::Paused;
    }
    /// Sets bounded speed from 0.1x through 16x, represented in milli-x.
    pub fn set_speed(&mut self, speed_milli: u32) -> Result<(), Error> {
        if !(100..=16_000).contains(&speed_milli) {
            return Err(Error::Bounds);
        }
        self.speed_milli = speed_milli;
        Ok(())
    }
    /// Adds a bookmark.
    pub fn bookmark(&mut self) {
        self.bookmarks.insert(self.virtual_time_ms);
    }
    /// Returns synchronized track names.
    pub fn tracks(&self) -> &BTreeSet<String> {
        &self.tracks
    }
    /// Advances one event and applies it to an isolated projection.
    pub fn step<P: ReplayProjection>(&mut self, projection: &mut P) -> Option<&ReplayEvent> {
        let event = self.events.get(self.cursor)?;
        projection.apply(event);
        self.virtual_time_ms = event.event_time_ms;
        self.cursor += 1;
        Some(event)
    }
    /// Seeks deterministically by resetting and reducing events. A corrupt checkpoint fails closed.
    pub fn seek<P: ReplayProjection>(
        &mut self,
        target_ms: u64,
        projection: &mut P,
    ) -> Result<(), Error> {
        if self
            .checkpoints
            .iter()
            .any(|c| c.event_time_ms <= target_ms && c.digest.len() < 16)
        {
            return Err(Error::CheckpointCorrupt);
        }
        projection.reset();
        self.cursor = 0;
        self.virtual_time_ms = target_ms;
        while self.cursor < self.events.len() && self.events[self.cursor].event_time_ms <= target_ms
        {
            let event = &self.events[self.cursor];
            projection.apply(event);
            self.cursor += 1;
        }
        Ok(())
    }
    /// Deterministically resets the session.
    pub fn reset<P: ReplayProjection>(&mut self, projection: &mut P) {
        projection.reset();
        self.cursor = 0;
        self.virtual_time_ms = 0;
        self.state = Playback::Paused;
    }
    /// Current virtual event time.
    pub fn virtual_time_ms(&self) -> u64 {
        self.virtual_time_ms
    }
}

/// Fail-closed boundary errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Bounds,
    Forbidden,
    InvalidSample,
    InvalidReplay,
    CheckpointCorrupt,
}
