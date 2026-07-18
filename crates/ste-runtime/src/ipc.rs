//! Authenticated, bounded local IPC protocol and server/client abstractions.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Stable wire schema version.
pub const IPC_SCHEMA_V1: &str = "ste-ipc-v1";

/// Operator authorization role derived from peer credentials and a local secret.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRole {
    /// Read-only diagnostics.
    Viewer,
    /// Non-destructive operations.
    Operator,
    /// Destructive or lifecycle administration.
    Administrator,
}

/// Stable command classes with explicit minimum authorization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcCommand {
    /// Read status.
    Status,
    /// Run diagnostics.
    Doctor,
    /// Run bounded capture test.
    CaptureTest,
    /// Run calibration.
    Calibrate,
    /// Apply an update.
    Update,
    /// Reset managed state.
    Reset,
}
impl IpcCommand {
    fn required_role(&self) -> OperatorRole {
        match self {
            Self::Status | Self::Doctor => OperatorRole::Viewer,
            Self::CaptureTest | Self::Calibrate => OperatorRole::Operator,
            Self::Update | Self::Reset => OperatorRole::Administrator,
        }
    }
}

/// Stable authenticated request. Debug output always redacts the credential.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IpcRequest {
    /// Must equal [`IPC_SCHEMA_V1`].
    pub schema: String,
    /// Stable request identity.
    pub request_id: String,
    /// Required idempotency key.
    pub idempotency_key: String,
    /// Unique anti-replay nonce.
    pub nonce: String,
    /// Client wall-clock timestamp used only for a bounded acceptance window.
    pub issued_at_unix_seconds: u64,
    /// Local authentication secret.
    pub credential: String,
    /// Requested operation.
    pub command: IpcCommand,
    /// Bounded structured parameters.
    pub parameters: Value,
}
impl fmt::Debug for IpcRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpcRequest")
            .field("schema", &self.schema)
            .field("request_id", &self.request_id)
            .field("idempotency_key", &self.idempotency_key)
            .field("nonce", &self.nonce)
            .field("issued_at_unix_seconds", &self.issued_at_unix_seconds)
            .field("credential", &"[REDACTED]")
            .field("command", &self.command)
            .field("parameters", &redact_value(&self.parameters))
            .finish()
    }
}

/// Stable response envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IpcResponse {
    /// Schema version.
    pub schema: String,
    /// Correlated request identity, empty only when decoding failed.
    pub request_id: String,
    /// Typed process exit category.
    pub exit: TypedExit,
    /// Redacted structured result or diagnostic.
    pub body: Value,
}

/// Stable CLI exit semantics.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TypedExit {
    /// Request succeeded.
    Success,
    /// Request was malformed or expired.
    InvalidInput,
    /// Authentication failed.
    Unauthorized,
    /// Role lacks permission.
    Forbidden,
    /// Replay or idempotency conflict.
    Conflict,
    /// Required service unavailable.
    Unavailable,
    /// Internal handler failure.
    Internal,
}
impl TypedExit {
    /// Stable POSIX-compatible numeric code.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::InvalidInput => 2,
            Self::Unauthorized => 3,
            Self::Forbidden => 4,
            Self::Conflict => 5,
            Self::Unavailable => 6,
            Self::Internal => 70,
        }
    }
}

/// Kernel-derived peer identity supplied by the Unix socket acceptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeerIdentity {
    /// Effective Unix user ID.
    pub uid: u32,
}

#[derive(Clone)]
struct Credential {
    digest: [u8; 32],
    role: OperatorRole,
}

/// Local credential registry keyed by kernel peer UID.
#[derive(Clone, Default)]
pub struct IpcAuthenticator {
    credentials: BTreeMap<u32, Credential>,
}
impl IpcAuthenticator {
    /// Adds/replaces a UID-bound credential; plaintext is not retained.
    pub fn register(&mut self, uid: u32, secret: &str, role: OperatorRole) -> Result<(), IpcError> {
        if secret.len() < 16 || secret.len() > 256 {
            return Err(IpcError::InvalidCredential);
        }
        self.credentials.insert(
            uid,
            Credential {
                digest: digest(secret.as_bytes()),
                role,
            },
        );
        Ok(())
    }
    fn authenticate(&self, peer: PeerIdentity, secret: &str) -> Result<OperatorRole, IpcError> {
        let expected = self
            .credentials
            .get(&peer.uid)
            .ok_or(IpcError::Unauthenticated)?;
        if constant_time_equal(&expected.digest, &digest(secret.as_bytes())) {
            Ok(expected.role)
        } else {
            Err(IpcError::Unauthenticated)
        }
    }
}

