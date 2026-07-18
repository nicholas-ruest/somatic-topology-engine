#![forbid(unsafe_code)]
//! Production HTTP/SSE transport for the policy-safe UI gateway.

use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{Path, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Duration};
use ste_query_plane::{HistoryPage, HistoryQuery};
use ste_ui_gateway::{AssetManifest, GatewayError, ReadModel, Role};
use tokio::sync::Semaphore;
use tower::{ServiceBuilder, limit::ConcurrencyLimitLayer};
use tower_http::{
    catch_panic::CatchPanicLayer, limit::RequestBodyLimitLayer, timeout::TimeoutLayer,
};

mod composition;
pub use composition::{ProductionServices, SessionRecord};

/// Stable browser error envelope. Details and inputs are never reflected.
#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    /// Machine-readable, non-sensitive category.
    pub code: &'static str,
    /// Correlation identifier generated outside domain data.
    pub correlation_id: String,
}

/// Authenticated session projection returned to the browser.
#[derive(Clone, Debug, Serialize)]
pub struct BrowserSession {
    /// Opaque session identifier.
    pub id: String,
    /// Server-authoritative role.
    pub role: Role,
    /// Allowlisted capabilities.
    pub capabilities: Vec<String>,
    /// CSRF token bound to this browser session.
    pub csrf: String,
}

/// A policy-filtered SSE event with observable ordering and loss.
#[derive(Clone, Debug, Serialize)]
pub struct StreamEvent {
    /// Strictly increasing stream sequence.
    pub sequence: u64,
    /// Number of source events omitted before this event.
    pub dropped: u64,
    /// Event contract schema version.
    pub schema_version: u16,
    /// Redacted payload.
    pub payload: Value,
}

/// Authoritative application boundary used by HTTP handlers.
pub trait ApplicationServices: Send + Sync + 'static {
    /// Resolves and authorizes the current browser session.
    fn session(&self, cookie: &str) -> Result<BrowserSession, GatewayError>;
    /// Returns a policy-filtered read projection.
    fn read_model(&self, session: &BrowserSession, area: &str) -> Result<ReadModel, GatewayError>;
    /// Returns a bounded stream snapshot after an optional cursor.
    fn stream(
        &self,
        session: &BrowserSession,
        name: &str,
        after: Option<u64>,
    ) -> Result<Vec<StreamEvent>, GatewayError>;
    /// Executes a typed, bounded, read-only history query.
    fn query(
        &self,
        _session: &BrowserSession,
        _query: &HistoryQuery,
    ) -> Result<HistoryPage, GatewayError> {
        Err(GatewayError::QueryForbidden)
    }
    /// Executes an allowlisted application command.
    fn command(
        &self,
        session: &BrowserSession,
        name: &str,
        idempotency_key: &str,
        payload: &Value,
    ) -> Result<Value, GatewayError>;
    /// Queries a durable workflow projection.
    fn workflow(&self, session: &BrowserSession, id: &str) -> Result<Value, GatewayError>;
}

/// Immutable, already-verified production assets.
#[derive(Clone)]
pub struct VerifiedAssets {
    files: Arc<BTreeMap<String, Bytes>>,
}

impl VerifiedAssets {
    /// Constructs an asset provider only if every file matches the signed manifest.
    pub fn new(
        manifest: &AssetManifest,
        mut files: BTreeMap<String, Vec<u8>>,
    ) -> Result<Self, GatewayError> {
        manifest.verify(&files)?;
        let manifest_bytes =
            serde_json::to_vec(manifest).map_err(|_| GatewayError::AssetMismatch)?;
        files.insert("asset-manifest.json".to_owned(), manifest_bytes);
        Ok(Self {
            files: Arc::new(
                files
                    .into_iter()
                    .map(|(k, v)| (k, Bytes::from(v)))
                    .collect(),
            ),
        })
    }

    fn get(&self, path: &str) -> Option<Bytes> {
        self.files.get(path).cloned()
    }
}

/// Validated transport limits and exact browser authority.
#[derive(Clone, Debug)]
pub struct WebConfig {
    /// Exact loopback socket.
    pub bind: SocketAddr,
    /// Exact Host header, including port.
    pub host: String,
    /// Exact browser origin.
    pub origin: String,
    /// Maximum request bytes.
    pub max_body_bytes: usize,
    /// Maximum concurrent requests.
    pub max_connections: usize,
    /// Maximum concurrent streams.
    pub max_streams: usize,
    /// Handler deadline.
    pub request_timeout: Duration,
}

