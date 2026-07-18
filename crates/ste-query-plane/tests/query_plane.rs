#![allow(missing_docs)]

use proptest::prelude::*;
use std::collections::BTreeSet;
use ste_query_plane::*;

fn sample(seq: u64, time: u64, value: f64, series: SeriesKind) -> Sample {
    Sample {
        schema_version: SCHEMA_VERSION,
        source: Source::History,
        stream_id: "s".into(),
        series,
        sequence: seq,
        event_time_ms: time,
        emitted_time_ms: time + 1,
        unit: "ratio".into(),
        algorithm_version: "a1".into(),
        configuration_version: "c1".into(),
        provenance: "fixture".into(),
        scope: Scope::Aggregate,
        retention_class: "short".into(),
        quality: Quality {
            score: 0.9,
            contaminated: seq == 2,
            gap: seq == 3,
            stale: false,
        },
        value,
    }
}
fn aggregate() -> Authorization {
    Authorization {
        scope: Scope::Aggregate,
        now_ms: 10,
        diagnostic: None,
    }
}

#[test]
fn slow_live_consumer_gets_explicit_gap() {
    let mut ring = LiveRing::new(2).unwrap();
    for i in 1..=3 {
        ring.push(sample(i, i, i as f64, SeriesKind::CaptureHealth))
            .unwrap();
    }
    let snapshot = ring.resume(0, 2, &aggregate()).unwrap();
    assert_eq!(snapshot.gap.unwrap().dropped, 1);
    assert_eq!(snapshot.samples[0].sequence, 2);
}

#[test]
fn diagnostic_access_requires_narrow_active_lease() {
    let mut ring = LiveRing::new(2).unwrap();
    ring.push(sample(1, 1, 1.0, SeriesKind::CsiDiagnostic))
        .unwrap();
    assert_eq!(
        ring.resume(0, 1, &aggregate()).unwrap_err(),
        Error::Forbidden
    );
    let auth = Authorization {
        scope: Scope::Diagnostic,
        now_ms: 9,
        diagnostic: Some(DiagnosticLease {
            purpose: "commission".into(),
            expires_at_ms: 10,
            series: BTreeSet::from([SeriesKind::CsiDiagnostic]),
        }),
    };
    assert_eq!(ring.resume(0, 1, &auth).unwrap().samples.len(), 1);
}

#[test]
fn downsampling_preserves_extrema_quality_and_gap() {
    let data = vec![
        sample(1, 10, -7.0, SeriesKind::RuntimeMetrics),
        sample(2, 11, 20.0, SeriesKind::RuntimeMetrics),
        sample(3, 12, 2.0, SeriesKind::RuntimeMetrics),
    ];
    let q = HistoryQuery {
        series: SeriesKind::RuntimeMetrics,
        start_ms: 0,
        end_ms: 100,
        limit: 10,
        bucket_ms: 100,
        cursor_after: None,
        min_quality_milli: 0,
    };
    let b = &query_history(&data, &q, &aggregate()).unwrap().buckets[0];
    assert_eq!((b.min, b.max, b.count), (-7.0, 20.0, 3));
    assert!(b.contaminated && b.gap);
}

#[derive(Default)]
struct Projection(Vec<(String, f64)>);
impl ReplayProjection for Projection {
    fn reset(&mut self) {
        self.0.clear();
    }
    fn apply(&mut self, e: &ReplayEvent) {
        self.0.push((e.track.clone(), e.value));
    }
}

#[test]
fn seek_reset_step_are_deterministic_and_multitrack() {
    let events = vec![
        ReplayEvent {
            event_time_ms: 1,
            track: "a".into(),
            value: 1.0,
        },
        ReplayEvent {
            event_time_ms: 2,
            track: "b".into(),
            value: 2.0,
        },
    ];
    let mut replay = ReplaySession::new(
        "artifact".into(),
        "config".into(),
        "model".into(),
        events,
        vec![Checkpoint {
            event_time_ms: 0,
            digest: "0123456789abcdef".into(),
        }],
    )
    .unwrap();
    let mut p = Projection::default();
    replay.seek(2, &mut p).unwrap();
    let once = p.0.clone();
    replay.seek(2, &mut p).unwrap();
    assert_eq!(p.0, once);
    assert_eq!(replay.tracks().len(), 2);
    replay.reset(&mut p);
    assert!(p.0.is_empty());
}

#[test]
fn corrupt_checkpoint_and_unbounded_queries_fail_closed() {
    let mut r = ReplaySession::new(
        "a".into(),
        "c".into(),
        "m".into(),
        vec![],
        vec![Checkpoint {
            event_time_ms: 0,
            digest: "bad".into(),
        }],
    )
    .unwrap();
    assert_eq!(
        r.seek(1, &mut Projection::default()),
        Err(Error::CheckpointCorrupt)
    );
    let q = HistoryQuery {
        series: SeriesKind::RuntimeMetrics,
        start_ms: 0,
        end_ms: MAX_RANGE_MS + 1,
        limit: 1,
        bucket_ms: 1,
        cursor_after: None,
        min_quality_milli: 0,
    };
    assert_eq!(query_history(&[], &q, &aggregate()), Err(Error::Bounds));
}

#[test]
fn invalid_nan_sample_is_rejected() {
    assert_eq!(
        sample(1, 1, f64::NAN, SeriesKind::RuntimeMetrics).validate(),
        Err(Error::InvalidSample)
    );
}

#[test]
fn speed_is_bounded() {
    let mut r = ReplaySession::new("a".into(), "c".into(), "m".into(), vec![], vec![]).unwrap();
    assert_eq!(r.set_speed(99), Err(Error::Bounds));
    assert!(r.set_speed(16_000).is_ok());
}

proptest! {
    #[test]
    fn aggregation_never_loses_extrema(values in proptest::collection::vec(-1.0e6f64..1.0e6, 1..128)) {
        let data: Vec<_> = values.iter().enumerate().map(|(i, value)| sample(i as u64 + 1, i as u64 + 1, *value, SeriesKind::RuntimeMetrics)).collect();
        let q = HistoryQuery { series: SeriesKind::RuntimeMetrics, start_ms: 0, end_ms: 1000, limit: 1, bucket_ms: 1000, cursor_after: None, min_quality_milli: 0 };
        let result = query_history(&data, &q, &aggregate()).unwrap();
        let bucket = &result.buckets[0];
        prop_assert_eq!(bucket.min, values.iter().copied().fold(f64::INFINITY, f64::min));
        prop_assert_eq!(bucket.max, values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
        prop_assert!(result.buckets.len() <= q.limit);
    }
}
