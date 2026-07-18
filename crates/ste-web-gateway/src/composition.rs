//! Production composition over the bounded query plane and durable workflow engine.

use super::{ApplicationServices, BrowserSession, StreamEvent};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Mutex, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use ste_query_plane::{
    Authorization as QueryAuthorization, HistoryPage, HistoryQuery, LiveRing, Sample, Scope,
    query_history,
};
use ste_ui_gateway::{FunctionalArea, GatewayError, ReadModel, Role};
use ste_workflows::{
    Action, Authorization, Command, Journal, WorkflowEngine, WorkflowRequest, WorkflowType,
};

/// Server-side browser session record; cookie material is never returned in projections.
#[derive(Clone, Debug)]
pub struct SessionRecord {
    /// Opaque session projection.
    pub session: BrowserSession,
    /// Absolute server expiry.
    pub expires_at_ms: u64,
    /// Query scope determined by policy.
    pub query_scope: Scope,
    /// Immutable authorization reference for workflow creation.
    pub authorization_ref: String,
    /// Immutable purpose reference for workflow creation.
    pub purpose_ref: String,
}

/// Real application adapter connecting HTTP to bounded projections and durable workflows.
pub struct ProductionServices<J, A> {
    sessions: RwLock<BTreeMap<String, SessionRecord>>,
    live: Mutex<LiveRing>,
    live_capacity: usize,
    workflows: WorkflowEngine<J, A>,
}

impl<J: Journal, A: Authorization> ProductionServices<J, A> {
    /// Creates a composition around a bounded ring and a journal-backed workflow engine.
    pub fn new(live_capacity: usize, journal: J, authorization: A) -> Result<Self, GatewayError> {
        Ok(Self {
            sessions: RwLock::new(BTreeMap::new()),
            live: Mutex::new(
                LiveRing::new(live_capacity).map_err(|_| GatewayError::InvalidCapacity)?,
            ),
            live_capacity,
            workflows: WorkflowEngine::new(journal, authorization),
        })
    }

    /// Registers server-provisioned cookie material. Production callers must provide entropy.
    pub fn register_session(
        &self,
        cookie: String,
        record: SessionRecord,
    ) -> Result<(), GatewayError> {
        if cookie.len() < 32 || record.session.id.is_empty() || record.expires_at_ms <= now_ms() {
            return Err(GatewayError::Unauthorized);
        }
        self.sessions
            .write()
            .map_err(|_| GatewayError::ExecutionRejected)?
            .insert(cookie, record);
        Ok(())
    }

    /// Publishes one already-policy-approved sample without coupling producers to HTTP clients.
    pub fn publish(&self, sample: Sample) -> Result<u64, GatewayError> {
        self.live
            .lock()
            .map_err(|_| GatewayError::ExecutionRejected)?
            .push(sample)
            .map_err(|_| GatewayError::InvalidReadModel)
    }

    fn record(&self, cookie: &str) -> Result<SessionRecord, GatewayError> {
        let record = self
            .sessions
            .read()
            .map_err(|_| GatewayError::Unauthorized)?
            .get(cookie)
            .cloned()
            .ok_or(GatewayError::Unauthorized)?;
        if record.expires_at_ms <= now_ms() {
            Err(GatewayError::Unauthorized)
        } else {
            Ok(record)
        }
    }

    fn query_auth(record: &SessionRecord) -> QueryAuthorization {
        QueryAuthorization {
            scope: record.query_scope,
            now_ms: now_ms(),
            diagnostic: None,
        }
    }
}

#[derive(Deserialize)]
struct CreateWorkflow {
    workflow_type: WorkflowType,
    scope: String,
    expires_at_ms: u64,
    correlation_id: Option<String>,
}

#[derive(Deserialize)]
struct ApplyWorkflow {
    workflow_id: String,
    expected_version: u64,
    action: Action,
}