impl WebConfig {
    /// Rejects wildcard/LAN listeners and unbounded or ambiguous limits.
    pub fn validate(&self) -> Result<(), GatewayError> {
        if !self.bind.ip().is_loopback()
            || self.bind.port() == 0
            || self.host != self.bind.to_string()
            || !(self.origin == format!("http://{}", self.host)
                || self.origin == format!("https://{}", self.host))
            || self.max_body_bytes == 0
            || self.max_body_bytes > 1_048_576
            || self.max_connections == 0
            || self.max_connections > 1024
            || self.max_streams == 0
            || self.max_streams > 128
            || self.request_timeout.is_zero()
            || self.request_timeout > Duration::from_secs(30)
        {
            Err(GatewayError::UnsafeHost)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
struct AppState {
    services: Arc<dyn ApplicationServices>,
    assets: VerifiedAssets,
    config: WebConfig,
    streams: Arc<Semaphore>,
}

/// Builds the exact, closed route set and all resource/security middleware.
pub fn router(
    config: WebConfig,
    services: Arc<dyn ApplicationServices>,
    assets: VerifiedAssets,
) -> Result<Router, GatewayError> {
    config.validate()?;
    let state = AppState {
        services,
        assets,
        streams: Arc::new(Semaphore::new(config.max_streams)),
        config: config.clone(),
    };
    let middleware = ServiceBuilder::new()
        .layer(CatchPanicLayer::new())
        .layer(RequestBodyLimitLayer::new(config.max_body_bytes))
        .layer(ConcurrencyLimitLayer::new(config.max_connections))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            config.request_timeout,
        ));
    Ok(Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/session", get(session))
        .route("/api/v1/read-models/{area}", get(read_model))
        .route("/api/v1/streams/{stream}", get(events))
        .route("/api/v1/query", post(query))
        .route("/api/v1/commands/{command}", post(command))
        .route("/api/v1/workflows/{id}", get(workflow))
        .route("/api/v1/workflows/{id}/stream", get(workflow_events))
        .route("/asset-manifest.json", get(asset_manifest))
        .route("/", get(index))
        .route("/assets/{*path}", get(asset))
        .fallback(not_found)
        .layer(middleware)
        .with_state(state))
}

/// Binds and serves only after validating the exact loopback address.
pub async fn serve(config: WebConfig, app: Router) -> Result<(), std::io::Error> {
    config.validate().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, "unsafe gateway bind")
    })?;
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    axum::serve(listener, app).await
}

async fn health(State(state): State<AppState>, request: Request) -> Response {
    if let Err(r) = authority(&state, &request, false) {
        return r;
    }
    secure(
        StatusCode::OK,
        Json(json!({"status":"ready"})).into_response(),
        false,
    )
}

async fn session(State(state): State<AppState>, request: Request) -> Response {
    let session = match authenticate(&state, &request, false) {
        Ok(v) => v,
        Err(r) => return r,
    };
    secure(StatusCode::OK, Json(session).into_response(), false)
}

async fn read_model(
    State(state): State<AppState>,
    Path(area): Path<String>,
    request: Request,
) -> Response {
    let session = match authenticate(&state, &request, false) {
        Ok(v) => v,
        Err(r) => return r,
    };
    result(state.services.read_model(&session, &area), &request)
}

#[derive(Deserialize)]
struct CommandBody {
    payload: Value,
}

async fn command(
    State(state): State<AppState>,
    Path(name): Path<String>,
    headers: HeaderMap,
    request: Request,
) -> Response {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("application/json")
    {
        return rejection(StatusCode::UNSUPPORTED_MEDIA_TYPE, "content_type", &request);
    }
    let session = match authenticate(&state, &request, true) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let key = match headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        Some(v) if !v.is_empty() && v.len() <= 128 => v,
        _ => return rejection(StatusCode::BAD_REQUEST, "invalid_request", &request),
    };
    let bytes = match axum::body::to_bytes(request.into_body(), state.config.max_body_bytes).await {
        Ok(v) => v,
        Err(_) => return bare_rejection(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"),
    };
    let body: CommandBody = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return bare_rejection(StatusCode::BAD_REQUEST, "invalid_json"),
    };
    if json_complexity(&body.payload, 0, &mut 0) {
        return bare_rejection(StatusCode::BAD_REQUEST, "payload_complexity");
    }
    let value = state.services.command(&session, &name, key, &body.payload);
    result_without_request(value)
}

