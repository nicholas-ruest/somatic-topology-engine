#![forbid(unsafe_code)]
//! Fail-closed, transport-neutral browser presentation boundary.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use ste_query_plane::{Authorization, DiagnosticLease, HistoryQuery, Sample, Scope, query_history};

/// Browser read-model schema supported by this gateway.
pub const SCHEMA_VERSION: u16 = 1;
const MAX_JSON_DEPTH: usize = 12;
const MAX_FIELDS: usize = 512;
const PROHIBITED_KEYS: &[&str] = &[
    "secret",
    "password",
    "token",
    "raw_csi",
    "participant_id",
    "private_key",
];

/// Every product area named by ADR-057.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionalArea {
    LiveOverview,
    SpatialSignalTopology,
    RadioAcquisition,
    SignalObservation,
    PhysiologyEstimation,
    StateInference,
    PersonalizationMemory,
    DeviceInteraction,
    ConsentGovernance,
    ExperimentValidation,
    ModelsCapabilities,
    ObservabilityReliability,
    Commissioning,
    OperationsDataLifecycle,
    SecurityIncidents,
    ReleaseCommercialReadiness,
}

/// Roles are explicit and never inferred from route visibility.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Participant,
    Operator,
    Support,
    Validation,
    Security,
    Release,
}

/// Policy-filtered wire envelope. Payload has already crossed the redaction boundary.
#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReadModel {
    pub schema_version: u16,
    pub area: FunctionalArea,
    pub sequence: u64,
    pub emitted_at_ms: u64,
    pub stale_after_ms: u64,
    pub provenance: String,
    pub payload: Value,
}

impl ReadModel {
    /// Validates metadata and recursively rejects sensitive or excessively complex payloads.
    pub fn new(
        area: FunctionalArea,
        sequence: u64,
        emitted_at_ms: u64,
        stale_after_ms: u64,
        provenance: impl Into<String>,
        payload: Value,
    ) -> Result<Self, GatewayError> {
        let provenance = provenance.into();
        if sequence == 0
            || stale_after_ms == 0
            || stale_after_ms > 300_000
            || provenance.is_empty()
            || provenance.len() > 128
        {
            return Err(GatewayError::InvalidReadModel);
        }
        let mut fields = 0;
        validate_value(&payload, 0, &mut fields)?;
        Ok(Self {
            schema_version: SCHEMA_VERSION,
            area,
            sequence,
            emitted_at_ms,
            stale_after_ms,
            provenance,
            payload,
        })
    }

    /// Fails closed on incompatible schemas or stale data.
    pub fn validate_at(&self, now_ms: u64) -> Result<(), GatewayError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(GatewayError::SchemaMismatch);
        }
        if now_ms.saturating_sub(self.emitted_at_ms) > self.stale_after_ms {
            return Err(GatewayError::Stale);
        }
        let mut fields = 0;
        validate_value(&self.payload, 0, &mut fields)
    }
}

fn validate_value(value: &Value, depth: usize, fields: &mut usize) -> Result<(), GatewayError> {
    if depth > MAX_JSON_DEPTH {
        return Err(GatewayError::PayloadComplexity);
    }
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                *fields += 1;
                let normalized = key.to_ascii_lowercase();
                if PROHIBITED_KEYS.iter().any(|bad| normalized.contains(bad)) {
                    return Err(GatewayError::ProhibitedField);
                }
                if *fields > MAX_FIELDS {
                    return Err(GatewayError::PayloadComplexity);
                }
                validate_value(item, depth + 1, fields)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                validate_value(item, depth + 1, fields)?;
            }
        }
        Value::String(text) if text.len() > 4096 => return Err(GatewayError::PayloadComplexity),
        _ => {}
    }
    Ok(())
}

