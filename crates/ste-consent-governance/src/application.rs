//! Commands, policy queries, repository ports, and separated records.
#![allow(missing_docs)]

use crate::domain::{
    AuthorizationRequest, DenialReason, DomainBoundary, DomainError, GovernanceCommand,
    GovernanceEvent, PolicyDecision, SensingAuthorization, SensingAuthorizationId,
};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Returns a domain marker without exposing adapter internals.
#[must_use]
pub const fn boundary() -> DomainBoundary {
    DomainBoundary
}

/// Persistence error with no sensitive payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepositoryError {
    Unavailable,
    Corrupt,
    Conflict,
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Unavailable => "authorization repository unavailable",
            Self::Corrupt => "authorization repository is corrupt",
            Self::Conflict => "authorization repository write conflict",
        })
    }
}

impl Error for RepositoryError {}

/// Narrow repository port owned by the application layer.
pub trait SensingAuthorizationRepository: Send + Sync {
    fn load(
        &self,
        id: &SensingAuthorizationId,
    ) -> Result<Option<SensingAuthorization>, RepositoryError>;
    fn save(&self, authorization: &SensingAuthorization) -> Result<(), RepositoryError>;
}

/// Fail-closed capture query. Repository absence, corruption, and downtime deny.
#[derive(Clone)]
pub struct PolicyDecisionPoint {
    repository: Arc<dyn SensingAuthorizationRepository>,
}

impl PolicyDecisionPoint {
    #[must_use]
    pub fn new(repository: Arc<dyn SensingAuthorizationRepository>) -> Self {
        Self { repository }
    }

    #[must_use]
    pub fn evaluate(
        &self,
        id: &SensingAuthorizationId,
        request: &AuthorizationRequest,
    ) -> PolicyDecision {
        match self.repository.load(id) {
            Ok(Some(authorization)) => authorization.evaluate(request),
            Ok(None) | Err(_) => PolicyDecision::Denied(DenialReason::NotGranted),
        }
    }
}

/// Payload-minimized domain audit entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainAuditRecord {
    pub authorization_id: String,
    pub event_kind: &'static str,
}

/// Participant-visible history remains separate from operational records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantHistoryRecord {
    pub event: GovernanceEvent,
}

/// Security record, never mixed into participant history or diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityRecord {
    pub code: &'static str,
}

/// Bounded diagnostic record. It cannot carry arbitrary sensitive objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticRecord {
    pub code: &'static str,
    pub detail: &'static str,
}

impl DiagnosticRecord {
    /// Creates a bounded diagnostic while replacing forbidden sensitive detail.
    #[must_use]
    pub fn redacted(code: &'static str, detail: &'static str) -> Self {
        const FORBIDDEN: [&str; 7] = [
            "secret",
            "password",
            "participant",
            "raw_csi",
            "embedding",
            "model_input",
            "prompt",
        ];
        let normalized = detail.to_ascii_lowercase();
        let detail = if FORBIDDEN.iter().any(|word| normalized.contains(word)) {
            "[REDACTED]"
        } else {
            detail
        };
        Self { code, detail }
    }
}

/// Separated output ports for records with different audiences and lifecycles.
pub trait GovernanceRecords: Send + Sync {
    fn domain_audit(&self, record: DomainAuditRecord);
    fn participant_history(&self, record: ParticipantHistoryRecord);
    fn security(&self, record: SecurityRecord);
    fn diagnostic(&self, record: DiagnosticRecord);
}

/// Application failure preserves typed domain/repository causality.
#[derive(Debug)]
pub enum ApplicationError {
    NotFound,
    Domain(DomainError),
    Repository(RepositoryError),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => formatter.write_str("authorization not found"),
            Self::Domain(error) => error.fmt(formatter),
            Self::Repository(error) => error.fmt(formatter),
        }
    }
}

impl Error for ApplicationError {}

impl From<DomainError> for ApplicationError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}

impl From<RepositoryError> for ApplicationError {
    fn from(value: RepositoryError) -> Self {
        Self::Repository(value)
    }
}

/// Offline-capable command service. Events are persisted before publication.
pub struct GovernanceApplicationService {
    repository: Arc<dyn SensingAuthorizationRepository>,
    records: Arc<dyn GovernanceRecords>,
}

impl GovernanceApplicationService {
    #[must_use]
    pub fn new(
        repository: Arc<dyn SensingAuthorizationRepository>,
        records: Arc<dyn GovernanceRecords>,
    ) -> Self {
        Self {
            repository,
            records,
        }
    }

    pub fn create(&self, authorization: SensingAuthorization) -> Result<(), ApplicationError> {
        self.repository.save(&authorization)?;
        Ok(())
    }

    pub fn execute(
        &self,
        id: &SensingAuthorizationId,
        command: GovernanceCommand,
        now: u64,
    ) -> Result<Vec<GovernanceEvent>, ApplicationError> {
        let mut authorization = self
            .repository
            .load(id)?
            .ok_or(ApplicationError::NotFound)?;
        let events = match command {
            GovernanceCommand::AuthorizeSpace(space) => authorization.authorize_space(space)?,
            GovernanceCommand::RecordParticipantConsent(consent) => authorization
                .record_participant_consent(
                    consent.participant,
                    consent.purpose,
                    consent.version,
                    consent.valid_until,
                    now,
                )?,
            GovernanceCommand::ApplyRetentionPolicy(rule) => {
                authorization.apply_retention_policy(rule)?
            }
            GovernanceCommand::GrantSensingAuthorization(grant) => authorization.grant(
                authorization
                    .space()
                    .cloned()
                    .ok_or(DomainError::SpaceNotAuthorized)?,
                grant.purpose,
                grant.participants,
                grant.expires_at,
                now,
            )?,
            GovernanceCommand::RevokeConsent {
                participant,
                reason,
                revoked_at,
            } => authorization.revoke_consent(participant, reason, revoked_at)?,
            GovernanceCommand::RecordDeletionCompletion {
                participant,
                completed_at,
            } => authorization.record_deletion_completion(participant, completed_at)?,
        };
        self.repository.save(&authorization)?;
        self.publish_records(id, &events);
        Ok(events)
    }

    fn publish_records(&self, id: &SensingAuthorizationId, events: &[GovernanceEvent]) {
        for event in events {
            self.records.domain_audit(DomainAuditRecord {
                authorization_id: id.as_str().to_owned(),
                event_kind: event_kind(event),
            });
            self.records.participant_history(ParticipantHistoryRecord {
                event: event.clone(),
            });
        }
    }
}

fn event_kind(event: &GovernanceEvent) -> &'static str {
    match event {
        GovernanceEvent::SpaceAuthorized { .. } => "space_authorized",
        GovernanceEvent::ParticipantConsentRecorded { .. } => "consent_recorded",
        GovernanceEvent::RetentionPolicyApplied { .. } => "retention_applied",
        GovernanceEvent::SensingAuthorizationGranted { .. } => "authorization_granted",
        GovernanceEvent::SensingAuthorizationRevoked { .. } => "authorization_revoked",
        GovernanceEvent::ParticipantDeletionRequested { .. } => "deletion_requested",
        GovernanceEvent::ParticipantDeletionCompleted { .. } => "deletion_completed",
        GovernanceEvent::RetentionExpired { .. } => "retention_expired",
    }
}
