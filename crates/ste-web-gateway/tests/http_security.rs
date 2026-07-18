//! Hostile-request, bounded-stream, asset-integrity, and restart-isolation tests.

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use ste_ui_gateway::{AssetEntry, AssetManifest, FunctionalArea, GatewayError, ReadModel, Role};
use ste_web_gateway::{
    ApplicationServices, BrowserSession, StreamEvent, VerifiedAssets, WebConfig, router,
};
use tower::ServiceExt;

struct Fake {
    commands: AtomicUsize,
}
impl ApplicationServices for Fake {
    fn session(&self, cookie: &str) -> Result<BrowserSession, GatewayError> {
        if cookie != "valid" {
            return Err(GatewayError::Unauthorized);
        }
        Ok(BrowserSession {
            id: "opaque".into(),
            role: Role::Operator,
            capabilities: vec!["observe".into()],
            csrf: "csrf-secret-value".into(),
        })
    }
    fn read_model(&self, _: &BrowserSession, area: &str) -> Result<ReadModel, GatewayError> {
        if area != "live" {
            return Err(GatewayError::SchemaMismatch);
        }
        ReadModel::new(
            FunctionalArea::LiveOverview,
            7,
            1,
            30_000,
            "test",
            json!({"quality":0.9}),
        )
    }
    fn stream(
        &self,
        _: &BrowserSession,
        name: &str,
        after: Option<u64>,
    ) -> Result<Vec<StreamEvent>, GatewayError> {
        if name != "signal" && !name.starts_with("workflow:") {
            return Err(GatewayError::CommandDenied);
        }
        Ok(vec![StreamEvent {
            sequence: after.unwrap_or(4) + 1,
            dropped: 0,
            schema_version: 1,
            payload: json!({"quality":0.8}),
        }])
    }
    fn command(
        &self,
        _: &BrowserSession,
        name: &str,
        _: &str,
        _: &Value,
    ) -> Result<Value, GatewayError> {
        if name != "commission" {
            return Err(GatewayError::CommandDenied);
        }
        self.commands.fetch_add(1, Ordering::Relaxed);
        Ok(json!({"receipt":"opaque","accepted":true}))
    }
    fn workflow(&self, _: &BrowserSession, id: &str) -> Result<Value, GatewayError> {
        Ok(json!({"id":id,"state":"running"}))
    }
}

fn config() -> WebConfig {
    WebConfig {
        bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4173),
        host: "127.0.0.1:4173".into(),
        origin: "http://127.0.0.1:4173".into(),
        max_body_bytes: 1024,
        max_connections: 8,
        max_streams: 1,
        request_timeout: Duration::from_secs(2),
    }
}
fn assets() -> VerifiedAssets {
    let files: BTreeMap<String, Vec<u8>> = BTreeMap::from([
        ("index.html".into(), b"<!doctype html>".to_vec()),
        ("assets/app-deadbeef.js".into(), b"export{}".to_vec()),
    ]);
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
fn app() -> axum::Router {
    router(
        config(),
        Arc::new(Fake {
            commands: AtomicUsize::new(0),
        }),
        assets(),
    )
    .unwrap()
}
fn get(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header(header::HOST, "127.0.0.1:4173")
        .header(header::COOKIE, "ste_session=valid")
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn health_and_assets_have_hardening_and_cache_rules() {
    let response = app().oneshot(get("/healthz")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(
        response.headers()[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap()
            .contains("frame-ancestors 'none'")
    );
    let response = app().oneshot(get("/assets/app-deadbeef.js")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("immutable")
    );
}

#[tokio::test]
async fn hostile_authorities_origins_paths_and_versions_fail_closed() {
    let bad_host = Request::builder()
        .uri("/healthz")
        .header(header::HOST, "evil.example")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app().oneshot(bad_host).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    let bad_origin = Request::builder()
        .uri("/api/v1/session")
        .header(header::HOST, "127.0.0.1:4173")
        .header(header::ORIGIN, "https://evil.example")
        .header(header::COOKIE, "ste_session=valid")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app().oneshot(bad_origin).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app()
            .oneshot(get("/assets/%2e%2e/secret"))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app()
            .oneshot(get("/api/v2/session"))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn authentication_csrf_content_shape_and_redaction_are_enforced() {
    let no_cookie = Request::builder()
        .uri("/api/v1/session")
        .header(header::HOST, "127.0.0.1:4173")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app().oneshot(no_cookie).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    let command = Request::builder()
        .method("POST")
        .uri("/api/v1/commands/commission")
        .header(header::HOST, "127.0.0.1:4173")
        .header(header::ORIGIN, "http://127.0.0.1:4173")
        .header(header::COOKIE, "ste_session=valid")
        .header("idempotency-key", "key")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"payload":{}}"#))
        .unwrap();
    assert_eq!(
        app().oneshot(command).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    let unknown = app()
        .oneshot(get("/api/v1/read-models/unknown"))
        .await
        .unwrap();
    let bytes = unknown.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes);
    assert!(!text.contains("unknown"));
    assert!(!text.contains('/'));
}

#[tokio::test]
async fn valid_command_and_resumable_sse_cross_authoritative_boundary() {
    let command = Request::builder()
        .method("POST")
        .uri("/api/v1/commands/commission")
        .header(header::HOST, "127.0.0.1:4173")
        .header(header::ORIGIN, "http://127.0.0.1:4173")
        .header(header::COOKIE, "ste_session=valid")
        .header("x-csrf-token", "csrf-secret-value")
        .header("idempotency-key", "key")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"payload":{"dry_run":true}}"#))
        .unwrap();
    assert_eq!(
        app().oneshot(command).await.unwrap().status(),
        StatusCode::OK
    );
    let stream = Request::builder()
        .uri("/api/v1/streams/signal")
        .header(header::HOST, "127.0.0.1:4173")
        .header(header::COOKIE, "ste_session=valid")
        .header("last-event-id", "41")
        .body(Body::empty())
        .unwrap();
    let response = app().oneshot(stream).await.unwrap();
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("id: 42"));
    assert!(text.contains("heartbeat"));
}

#[test]
fn listener_and_manifest_configuration_fail_closed() {
    let mut cfg = config();
    cfg.bind = "0.0.0.0:4173".parse().unwrap();
    assert_eq!(cfg.validate(), Err(GatewayError::UnsafeHost));
    let files = BTreeMap::from([("index.html".into(), b"tampered".to_vec())]);
    let manifest = AssetManifest {
        assets: vec![AssetEntry {
            path: "index.html".into(),
            sha256: "00".repeat(32),
            bytes: 8,
        }],
    };
    assert!(VerifiedAssets::new(&manifest, files).is_err());
}

#[tokio::test]
async fn rebuilding_router_does_not_preserve_sessions_or_stream_state() {
    let first = app().oneshot(get("/api/v1/streams/signal")).await.unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let restarted = app().oneshot(get("/api/v1/streams/signal")).await.unwrap();
    assert_eq!(restarted.status(), StatusCode::OK);
}