/// Browser-facing security configuration, validated before binding a listener.
#[allow(missing_docs)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserSecurityPolicy {
    pub allowed_origin: String,
    pub csp: String,
    pub session_ttl_ms: u64,
    pub max_body_bytes: usize,
    pub requests_per_minute: u32,
}

impl BrowserSecurityPolicy {
    /// Requires an exact HTTPS/loopback origin and a restrictive, local-only CSP.
    pub fn validate(&self) -> Result<(), GatewayError> {
        let origin_ok = self.allowed_origin.starts_with("https://127.0.0.1:")
            || self.allowed_origin.starts_with("https://localhost:")
            || self.allowed_origin.starts_with("http://127.0.0.1:")
            || self.allowed_origin.starts_with("http://localhost:");
        if !origin_ok
            || self.allowed_origin.contains('*')
            || self.session_ttl_ms == 0
            || self.session_ttl_ms > 900_000
            || self.max_body_bytes == 0
            || self.max_body_bytes > 1_048_576
            || self.requests_per_minute == 0
            || self.requests_per_minute > 600
            || !self.csp.contains("default-src 'self'")
            || !self.csp.contains("object-src 'none'")
            || !self.csp.contains("frame-ancestors 'none'")
            || self.csp.contains("unsafe-eval")
        {
            return Err(GatewayError::UnsafePolicy);
        }
        Ok(())
    }
}

/// Short-lived server-side session binding.
#[allow(missing_docs)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
    pub id: String,
    pub role: Role,
    pub device_binding: String,
    pub csrf: String,
    pub expires_at_ms: u64,
}

impl Session {
    /// Authenticates all browser request bindings with constant-shape comparisons.
    pub fn authorize(
        &self,
        now_ms: u64,
        device: &str,
        csrf: &str,
        origin: &str,
        policy: &BrowserSecurityPolicy,
    ) -> Result<(), GatewayError> {
        policy.validate()?;
        if self.id.is_empty()
            || self.csrf.len() < 16
            || self.device_binding.is_empty()
            || now_ms >= self.expires_at_ms
            || !same(&self.device_binding, device)
            || !same(&self.csrf, csrf)
            || origin != policy.allowed_origin
        {
            return Err(GatewayError::Unauthorized);
        }
        Ok(())
    }
}

/// Transport-neutral fixed-window request and body-size enforcement.
#[derive(Clone, Debug)]
pub struct RequestGuards {
    window_started_ms: u64,
    requests: u32,
}

impl RequestGuards {
    /// Creates an empty rate window.
    pub const fn new(now_ms: u64) -> Self {
        Self {
            window_started_ms: now_ms,
            requests: 0,
        }
    }

    /// Rejects an oversized body or a request beyond the configured minute ceiling.
    pub fn admit(
        &mut self,
        now_ms: u64,
        body_bytes: usize,
        policy: &BrowserSecurityPolicy,
    ) -> Result<(), GatewayError> {
        policy.validate()?;
        if body_bytes > policy.max_body_bytes {
            return Err(GatewayError::BodyTooLarge);
        }
        if now_ms.saturating_sub(self.window_started_ms) >= 60_000 {
            self.window_started_ms = now_ms;
            self.requests = 0;
        }
        if self.requests >= policy.requests_per_minute {
            return Err(GatewayError::RateLimited);
        }
        self.requests += 1;
        Ok(())
    }
}

fn same(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.bytes()
        .zip(right.bytes())
        .fold(0_u8, |d, (a, b)| d | (a ^ b))
        == 0
}