/// Application command handler behind the authenticated boundary.
pub trait IpcHandler {
    /// Handles one authorized request and returns an already-safe value.
    fn handle(
        &mut self,
        role: OperatorRole,
        command: &IpcCommand,
        parameters: &Value,
    ) -> Result<Value, IpcError>;
}

#[derive(Clone)]
struct ReplayEntry {
    request_digest: [u8; 32],
    response: IpcResponse,
}

/// Stateful bounded server core used by a Unix-domain accept loop.
pub struct IpcServer<H> {
    authenticator: IpcAuthenticator,
    handler: H,
    maximum_request_bytes: usize,
    maximum_clock_skew_seconds: u64,
    replay_capacity: usize,
    seen_nonces: BTreeMap<(u32, String), u64>,
    idempotency: BTreeMap<(u32, String), ReplayEntry>,
}
impl<H: IpcHandler> IpcServer<H> {
    /// Creates a bounded server core.
    pub fn new(
        authenticator: IpcAuthenticator,
        handler: H,
        maximum_request_bytes: usize,
        maximum_clock_skew_seconds: u64,
        replay_capacity: usize,
    ) -> Result<Self, IpcError> {
        if !(256..=1_048_576).contains(&maximum_request_bytes)
            || maximum_clock_skew_seconds == 0
            || replay_capacity == 0
        {
            return Err(IpcError::InvalidConfiguration);
        }
        Ok(Self {
            authenticator,
            handler,
            maximum_request_bytes,
            maximum_clock_skew_seconds,
            replay_capacity,
            seen_nonces: BTreeMap::new(),
            idempotency: BTreeMap::new(),
        })
    }

    /// Decodes and executes one frame for a kernel-authenticated peer.
    pub fn handle_frame(&mut self, peer: PeerIdentity, frame: &[u8], now: u64) -> IpcResponse {
        match self.try_handle(peer, frame, now) {
            Ok(response) => response,
            Err(error) => error_response(error),
        }
    }
    fn try_handle(
        &mut self,
        peer: PeerIdentity,
        frame: &[u8],
        now: u64,
    ) -> Result<IpcResponse, IpcError> {
        if frame.is_empty() || frame.len() > self.maximum_request_bytes {
            return Err(IpcError::RequestTooLarge);
        }
        let request: IpcRequest =
            serde_json::from_slice(frame).map_err(|_| IpcError::MalformedRequest)?;
        validate_request(&request)?;
        let role = self.authenticator.authenticate(peer, &request.credential)?;
        if role < request.command.required_role() {
            return Err(IpcError::Forbidden);
        }
        if now.abs_diff(request.issued_at_unix_seconds) > self.maximum_clock_skew_seconds {
            return Err(IpcError::ExpiredRequest);
        }
        let idempotency_key = (peer.uid, request.idempotency_key.clone());
        let request_digest = canonical_request_digest(&request)?;
        if let Some(entry) = self.idempotency.get(&idempotency_key) {
            return if entry.request_digest == request_digest {
                Ok(entry.response.clone())
            } else {
                Err(IpcError::IdempotencyConflict)
            };
        }
        let nonce_key = (peer.uid, request.nonce.clone());
        if self.seen_nonces.contains_key(&nonce_key) {
            return Err(IpcError::ReplayDetected);
        }
        if self.seen_nonces.len() >= self.replay_capacity {
            return Err(IpcError::ReplayCacheFull);
        }
        self.seen_nonces.insert(nonce_key, now);
        let body = redact_value(&self.handler.handle(
            role,
            &request.command,
            &request.parameters,
        )?);
        let response = IpcResponse {
            schema: IPC_SCHEMA_V1.into(),
            request_id: request.request_id,
            exit: TypedExit::Success,
            body,
        };
        self.idempotency.insert(
            idempotency_key,
            ReplayEntry {
                request_digest,
                response: response.clone(),
            },
        );
        Ok(response)
    }
}

/// Client-side stable encoder and validated Unix socket target.
#[derive(Clone, Debug)]
pub struct IpcClient {
    socket_path: PathBuf,
    maximum_request_bytes: usize,
}
impl IpcClient {
    /// Validates an absolute Unix socket path and frame limit.
    pub fn new(
        socket_path: impl AsRef<Path>,
        maximum_request_bytes: usize,
    ) -> Result<Self, IpcError> {
        let path = socket_path.as_ref();
        if !path.is_absolute() || path.as_os_str().is_empty() || maximum_request_bytes < 256 {
            return Err(IpcError::InvalidConfiguration);
        }
        Ok(Self {
            socket_path: path.to_owned(),
            maximum_request_bytes,
        })
    }
    /// Socket path to connect with owner-only filesystem permissions.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
    /// Encodes a stable bounded frame.
    pub fn encode(&self, request: &IpcRequest) -> Result<Vec<u8>, IpcError> {
        validate_request(request)?;
        let bytes = serde_json::to_vec(request).map_err(|_| IpcError::MalformedRequest)?;
        if bytes.len() > self.maximum_request_bytes {
            Err(IpcError::RequestTooLarge)
        } else {
            Ok(bytes)
        }
    }
}