impl<J: Journal + Send + Sync + 'static, A: Authorization + Send + Sync + 'static>
    ApplicationServices for ProductionServices<J, A>
{
    fn session(&self, cookie: &str) -> Result<BrowserSession, GatewayError> {
        Ok(self.record(cookie)?.session)
    }

    fn read_model(&self, session: &BrowserSession, area: &str) -> Result<ReadModel, GatewayError> {
        if area != "live_overview" {
            return Err(GatewayError::SchemaMismatch);
        }
        let record = self
            .sessions
            .read()
            .map_err(|_| GatewayError::Unauthorized)?
            .values()
            .find(|r| r.session.id == session.id && r.expires_at_ms > now_ms())
            .cloned()
            .ok_or(GatewayError::Unauthorized)?;
        let snapshot = self
            .live
            .lock()
            .map_err(|_| GatewayError::ExecutionRejected)?
            .resume(0, self.live_capacity, &Self::query_auth(&record))
            .map_err(query_error)?;
        let gap = snapshot.gap.map(|gap| {
            json!({
                "requested_after": gap.requested_after,
                "first_available": gap.first_available,
                "dropped": gap.dropped,
            })
        });
        ReadModel::new(
            FunctionalArea::LiveOverview,
            snapshot.next_sequence.max(1),
            now_ms(),
            5_000,
            "ste-query-plane/v1",
            json!({"samples": snapshot.samples, "gap": gap}),
        )
    }

    fn stream(
        &self,
        session: &BrowserSession,
        name: &str,
        after: Option<u64>,
    ) -> Result<Vec<StreamEvent>, GatewayError> {
        if let Some(id) = name.strip_prefix("workflow:") {
            let projection = self.workflows.snapshot(id).map_err(engine_error)?;
            return Ok(vec![StreamEvent {
                sequence: projection.version,
                dropped: 0,
                schema_version: 1,
                payload: serde_json::to_value(projection)
                    .map_err(|_| GatewayError::ExecutionRejected)?,
            }]);
        }
        if name != "live" {
            return Err(GatewayError::CommandDenied);
        }
        let record = self
            .sessions
            .read()
            .map_err(|_| GatewayError::Unauthorized)?
            .values()
            .find(|r| r.session.id == session.id && r.expires_at_ms > now_ms())
            .cloned()
            .ok_or(GatewayError::Unauthorized)?;
        let snapshot = self
            .live
            .lock()
            .map_err(|_| GatewayError::ExecutionRejected)?
            .resume(
                after.unwrap_or(0),
                self.live_capacity,
                &Self::query_auth(&record),
            )
            .map_err(query_error)?;
        let dropped = snapshot.gap.map_or(0, |gap| gap.dropped);
        snapshot
            .samples
            .into_iter()
            .enumerate()
            .map(|(index, sample)| {
                Ok(StreamEvent {
                    sequence: sample.sequence,
                    dropped: if index == 0 { dropped } else { 0 },
                    schema_version: 1,
                    payload: serde_json::to_value(sample)
                        .map_err(|_| GatewayError::ExecutionRejected)?,
                })
            })
            .collect()
    }

    fn command(
        &self,
        session: &BrowserSession,
        name: &str,
        idempotency_key: &str,
        payload: &Value,
    ) -> Result<Value, GatewayError> {
        let record = self
            .sessions
            .read()
            .map_err(|_| GatewayError::Unauthorized)?
            .values()
            .find(|r| r.session.id == session.id && r.expires_at_ms > now_ms())
            .cloned()
            .ok_or(GatewayError::Unauthorized)?;
        match name {
            "workflow.create" => {
                let input: CreateWorkflow = serde_json::from_value(payload.clone())
                    .map_err(|_| GatewayError::InvalidReadModel)?;
                let request = WorkflowRequest {
                    workflow_type: input.workflow_type,
                    scope: input.scope,
                    requester_role: workflow_role(session.role),
                    requester_session: session.id.clone(),
                    authorization_ref: record.authorization_ref,
                    purpose_ref: record.purpose_ref,
                    idempotency_key: idempotency_key.to_owned(),
                    correlation_id: input.correlation_id,
                    expires_at_ms: input.expires_at_ms,
                };
                serde_json::to_value(
                    self.workflows
                        .create(request, now_ms())
                        .map_err(engine_error)?,
                )
                .map_err(|_| GatewayError::ExecutionRejected)
            }
            "workflow.apply" => {
                let input: ApplyWorkflow = serde_json::from_value(payload.clone())
                    .map_err(|_| GatewayError::InvalidReadModel)?;
                serde_json::to_value(
                    self.workflows
                        .execute(
                            &input.workflow_id,
                            input.expected_version,
                            Command::Apply(input.action),
                            now_ms(),
                        )
                        .map_err(engine_error)?,
                )
                .map_err(|_| GatewayError::ExecutionRejected)
            }
            _ => Err(GatewayError::CommandDenied),
        }
    }

    fn query(
        &self,
        session: &BrowserSession,
        query: &HistoryQuery,
    ) -> Result<HistoryPage, GatewayError> {
        let record = self
            .sessions
            .read()
            .map_err(|_| GatewayError::Unauthorized)?
            .values()
            .find(|r| r.session.id == session.id && r.expires_at_ms > now_ms())
            .cloned()
            .ok_or(GatewayError::Unauthorized)?;
        let snapshot = self
            .live
            .lock()
            .map_err(|_| GatewayError::ExecutionRejected)?
            .resume(0, self.live_capacity, &Self::query_auth(&record))
            .map_err(query_error)?;
        query_history(&snapshot.samples, query, &Self::query_auth(&record)).map_err(query_error)
    }

    fn workflow(&self, session: &BrowserSession, id: &str) -> Result<Value, GatewayError> {
        let _ = self
            .sessions
            .read()
            .map_err(|_| GatewayError::Unauthorized)?
            .values()
            .find(|r| r.session.id == session.id && r.expires_at_ms > now_ms())
            .ok_or(GatewayError::Unauthorized)?;
        serde_json::to_value(self.workflows.snapshot(id).map_err(engine_error)?)
            .map_err(|_| GatewayError::ExecutionRejected)
    }
}

fn workflow_role(role: Role) -> ste_workflows::Role {
    match role {
        Role::Participant => ste_workflows::Role::Viewer,
        Role::Operator | Role::Support => ste_workflows::Role::Operator,
        Role::Validation => ste_workflows::Role::Researcher,
        Role::Security => ste_workflows::Role::SafetyOfficer,
        Role::Release => ste_workflows::Role::Administrator,
    }
}

fn query_error(error: ste_query_plane::Error) -> GatewayError {
    match error {
        ste_query_plane::Error::Forbidden => GatewayError::QueryForbidden,
        ste_query_plane::Error::Bounds => GatewayError::QueryRejected,
        _ => GatewayError::InvalidReadModel,
    }
}
fn engine_error(error: ste_workflows::EngineError) -> GatewayError {
    match error {
        ste_workflows::EngineError::NotFound => GatewayError::InvalidReadModel,
        ste_workflows::EngineError::IdempotencyConflict
        | ste_workflows::EngineError::VersionConflict => GatewayError::IdempotencyConflict,
        ste_workflows::EngineError::Unauthorized(_) => GatewayError::Unauthorized,
        _ => GatewayError::ExecutionRejected,
    }
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}