/// Versioned command authorization independent of transport and UI affordances.
#[derive(Clone, Debug)]
pub struct CommandBridge {
    allowed: BTreeMap<String, BTreeSet<Role>>,
}
impl CommandBridge {
    /// Creates an immutable command/role allowlist.
    pub fn new(allowed: BTreeMap<String, BTreeSet<Role>>) -> Result<Self, GatewayError> {
        if allowed.is_empty()
            || allowed
                .keys()
                .any(|k| k.is_empty() || !k.starts_with("v1."))
        {
            Err(GatewayError::InvalidAllowlist)
        } else {
            Ok(Self { allowed })
        }
    }
    /// Requires allowlisting, role authorization, and a bounded idempotency key.
    pub fn authorize(
        &self,
        command: &str,
        role: Role,
        idempotency_key: &str,
    ) -> Result<(), GatewayError> {
        if idempotency_key.is_empty() || idempotency_key.len() > 128 {
            return Err(GatewayError::InvalidIdempotencyKey);
        }
        if self
            .allowed
            .get(command)
            .is_none_or(|roles| !roles.contains(&role))
        {
            return Err(GatewayError::CommandDenied);
        }
        Ok(())
    }
}

/// Exact application command dispatcher implemented by the authoritative Rust boundary.
pub trait CommandExecutor {
    /// Executes one already-authorized command and returns a presentation-safe candidate value.
    fn execute(&mut self, command: &str, body: &Value) -> Result<Value, GatewayError>;
}

/// Authenticated browser command request metadata and payload.
#[allow(missing_docs)]
pub struct CommandRequest<'a> {
    pub now_ms: u64,
    pub origin: &'a str,
    pub device_binding: &'a str,
    pub csrf: &'a str,
    pub command: &'a str,
    pub idempotency_key: &'a str,
    pub body_bytes: usize,
    pub body: &'a Value,
}

/// Stable command receipt returned both for initial execution and exact retries.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CommandReceipt {
    /// Opaque idempotency key supplied by the client.
    pub idempotency_key: String,
    /// Exact allowlisted command name.
    pub command: String,
    /// Policy-filtered application result.
    pub response: Value,
    /// True when returned from the idempotency cache rather than dispatched.
    pub replayed: bool,
}

#[derive(Clone, Debug)]
struct StoredReceipt {
    fingerprint: String,
    receipt: CommandReceipt,
}

/// Complete transport-independent authorization and dispatch service.
pub struct GatewayService<E> {
    policy: BrowserSecurityPolicy,
    bridge: CommandBridge,
    guards: RequestGuards,
    executor: E,
    receipts: BTreeMap<(String, String), StoredReceipt>,
}

impl<E: CommandExecutor> GatewayService<E> {
    /// Constructs a service after validating its production security policy.
    pub fn new(
        policy: BrowserSecurityPolicy,
        bridge: CommandBridge,
        executor: E,
        now_ms: u64,
    ) -> Result<Self, GatewayError> {
        policy.validate()?;
        Ok(Self {
            policy,
            bridge,
            guards: RequestGuards::new(now_ms),
            executor,
            receipts: BTreeMap::new(),
        })
    }

    /// Authenticates, bounds, authorizes, deduplicates, dispatches, and filters one command.
    pub fn dispatch(
        &mut self,
        session: &Session,
        request: CommandRequest<'_>,
    ) -> Result<CommandReceipt, GatewayError> {
        session.authorize(
            request.now_ms,
            request.device_binding,
            request.csrf,
            request.origin,
            &self.policy,
        )?;
        self.guards
            .admit(request.now_ms, request.body_bytes, &self.policy)?;
        self.bridge
            .authorize(request.command, session.role, request.idempotency_key)?;
        let fingerprint = command_fingerprint(request.command, request.body)?;
        let cache_key = (session.id.clone(), request.idempotency_key.to_owned());
        if let Some(stored) = self.receipts.get(&cache_key) {
            if !same(&stored.fingerprint, &fingerprint) {
                return Err(GatewayError::IdempotencyConflict);
            }
            let mut receipt = stored.receipt.clone();
            receipt.replayed = true;
            return Ok(receipt);
        }
        let response = redact_value(self.executor.execute(request.command, request.body)?);
        let mut fields = 0;
        validate_value(&response, 0, &mut fields)?;
        let receipt = CommandReceipt {
            idempotency_key: request.idempotency_key.to_owned(),
            command: request.command.to_owned(),
            response,
            replayed: false,
        };
        self.receipts.insert(
            cache_key,
            StoredReceipt {
                fingerprint,
                receipt: receipt.clone(),
            },
        );
        Ok(receipt)
    }