async fn query(State(state): State<AppState>, headers: HeaderMap, request: Request) -> Response {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        != Some("application/json")
    {
        return rejection(StatusCode::UNSUPPORTED_MEDIA_TYPE, "content_type", &request);
    }
    let session = match authenticate(&state, &request, true) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let bytes = match axum::body::to_bytes(request.into_body(), state.config.max_body_bytes).await {
        Ok(v) => v,
        Err(_) => return bare_rejection(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"),
    };
    let query: HistoryQuery = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return bare_rejection(StatusCode::BAD_REQUEST, "invalid_json"),
    };
    result_without_request(state.services.query(&session, &query))
}

fn json_complexity(value: &Value, depth: usize, fields: &mut usize) -> bool {
    if depth > 12 || *fields > 512 {
        return true;
    }
    match value {
        Value::Object(values) => values.iter().any(|(key, value)| {
            *fields += 1;
            key.len() > 128 || json_complexity(value, depth + 1, fields)
        }),
        Value::Array(values) => values
            .iter()
            .any(|value| json_complexity(value, depth + 1, fields)),
        Value::String(value) => value.len() > 4096,
        _ => false,
    }
}

async fn workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let session = match authenticate(&state, &request, false) {
        Ok(v) => v,
        Err(r) => return r,
    };
    result(state.services.workflow(&session, &id), &request)
}

async fn events(
    State(state): State<AppState>,
    Path(name): Path<String>,
    request: Request,
) -> Response {
    stream_response(state, name, request).await
}
async fn workflow_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    stream_response(state, format!("workflow:{id}"), request).await
}

async fn stream_response(state: AppState, name: String, request: Request) -> Response {
    let session = match authenticate(&state, &request, false) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let permit = match state.streams.clone().try_acquire_owned() {
        Ok(v) => v,
        Err(_) => return rejection(StatusCode::TOO_MANY_REQUESTS, "stream_limit", &request),
    };
    let cursor = request
        .headers()
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok());
    let events = match state.services.stream(&session, &name, cursor) {
        Ok(v) if v.len() <= 1024 => v,
        Ok(_) => {
            return rejection(
                StatusCode::INSUFFICIENT_STORAGE,
                "stream_overflow",
                &request,
            );
        }
        Err(e) => return gateway_rejection(e, &request),
    };
    let chunks = stream::iter(events.into_iter().map(|event| {
        let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_owned());
        Ok::<Bytes, std::convert::Infallible>(Bytes::from(format!(
            "id: {}\nevent: projection\ndata: {json}\n\n",
            event.sequence
        )))
    }))
    .chain(stream::once(async move {
        drop(permit);
        Ok(Bytes::from_static(b": heartbeat\n\n"))
    }));
    let mut response = Body::from_stream(chunks).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    secure(StatusCode::OK, response, true)
}

async fn index(State(state): State<AppState>, request: Request) -> Response {
    static_file(&state, &request, "index.html", false)
}
async fn asset_manifest(State(state): State<AppState>, request: Request) -> Response {
    static_file(&state, &request, "asset-manifest.json", false)
}
async fn asset(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Response {
    if path.contains("..") || path.contains('\\') {
        return rejection(StatusCode::NOT_FOUND, "not_found", &request);
    }
    static_file(&state, &request, &format!("assets/{path}"), true)
}
fn static_file(state: &AppState, request: &Request, path: &str, immutable: bool) -> Response {
    if let Err(r) = authority(state, request, false) {
        return r;
    }
    match state.assets.get(path) {
        Some(body) => {
            let mut response = Body::from(body).into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(content_type(path)),
            );
            secure(StatusCode::OK, response, immutable)
        }
        None => rejection(StatusCode::NOT_FOUND, "not_found", request),
    }
}

async fn not_found(request: Request) -> Response {
    rejection(StatusCode::NOT_FOUND, "not_found", &request)
}

fn authority(state: &AppState, request: &Request, mutating: bool) -> Result<(), Response> {
    if request.uri().path().len() > 2048 || request.headers().len() > 64 {
        return Err(rejection(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            request,
        ));
    }
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok());
    if host != Some(state.config.host.as_str()) {
        return Err(rejection(
            StatusCode::FORBIDDEN,
            "authority_denied",
            request,
        ));
    }
    if let Some(origin) = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
    {
        if origin != state.config.origin {
            return Err(rejection(StatusCode::FORBIDDEN, "origin_denied", request));
        }
    } else if mutating {
        return Err(rejection(StatusCode::FORBIDDEN, "origin_required", request));
    }
    Ok(())
}

