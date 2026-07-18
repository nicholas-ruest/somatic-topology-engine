//! End-to-end proof over the real bounded query plane and durable workflow engine.

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Duration};
use ste_query_plane::{Quality, Sample, Scope, SeriesKind, Source};
use ste_ui_gateway::{AssetEntry, AssetManifest, Role};
use ste_web_gateway::{
    BrowserSession, ProductionServices, SessionRecord, VerifiedAssets, WebConfig, router,
};
use ste_workflows::{Authorization, AuthorizationDecision, InMemoryJournal, WorkflowRequest};
use tower::ServiceExt;

struct Allow;
impl Authorization for Allow {
    fn reauthorize(&self, _: &WorkflowRequest, _: u64) -> AuthorizationDecision {
        AuthorizationDecision::Allow
    }
}

fn assets() -> VerifiedAssets {
    let files: BTreeMap<String, Vec<u8>> = BTreeMap::from([("index.html".into(), b"ok".to_vec())]);
    let manifest = AssetManifest {
        assets: files
            .iter()
            .map(|(path, body)| AssetEntry {
                path: path.clone(),
                sha256: format!("{:x}", Sha256::digest(body)),
                bytes: body.len() as u64,
            })
            .collect(),
    };
    VerifiedAssets::new(&manifest, files).unwrap()
}

fn request(method: &str, path: &str, body: Body) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, "127.0.0.1:4173")
        .header(
            header::COOKIE,
            "ste_session=abcdefghijklmnopqrstuvwxyz012345",
        );
    if method == "POST" {
        request = request
            .header(header::ORIGIN, "http://127.0.0.1:4173")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-csrf-token", "server-csrf-token")
            .header("idempotency-key", format!("key-{path}"));
    }
    request.body(body).unwrap()
}

#[tokio::test]
async fn authenticated_query_and_durable_workflow_share_real_composition() {
    let services = Arc::new(ProductionServices::new(8, InMemoryJournal::default(), Allow).unwrap());
    services
        .register_session(
            "abcdefghijklmnopqrstuvwxyz012345".into(),
            SessionRecord {
                session: BrowserSession {
                    id: "session-opaque".into(),
                    role: Role::Operator,
                    capabilities: vec!["workflow.create".into()],
                    csrf: "server-csrf-token".into(),
                },
                expires_at_ms: u64::MAX,
                query_scope: Scope::Aggregate,
                authorization_ref: "policy:v1".into(),
                purpose_ref: "operations".into(),
            },
        )
        .unwrap();
    services
        .publish(Sample {
            schema_version: 1,
            source: Source::Live,
            stream_id: "approved-live".into(),
            series: SeriesKind::CaptureHealth,
            sequence: 1,
            event_time_ms: 1,
            emitted_time_ms: 1,
            unit: "ratio".into(),
            algorithm_version: "v1".into(),
            configuration_version: "v1".into(),
            provenance: "runtime".into(),
            scope: Scope::Aggregate,
            retention_class: "transient".into(),
            quality: Quality {
                score: 0.9,
                contaminated: false,
                gap: false,
                stale: false,
            },
            value: 0.9,
        })
        .unwrap();
    let config = WebConfig {
        bind: SocketAddr::from(([127, 0, 0, 1], 4173)),
        host: "127.0.0.1:4173".into(),
        origin: "http://127.0.0.1:4173".into(),
        max_body_bytes: 4096,
        max_connections: 8,
        max_streams: 2,
        request_timeout: Duration::from_secs(2),
    };
    let app = router(config, services, assets()).unwrap();

    let live = app
        .clone()
        .oneshot(request("GET", "/api/v1/streams/live", Body::empty()))
        .await
        .unwrap();
    assert_eq!(live.status(), StatusCode::OK);
    let live_body = live.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&live_body).contains("capture_health"));

    let history = json!({"series":"capture_health","start_ms":0,"end_ms":2,"limit":8,"bucket_ms":1,"cursor_after":null,"min_quality_milli":800});
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/v1/query",
            Body::from(history.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let history: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(history["buckets"][0]["count"], 1);

    let history = json!({"series":"capture_health","start_ms":0,"end_ms":10,"limit":10,"bucket_ms":10,"cursor_after":null,"min_quality_milli":0});
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/v1/query",
            Body::from(history.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let page: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(page["buckets"][0]["max"], 0.9);

    let unbounded = json!({"series":"capture_health","start_ms":0,"end_ms":10,"limit":10001,"bucket_ms":10,"cursor_after":null,"min_quality_milli":0});
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/v1/query",
            Body::from(unbounded.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let diagnostic = json!({"series":"csi_diagnostic","start_ms":0,"end_ms":10,"limit":10,"bucket_ms":10,"cursor_after":null,"min_quality_milli":0});
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/v1/query",
            Body::from(diagnostic.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let create = json!({"payload":{"workflow_type":"hardware_probe","scope":"device:local","expires_at_ms":u64::MAX,"correlation_id":"corr-1"}});
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/v1/commands/workflow.create",
            Body::from(create.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let projection: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let id = projection["id"].as_str().unwrap();
    assert_eq!(projection["state"], "ready");

    let apply = json!({"payload":{"workflow_id":id,"expected_version":2,"action":"start"}});
    let response = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/v1/commands/workflow.apply",
            Body::from(apply.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let projection: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(projection["state"], "running");

    let response = app
        .oneshot(request(
            "GET",
            &format!("/api/v1/workflows/{id}"),
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let durable: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(durable["version"], 3);
    assert_eq!(durable["state"], "running");
}