    /// Consumes the gateway and returns its executor, useful for graceful shutdown and inspection.
    pub fn into_executor(self) -> E {
        self.executor
    }
}

/// Authenticated `/api/v1/query` request metadata and typed request body.
#[allow(missing_docs)]
pub struct QueryRequest<'a> {
    pub now_ms: u64,
    pub origin: &'a str,
    pub device_binding: &'a str,
    pub csrf: &'a str,
    pub body_bytes: usize,
    pub query: &'a HistoryQuery,
}

/// Read-only ADR-059 adapter used by the HTTP `POST /api/v1/query` route.
///
/// It deliberately owns only approved projection samples. There is no command,
/// journal, personalization, evidence, or production-write capability.
pub struct QueryService {
    policy: BrowserSecurityPolicy,
    guards: RequestGuards,
    samples: Vec<Sample>,
    diagnostic_lease: Option<DiagnosticLease>,
}

impl QueryService {
    /// Constructs a bounded query adapter after validating gateway policy.
    pub fn new(
        policy: BrowserSecurityPolicy,
        samples: Vec<Sample>,
        diagnostic_lease: Option<DiagnosticLease>,
        now_ms: u64,
    ) -> Result<Self, GatewayError> {
        policy.validate()?;
        if samples.len() > ste_query_plane::MAX_POINTS {
            return Err(GatewayError::QueryRejected);
        }
        for sample in &samples {
            sample.validate().map_err(|_| GatewayError::QueryRejected)?;
        }
        Ok(Self {
            policy,
            guards: RequestGuards::new(now_ms),
            samples,
            diagnostic_lease,
        })
    }

    /// Authenticates and executes one typed, bounded, read-only query.
    pub fn query(
        &mut self,
        session: &Session,
        request: QueryRequest<'_>,
    ) -> Result<Value, GatewayError> {
        session.authorize(
            request.now_ms,
            request.device_binding,
            request.csrf,
            request.origin,
            &self.policy,
        )?;
        self.guards
            .admit(request.now_ms, request.body_bytes, &self.policy)?;
        let scope = match session.role {
            Role::Participant => Scope::Aggregate,
            Role::Operator | Role::Support | Role::Validation | Role::Security | Role::Release => {
                Scope::Operator
            }
        };
        // A diagnostic lease never upgrades a browser role implicitly. The trusted
        // application must issue both a diagnostic-scoped session and narrow lease;
        // current browser roles therefore fail closed for CSI diagnostics.
        let authorization = Authorization {
            scope,
            now_ms: request.now_ms,
            diagnostic: self.diagnostic_lease.clone(),
        };
        let page = query_history(&self.samples, request.query, &authorization).map_err(
            |error| match error {
                ste_query_plane::Error::Forbidden => GatewayError::QueryForbidden,
                _ => GatewayError::QueryRejected,
            },
        )?;
        serde_json::to_value(page).map_err(|_| GatewayError::QueryRejected)
    }
}

fn command_fingerprint(command: &str, body: &Value) -> Result<String, GatewayError> {
    let canonical = serde_json::to_vec(body).map_err(|_| GatewayError::InvalidReadModel)?;
    let mut digest = Sha256::new();
    digest.update(command.as_bytes());
    digest.update([0]);
    digest.update(canonical);
    Ok(format!("{:x}", digest.finalize()))
}

fn redact_value(mut value: Value) -> Value {
    match &mut value {
        Value::Object(map) => {
            map.retain(|key, _| {
                let normalized = key.to_ascii_lowercase();
                !PROHIBITED_KEYS.iter().any(|bad| normalized.contains(bad))
            });
            for child in map.values_mut() {
                *child = redact_value(child.take());
            }
        }
        Value::Array(items) => {
            for child in items {
                *child = redact_value(child.take());
            }
        }
        _ => {}
    }
    value
}