fn authenticate(
    state: &AppState,
    request: &Request,
    mutating: bool,
) -> Result<BrowserSession, Response> {
    authority(state, request, mutating)?;
    let cookie = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.split(';')
                .map(str::trim)
                .find_map(|p| p.strip_prefix("ste_session="))
        })
        .ok_or_else(|| rejection(StatusCode::UNAUTHORIZED, "session_required", request))?;
    let session = state
        .services
        .session(cookie)
        .map_err(|e| gateway_rejection(e, request))?;
    if mutating {
        let csrf = request
            .headers()
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok());
        if csrf != Some(session.csrf.as_str()) {
            return Err(rejection(StatusCode::FORBIDDEN, "csrf_denied", request));
        }
    }
    Ok(session)
}

fn result<T: Serialize>(value: Result<T, GatewayError>, request: &Request) -> Response {
    match value {
        Ok(v) => secure(StatusCode::OK, Json(v).into_response(), false),
        Err(e) => gateway_rejection(e, request),
    }
}
fn result_without_request<T: Serialize>(value: Result<T, GatewayError>) -> Response {
    match value {
        Ok(v) => secure(StatusCode::OK, Json(v).into_response(), false),
        Err(e) => bare_gateway_rejection(e),
    }
}
fn gateway_rejection(error: GatewayError, request: &Request) -> Response {
    let (status, code) = map_error(error);
    rejection(status, code, request)
}
fn bare_gateway_rejection(error: GatewayError) -> Response {
    let (status, code) = map_error(error);
    bare_rejection(status, code)
}
fn map_error(error: GatewayError) -> (StatusCode, &'static str) {
    match error {
        GatewayError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
        GatewayError::CommandDenied => (StatusCode::FORBIDDEN, "command_denied"),
        GatewayError::BodyTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"),
        GatewayError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
        GatewayError::IdempotencyConflict => (StatusCode::CONFLICT, "idempotency_conflict"),
        GatewayError::Stale => (StatusCode::SERVICE_UNAVAILABLE, "projection_stale"),
        GatewayError::QueryForbidden => (StatusCode::FORBIDDEN, "query_forbidden"),
        GatewayError::QueryRejected => (StatusCode::BAD_REQUEST, "query_rejected"),
        _ => (StatusCode::BAD_REQUEST, "request_rejected"),
    }
}
fn rejection(status: StatusCode, code: &'static str, request: &Request) -> Response {
    let id = correlation(request.headers());
    error(status, code, id)
}
fn bare_rejection(status: StatusCode, code: &'static str) -> Response {
    error(status, code, "unavailable".to_owned())
}
fn error(status: StatusCode, code: &'static str, id: String) -> Response {
    secure(
        status,
        Json(ErrorEnvelope {
            code,
            correlation_id: id,
        })
        .into_response(),
        false,
    )
}
fn correlation(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .filter(|v| v.len() <= 64 && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
        .unwrap_or("unavailable")
        .to_owned()
}

fn secure(status: StatusCode, mut response: Response, immutable: bool) -> Response {
    *response.status_mut() = status;
    let h = response.headers_mut();
    for (name, value) in [
        (
            header::CONTENT_SECURITY_POLICY,
            "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'",
        ),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::REFERRER_POLICY, "no-referrer"),
        (
            HeaderName::from_static("permissions-policy"),
            "camera=(), microphone=(), geolocation=()",
        ),
        (
            HeaderName::from_static("cross-origin-opener-policy"),
            "same-origin",
        ),
        (
            HeaderName::from_static("cross-origin-resource-policy"),
            "same-origin",
        ),
    ] {
        h.insert(name, HeaderValue::from_static(value));
    }
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if immutable {
            "public, max-age=31536000, immutable"
        } else {
            "no-store"
        }),
    );
    response
}

fn content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    }
}

/// Returns the exact support matrix used by compatibility tests and documentation.
pub const fn supported_routes() -> &'static [(&'static str, &'static str)] {
    &[
        ("GET", "/healthz"),
        ("GET", "/api/v1/session"),
        ("GET", "/api/v1/read-models/{area}"),
        ("GET", "/api/v1/streams/{stream}"),
        ("POST", "/api/v1/commands/{command}"),
        ("GET", "/api/v1/workflows/{id}"),
        ("GET", "/api/v1/workflows/{id}/stream"),
    ]
}