fn validate_request(request: &IpcRequest) -> Result<(), IpcError> {
    if request.schema != IPC_SCHEMA_V1
        || request.request_id.len() > 128
        || request.request_id.is_empty()
        || request.idempotency_key.len() < 8
        || request.idempotency_key.len() > 128
        || request.nonce.len() < 16
        || request.nonce.len() > 128
        || request.credential.len() > 256
        || !bounded_value(&request.parameters, 0)
    {
        Err(IpcError::MalformedRequest)
    } else {
        Ok(())
    }
}
fn bounded_value(value: &Value, depth: usize) -> bool {
    if depth > 8 {
        return false;
    }
    match value {
        Value::String(value) => value.len() <= 4096,
        Value::Array(values) => {
            values.len() <= 128 && values.iter().all(|value| bounded_value(value, depth + 1))
        }
        Value::Object(values) => {
            values.len() <= 128
                && values
                    .iter()
                    .all(|(key, value)| key.len() <= 128 && bounded_value(value, depth + 1))
        }
        _ => true,
    }
}
fn canonical_request_digest(request: &IpcRequest) -> Result<[u8; 32], IpcError> {
    serde_json::to_vec(request)
        .map(|bytes| digest(&bytes))
        .map_err(|_| IpcError::MalformedRequest)
}
fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
fn constant_time_equal(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.iter()
        .zip(right)
        .fold(0_u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}
fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| {
                    let lower = key.to_ascii_lowercase();
                    (
                        key.clone(),
                        if ["credential", "secret", "token", "password"]
                            .iter()
                            .any(|term| lower.contains(term))
                        {
                            Value::String("[REDACTED]".into())
                        } else {
                            redact_value(value)
                        },
                    )
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(redact_value).collect()),
        _ => value.clone(),
    }
}
fn error_response(error: IpcError) -> IpcResponse {
    let exit = match error {
        IpcError::Unauthenticated => TypedExit::Unauthorized,
        IpcError::Forbidden => TypedExit::Forbidden,
        IpcError::ReplayDetected | IpcError::IdempotencyConflict | IpcError::ReplayCacheFull => {
            TypedExit::Conflict
        }
        IpcError::HandlerUnavailable => TypedExit::Unavailable,
        IpcError::HandlerFailed => TypedExit::Internal,
        _ => TypedExit::InvalidInput,
    };
    IpcResponse {
        schema: IPC_SCHEMA_V1.into(),
        request_id: String::new(),
        exit,
        body: serde_json::json!({"error": error.code()}),
    }
}

/// Typed IPC boundary failure with non-sensitive stable codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpcError {
    /// Server/client bounds are invalid.
    InvalidConfiguration,
    /// Credential registration value is unsafe.
    InvalidCredential,
    /// Frame exceeds configured bound.
    RequestTooLarge,
    /// JSON or request fields are invalid.
    MalformedRequest,
    /// Peer or credential was not authenticated.
    Unauthenticated,
    /// Authenticated role lacks permission.
    Forbidden,
    /// Timestamp lies outside the acceptance window.
    ExpiredRequest,
    /// Nonce was already consumed.
    ReplayDetected,
    /// Replay cache cannot safely accept more entries.
    ReplayCacheFull,
    /// Idempotency key was reused for different content.
    IdempotencyConflict,
    /// Handler dependency is unavailable.
    HandlerUnavailable,
    /// Handler failed internally.
    HandlerFailed,
}
impl IpcError {
    fn code(self) -> &'static str {
        match self {
            Self::InvalidConfiguration => "invalid_configuration",
            Self::InvalidCredential => "invalid_credential",
            Self::RequestTooLarge => "request_too_large",
            Self::MalformedRequest => "malformed_request",
            Self::Unauthenticated => "unauthenticated",
            Self::Forbidden => "forbidden",
            Self::ExpiredRequest => "expired_request",
            Self::ReplayDetected => "replay_detected",
            Self::ReplayCacheFull => "replay_cache_full",
            Self::IdempotencyConflict => "idempotency_conflict",
            Self::HandlerUnavailable => "handler_unavailable",
            Self::HandlerFailed => "handler_failed",
        }
    }
}
