#![allow(missing_docs)]

use ste_query_plane::{
    HistoryQuery, MAX_POINTS, MAX_RANGE_MS, Quality, SCHEMA_VERSION, Sample, Scope, SeriesKind,
    Source,
};
use ste_ui_gateway::*;

fn policy() -> BrowserSecurityPolicy {
    BrowserSecurityPolicy {
        allowed_origin: "http://127.0.0.1:8080".into(),
        csp: "default-src 'self'; object-src 'none'; frame-ancestors 'none'".into(),
        session_ttl_ms: 10_000,
        max_body_bytes: 2048,
        requests_per_minute: 10,
    }
}
fn session(role: Role) -> Session {
    Session {
        id: "s".into(),
        role,
        device_binding: "device".into(),
        csrf: "csrf-token-123456".into(),
        expires_at_ms: 1000,
    }
}
fn sample(series: SeriesKind) -> Sample {
    Sample {
        schema_version: SCHEMA_VERSION,
        source: Source::History,
        stream_id: "approved".into(),
        series,
        sequence: 1,
        event_time_ms: 10,
        emitted_time_ms: 11,
        unit: "ratio".into(),
        algorithm_version: "1".into(),
        configuration_version: "1".into(),
        provenance: "projection".into(),
        scope: Scope::Aggregate,
        retention_class: "short".into(),
        quality: Quality {
            score: 1.0,
            contaminated: false,
            gap: false,
            stale: false,
        },
        value: 2.0,
    }
}
fn query(series: SeriesKind) -> HistoryQuery {
    HistoryQuery {
        series,
        start_ms: 0,
        end_ms: 100,
        limit: 10,
        bucket_ms: 10,
        cursor_after: None,
        min_quality_milli: 0,
    }
}
fn request(query: &HistoryQuery) -> QueryRequest<'_> {
    QueryRequest {
        now_ms: 10,
        origin: "http://127.0.0.1:8080",
        device_binding: "device",
        csrf: "csrf-token-123456",
        body_bytes: 100,
        query,
    }
}

#[test]
fn api_query_contract_returns_only_bounded_projection_page() {
    let mut service =
        QueryService::new(policy(), vec![sample(SeriesKind::RuntimeMetrics)], None, 0).unwrap();
    let value = service
        .query(
            &session(Role::Operator),
            request(&query(SeriesKind::RuntimeMetrics)),
        )
        .unwrap();
    assert_eq!(value["buckets"][0]["max"], 2.0);
    assert!(value.get("next_cursor").is_some());
}

#[test]
fn hostile_origin_csrf_device_and_expired_session_fail_before_query() {
    let q = query(SeriesKind::RuntimeMetrics);
    for (origin, device, csrf, now) in [
        ("http://evil.invalid", "device", "csrf-token-123456", 10),
        ("http://127.0.0.1:8080", "other", "csrf-token-123456", 10),
        ("http://127.0.0.1:8080", "device", "wrong", 10),
        ("http://127.0.0.1:8080", "device", "csrf-token-123456", 1001),
    ] {
        let mut service = QueryService::new(policy(), vec![], None, 0).unwrap();
        let req = QueryRequest {
            now_ms: now,
            origin,
            device_binding: device,
            csrf,
            body_bytes: 100,
            query: &q,
        };
        assert_eq!(
            service.query(&session(Role::Operator), req),
            Err(GatewayError::Unauthorized)
        );
    }
}

#[test]
fn query_bounds_and_body_limits_fail_closed() {
    let mut service = QueryService::new(policy(), vec![], None, 0).unwrap();
    let mut q = query(SeriesKind::RuntimeMetrics);
    q.limit = MAX_POINTS + 1;
    assert_eq!(
        service.query(&session(Role::Operator), request(&q)),
        Err(GatewayError::QueryRejected)
    );
    q.limit = 1;
    q.end_ms = MAX_RANGE_MS + 1;
    assert_eq!(
        service.query(&session(Role::Operator), request(&q)),
        Err(GatewayError::QueryRejected)
    );
    let req = QueryRequest {
        body_bytes: 2049,
        ..request(&q)
    };
    assert_eq!(
        service.query(&session(Role::Operator), req),
        Err(GatewayError::BodyTooLarge)
    );
}

#[test]
fn csi_diagnostics_cannot_be_obtained_by_ordinary_browser_roles() {
    let mut service =
        QueryService::new(policy(), vec![sample(SeriesKind::CsiDiagnostic)], None, 0).unwrap();
    for role in [
        Role::Participant,
        Role::Operator,
        Role::Support,
        Role::Validation,
        Role::Security,
        Role::Release,
    ] {
        assert_eq!(
            service.query(&session(role), request(&query(SeriesKind::CsiDiagnostic))),
            Err(GatewayError::QueryForbidden)
        );
    }
}

#[test]
fn oversized_or_invalid_projection_set_is_rejected_at_construction() {
    let invalid = Sample {
        value: f64::NAN,
        ..sample(SeriesKind::RuntimeMetrics)
    };
    assert!(matches!(
        QueryService::new(policy(), vec![invalid], None, 0),
        Err(GatewayError::QueryRejected)
    ));
}