/// Configuration for an immutable static UI host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticHostConfig {
    /// Loopback socket; wildcard and LAN bindings are prohibited.
    pub bind: SocketAddr,
    /// Relative, traversal-free asset root resolved by the hosting application.
    pub asset_root: PathBuf,
}

impl StaticHostConfig {
    /// Rejects non-loopback listeners, port zero, and ambiguous asset roots.
    pub fn validate(&self) -> Result<(), GatewayError> {
        if !self.bind.ip().is_loopback()
            || self.bind.port() == 0
            || self.asset_root.is_absolute()
            || self.asset_root.as_os_str().is_empty()
            || self
                .asset_root
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            Err(GatewayError::UnsafeHost)
        } else {
            Ok(())
        }
    }
}

/// Bounded stream drops the oldest approved projection and makes loss observable.
#[derive(Clone, Debug)]
pub struct BoundedStream {
    capacity: usize,
    dropped: u64,
    queue: VecDeque<ReadModel>,
}
impl BoundedStream {
    /// Creates a stream with a strict maximum capacity.
    pub fn new(capacity: usize) -> Result<Self, GatewayError> {
        if capacity == 0 || capacity > 1024 {
            Err(GatewayError::InvalidCapacity)
        } else {
            Ok(Self {
                capacity,
                dropped: 0,
                queue: VecDeque::new(),
            })
        }
    }
    /// Enqueues a projection, evicting the oldest when full.
    pub fn push(&mut self, model: ReadModel) {
        if self.queue.len() == self.capacity {
            self.queue.pop_front();
            self.dropped += 1;
        }
        self.queue.push_back(model);
    }
    /// Takes the oldest remaining projection.
    pub fn pop(&mut self) -> Option<ReadModel> {
        self.queue.pop_front()
    }
    /// Returns the observable number of evicted projections.
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }
}

/// Pinned production asset and its lowercase SHA-256 digest.
#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AssetEntry {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}
/// Immutable local production asset manifest.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AssetManifest {
    pub assets: Vec<AssetEntry>,
}
impl AssetManifest {
    /// Verifies exact membership, paths, sizes, and digests.
    pub fn verify(&self, files: &BTreeMap<String, Vec<u8>>) -> Result<(), GatewayError> {
        if self.assets.is_empty() || self.assets.len() != files.len() {
            return Err(GatewayError::AssetMismatch);
        }
        let mut seen = BTreeSet::new();
        for asset in &self.assets {
            if asset.path.starts_with('/') || asset.path.contains("..") || !seen.insert(&asset.path)
            {
                return Err(GatewayError::AssetMismatch);
            }
            let body = files.get(&asset.path).ok_or(GatewayError::AssetMismatch)?;
            let digest = format!("{:x}", Sha256::digest(body));
            if body.len() as u64 != asset.bytes || !same(&digest, &asset.sha256) {
                return Err(GatewayError::AssetMismatch);
            }
        }
        Ok(())
    }
}

/// Fail-closed boundary errors; no sensitive input is reflected.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayError {
    InvalidReadModel,
    SchemaMismatch,
    Stale,
    ProhibitedField,
    PayloadComplexity,
    UnsafePolicy,
    Unauthorized,
    InvalidAllowlist,
    InvalidIdempotencyKey,
    CommandDenied,
    InvalidCapacity,
    AssetMismatch,
    BodyTooLarge,
    RateLimited,
    IdempotencyConflict,
    ExecutionRejected,
    UnsafeHost,
    QueryRejected,
    QueryForbidden,
}
impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UI gateway request rejected: {self:?}")
    }
}
impl std::error::Error for GatewayError {}
