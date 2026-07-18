//! Consent and governance aggregate, value objects, commands, and domain events.

use std::{collections::BTreeMap, collections::BTreeSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

macro_rules! string_value {
    ($name:ident, $label:literal) => {
        #[doc = $label]
        #[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        pub struct $name(String);

        impl $name {
            /// Creates a non-empty, bounded value.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.trim().is_empty() || value.len() > 128 {
                    return Err(DomainError::InvalidValue($label));
                }
                Ok(Self(value))
            }

            /// Returns the opaque value for persistence or exact comparison.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

string_value!(SensingAuthorizationId, "sensing authorization identifier");
string_value!(ParticipantPseudonym, "participant pseudonym");
string_value!(SpaceId, "space identifier");

/// Version of the consent statement accepted by a participant.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ConsentVersion(u32);

impl ConsentVersion {
    /// Creates a non-zero consent version.
    pub fn new(value: u32) -> Result<Self, DomainError> {
        (value > 0)
            .then_some(Self(value))
            .ok_or(DomainError::InvalidValue("consent version"))
    }
}

/// Version of the policy used for a decision.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct PolicyVersion(u32);

impl PolicyVersion {
    /// Creates a non-zero policy version.
    pub fn new(value: u32) -> Result<Self, DomainError> {
        (value > 0)
            .then_some(Self(value))
            .ok_or(DomainError::InvalidValue("policy version"))
    }
}

/// Exact, purpose-limited reason for sensing.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Purpose {
    /// Approved research protocol.
    Research,
    /// Approved non-medical wellness capability.
    Wellness,
    /// Site or device calibration without latent claims.
    Calibration,
    /// Identity or re-identification.
    IdentityInference,
    /// Medical diagnosis, screening, or treatment.
    ClinicalDiagnosis,
    /// Workplace or hiring scoring.
    EmploymentScoring,
    /// Deception or truthfulness inference.
    DeceptionDetection,
    /// Sensing concealed from affected people.
    CovertSensing,
    /// Use unrelated to the participant's exact grant.
    UnrelatedSecondaryUse,
}

impl Purpose {
    /// Returns whether this purpose is permanently prohibited by domain policy.
    #[must_use]
    pub const fn is_prohibited(self) -> bool {
        matches!(
            self,
            Self::IdentityInference
                | Self::ClinicalDiagnosis
                | Self::EmploymentScoring
                | Self::DeceptionDetection
                | Self::CovertSensing
                | Self::UnrelatedSecondaryUse
        )
    }
}

/// Separately governed data classes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DataClass {
    /// Unprocessed channel-state information.
    RawCsi,
    /// Derived, non-physiological observations.
    Observation,
    /// Physiological estimates.
    Physiology,
    /// Latent-state estimates.
    LatentState,
    /// Participant personalization anchors.
    Anchor,
}

impl DataClass {
    /// Every sensitive data class requiring an explicit retention rule.
    pub const ALL: [Self; 5] = [
        Self::RawCsi,
        Self::Observation,
        Self::Physiology,
        Self::LatentState,
        Self::Anchor,
    ];
}

/// Positive retention duration in seconds.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RetentionPeriod(u64);

impl RetentionPeriod {
    /// Creates a non-zero period capped at ten years.
    pub fn new(seconds: u64) -> Result<Self, DomainError> {
        const TEN_YEARS_SECONDS: u64 = 10 * 366 * 24 * 60 * 60;
        (seconds > 0 && seconds <= TEN_YEARS_SECONDS)
            .then_some(Self(seconds))
            .ok_or(DomainError::InvalidValue("retention period"))
    }

    /// Returns the exact duration in seconds.
    #[must_use]
    pub const fn seconds(self) -> u64 {
        self.0
    }
}

/// Explicit retention policy for one class.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RetentionRule {
    /// Governed data class.
    pub data_class: DataClass,
    /// Maximum authorized retention.
    pub period: RetentionPeriod,
}

impl RetentionRule {
    /// Creates an explicit rule.
    #[must_use]
    pub const fn new(data_class: DataClass, period: RetentionPeriod) -> Self {
        Self { data_class, period }
    }
}

/// Participant consent entity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParticipantConsent {
    /// Pseudonymous participant identity.
    pub participant: ParticipantPseudonym,
    /// Exact authorized purpose.
    pub purpose: Purpose,
    /// Accepted consent text version.
    pub version: ConsentVersion,
    /// Exclusive validity endpoint supplied by a deterministic clock boundary.
    pub valid_until: u64,
    revoked_at: Option<u64>,
}

/// Space authorization entity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SpaceAuthorization {
    /// Exact authorized space.
    pub space: SpaceId,
}

/// Purpose grant entity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PurposeGrant {
    /// Exact purpose, immutable after granting.
    pub purpose: Purpose,
    /// Required participant set; additional people also require a new grant.
    pub participants: BTreeSet<ParticipantPseudonym>,
    /// Exclusive authorization endpoint.
    pub expires_at: u64,
}

/// Why a consent or authorization was revoked.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RevocationReason {
    /// A participant withdrew consent.
    ParticipantRequest,
    /// The authorized purpose ended.
    PurposeEnded,
    /// A policy or safety control required revocation.
    PolicyOrSafety,
}

/// Immutable revocation entity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Revocation {
    /// Affected participant, if participant-specific.
    pub participant: Option<ParticipantPseudonym>,
    /// Reason for revocation.
    pub reason: RevocationReason,
    /// Deterministic event time.
    pub revoked_at: u64,
}

/// Aggregate lifecycle state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthorizationState {
    /// Not yet granted.
    Draft,
    /// Granted and potentially usable after evaluation.
    Active,
    /// Terminally revoked.
    Revoked,
}

/// Complete local policy query; no ambient clock or identity source is consulted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuthorizationRequest {
    /// Space requesting capture.
    pub space: SpaceId,
    /// Every person currently in sensing scope.
    pub participants: BTreeSet<ParticipantPseudonym>,
    /// Exact requested purpose.
    pub purpose: Purpose,
    /// Policy version used by the caller.
    pub policy_version: PolicyVersion,
    /// Deterministic evaluation time.
    pub evaluated_at: u64,
}

/// Fail-closed result returned to capture and privileged-command boundaries.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PolicyDecision {
    /// Every invariant is currently satisfied.
    Authorized,
    /// Capture is denied for a stable reason.
    Denied(DenialReason),
}

/// Stable reasons for denied authorization.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DenialReason {
    /// No grant exists.
    NotGranted,
    /// Authorization has been irrevocably revoked.
    Revoked,
    /// Requested use is prohibited.
    ProhibitedPurpose,
    /// Space differs from the exact grant.
    SpaceMismatch,
    /// Purpose differs from the exact grant.
    PurposeMismatch,
    /// Policy version differs from the aggregate version.
    PolicyVersionMismatch,
    /// Present people differ from the exact consented set.
    ParticipantSetMismatch,
    /// Grant is expired.
    Expired,
    /// A required consent record is absent.
    ConsentMissing,
    /// A required consent is expired or revoked.
    ConsentInactive,
    /// A data class lacks an explicit retention rule.
    RetentionMissing,
}

/// Commands accepted by the aggregate/application boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GovernanceCommand {
    /// Bind this aggregate to an exact physical space.
    AuthorizeSpace(SpaceId),
    /// Record a participant's exact consent.
    RecordParticipantConsent(ParticipantConsent),
    /// Apply a per-class retention rule.
    ApplyRetentionPolicy(RetentionRule),
    /// Grant an exact purpose to an exact participant set.
    GrantSensingAuthorization(PurposeGrant),
    /// Revoke one participant's consent.
    RevokeConsent {
        /// Participant withdrawing consent.
        participant: ParticipantPseudonym,
        /// Revocation reason.
        reason: RevocationReason,
        /// Deterministic time.
        revoked_at: u64,
    },
    /// Record completion of local deletion propagation.
    RecordDeletionCompletion {
        /// Affected participant.
        participant: ParticipantPseudonym,
        /// Deterministic completion time.
        completed_at: u64,
    },
}

/// Payload-minimized events emitted for journaling and separated audit projections.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GovernanceEvent {
    /// Space was bound.
    SpaceAuthorized {
        /// Exact authorized space.
        space: SpaceId,
    },
    /// Consent was recorded without sensitive payloads.
    ParticipantConsentRecorded {
        /// Participant pseudonym.
        participant: ParticipantPseudonym,
        /// Exact purpose.
        purpose: Purpose,
        /// Consent validity endpoint.
        valid_until: u64,
    },
    /// A class-specific retention policy was applied.
    RetentionPolicyApplied {
        /// Governed class.
        data_class: DataClass,
    },
    /// Authorization became active.
    SensingAuthorizationGranted {
        /// Exact space.
        space: SpaceId,
        /// Exact purpose.
        purpose: Purpose,
        /// Validity endpoint.
        expires_at: u64,
    },
    /// Authorization became terminally revoked.
    SensingAuthorizationRevoked {
        /// Reason.
        reason: RevocationReason,
        /// Revocation time.
        revoked_at: u64,
    },
    /// Local deletion propagation must start without network availability.
    ParticipantDeletionRequested {
        /// Affected participant.
        participant: ParticipantPseudonym,
        /// Request time.
        requested_at: u64,
    },
    /// Local deletion propagation completed.
    ParticipantDeletionCompleted {
        /// Affected participant.
        participant: ParticipantPseudonym,
        /// Completion time.
        completed_at: u64,
    },
    /// A retention rule expired.
    RetentionExpired {
        /// Expired class.
        data_class: DataClass,
    },
}

/// Aggregate root and sole authority for sensing authorization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SensingAuthorization {
    id: SensingAuthorizationId,
    policy_version: PolicyVersion,
    state: AuthorizationState,
    space: Option<SpaceAuthorization>,
    consents: BTreeMap<ParticipantPseudonym, ParticipantConsent>,
    grant: Option<PurposeGrant>,
    retention: BTreeMap<DataClass, RetentionRule>,
    revocation: Option<Revocation>,
}

impl SensingAuthorization {
    /// Creates a capture-denied draft aggregate.
    #[must_use]
    pub fn new(id: SensingAuthorizationId, policy_version: PolicyVersion) -> Self {
        Self {
            id,
            policy_version,
            state: AuthorizationState::Draft,
            space: None,
            consents: BTreeMap::new(),
            grant: None,
            retention: BTreeMap::new(),
            revocation: None,
        }
    }

    /// Returns the aggregate identity.
    #[must_use]
    pub const fn id(&self) -> &SensingAuthorizationId {
        &self.id
    }

    /// Returns the monotonic aggregate state.
    #[must_use]
    pub const fn state(&self) -> AuthorizationState {
        self.state
    }

    /// Returns the exact bound space, or `None` while the draft is unbound.
    ///
    /// This read-only accessor lets application policy checks avoid reflecting
    /// over serialized aggregate state; it cannot widen or mutate the grant.
    #[must_use]
    pub fn space(&self) -> Option<&SpaceId> {
        self.space
            .as_ref()
            .map(|authorization| &authorization.space)
    }

    /// Binds the draft to one exact space.
    pub fn authorize_space(&mut self, space: SpaceId) -> Result<Vec<GovernanceEvent>, DomainError> {
        self.require_draft()?;
        self.space = Some(SpaceAuthorization {
            space: space.clone(),
        });
        Ok(vec![GovernanceEvent::SpaceAuthorized { space }])
    }

    /// Records purpose-specific, versioned, bounded participant consent.
    pub fn record_participant_consent(
        &mut self,
        participant: ParticipantPseudonym,
        purpose: Purpose,
        version: ConsentVersion,
        valid_until: u64,
        now: u64,
    ) -> Result<Vec<GovernanceEvent>, DomainError> {
        self.require_draft()?;
        reject_prohibited(purpose)?;
        if valid_until <= now {
            return Err(DomainError::InvalidTimeRange);
        }
        self.consents.insert(
            participant.clone(),
            ParticipantConsent {
                participant: participant.clone(),
                purpose,
                version,
                valid_until,
                revoked_at: None,
            },
        );
        Ok(vec![GovernanceEvent::ParticipantConsentRecorded {
            participant,
            purpose,
            valid_until,
        }])
    }

    /// Applies an explicit retention rule while the aggregate is a draft.
    pub fn apply_retention_policy(
        &mut self,
        rule: RetentionRule,
    ) -> Result<Vec<GovernanceEvent>, DomainError> {
        self.require_draft()?;
        self.retention.insert(rule.data_class, rule);
        Ok(vec![GovernanceEvent::RetentionPolicyApplied {
            data_class: rule.data_class,
        }])
    }

    /// Activates one immutable purpose/space/participant grant after validating all controls.
    pub fn grant(
        &mut self,
        space: SpaceId,
        purpose: Purpose,
        participants: BTreeSet<ParticipantPseudonym>,
        expires_at: u64,
        now: u64,
    ) -> Result<Vec<GovernanceEvent>, DomainError> {
        reject_prohibited(purpose)?;
        self.require_draft()?;
        if expires_at <= now {
            return Err(DomainError::InvalidTimeRange);
        }
        if participants.is_empty() {
            return Err(DomainError::ParticipantSetEmpty);
        }
        if self.space.as_ref().map(|bound| &bound.space) != Some(&space) {
            return Err(DomainError::SpaceNotAuthorized);
        }
        for class in DataClass::ALL {
            if !self.retention.contains_key(&class) {
                return Err(DomainError::RetentionMissing(class));
            }
        }
        for participant in &participants {
            let consent = self
                .consents
                .get(participant)
                .ok_or_else(|| DomainError::ConsentMissing(participant.clone()))?;
            if consent.purpose != purpose
                || consent.valid_until < expires_at
                || consent.valid_until <= now
                || consent.revoked_at.is_some()
            {
                return Err(DomainError::ConsentInactive(participant.clone()));
            }
        }
        self.grant = Some(PurposeGrant {
            purpose,
            participants,
            expires_at,
        });
        self.state = AuthorizationState::Active;
        Ok(vec![GovernanceEvent::SensingAuthorizationGranted {
            space,
            purpose,
            expires_at,
        }])
    }

    /// Evaluates current policy solely from explicit request values and aggregate state.
    #[must_use]
    pub fn evaluate(&self, request: &AuthorizationRequest) -> PolicyDecision {
        let deny = |reason| PolicyDecision::Denied(reason);
        if self.state == AuthorizationState::Revoked {
            return deny(DenialReason::Revoked);
        }
        if request.purpose.is_prohibited() {
            return deny(DenialReason::ProhibitedPurpose);
        }
        let Some(grant) = &self.grant else {
            return deny(DenialReason::NotGranted);
        };
        if self.state != AuthorizationState::Active {
            return deny(DenialReason::NotGranted);
        }
        if request.policy_version != self.policy_version {
            return deny(DenialReason::PolicyVersionMismatch);
        }
        if self.space.as_ref().map(|bound| &bound.space) != Some(&request.space) {
            return deny(DenialReason::SpaceMismatch);
        }
        if request.purpose != grant.purpose {
            return deny(DenialReason::PurposeMismatch);
        }
        if request.participants != grant.participants {
            return deny(DenialReason::ParticipantSetMismatch);
        }
        if request.evaluated_at >= grant.expires_at {
            return deny(DenialReason::Expired);
        }
        if DataClass::ALL
            .iter()
            .any(|class| !self.retention.contains_key(class))
        {
            return deny(DenialReason::RetentionMissing);
        }
        for participant in &grant.participants {
            let Some(consent) = self.consents.get(participant) else {
                return deny(DenialReason::ConsentMissing);
            };
            if consent.purpose != grant.purpose
                || consent.revoked_at.is_some()
                || request.evaluated_at >= consent.valid_until
            {
                return deny(DenialReason::ConsentInactive);
            }
        }
        PolicyDecision::Authorized
    }

    /// Revokes consent, immediately terminally revokes capture, and requests local deletion.
    pub fn revoke_consent(
        &mut self,
        participant: ParticipantPseudonym,
        reason: RevocationReason,
        now: u64,
    ) -> Result<Vec<GovernanceEvent>, DomainError> {
        if self.state == AuthorizationState::Revoked {
            return Err(DomainError::AuthorizationTerminal);
        }
        let consent = self
            .consents
            .get_mut(&participant)
            .ok_or_else(|| DomainError::ConsentMissing(participant.clone()))?;
        consent.revoked_at = Some(now);
        self.state = AuthorizationState::Revoked;
        self.revocation = Some(Revocation {
            participant: Some(participant.clone()),
            reason,
            revoked_at: now,
        });
        Ok(vec![
            GovernanceEvent::SensingAuthorizationRevoked {
                reason,
                revoked_at: now,
            },
            GovernanceEvent::ParticipantDeletionRequested {
                participant,
                requested_at: now,
            },
        ])
    }

    /// Records deletion completion without reactivating the terminal aggregate.
    pub fn record_deletion_completion(
        &self,
        participant: ParticipantPseudonym,
        completed_at: u64,
    ) -> Result<Vec<GovernanceEvent>, DomainError> {
        if self.state != AuthorizationState::Revoked {
            return Err(DomainError::DeletionNotRequested);
        }
        Ok(vec![GovernanceEvent::ParticipantDeletionCompleted {
            participant,
            completed_at,
        }])
    }

    fn require_draft(&self) -> Result<(), DomainError> {
        if self.state == AuthorizationState::Draft {
            Ok(())
        } else {
            Err(DomainError::AuthorizationTerminal)
        }
    }
}

fn reject_prohibited(purpose: Purpose) -> Result<(), DomainError> {
    if purpose.is_prohibited() {
        Err(DomainError::ProhibitedPurpose(purpose))
    } else {
        Ok(())
    }
}

/// Invariant failure returned without leaking participant-sensitive content to display text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    /// Invalid value object input.
    InvalidValue(&'static str),
    /// Purpose is prohibited and cannot be overridden.
    ProhibitedPurpose(Purpose),
    /// Expiry is not strictly later than the supplied deterministic time.
    InvalidTimeRange,
    /// The exact space was not authorized.
    SpaceNotAuthorized,
    /// No participants were supplied.
    ParticipantSetEmpty,
    /// A participant lacks consent.
    ConsentMissing(ParticipantPseudonym),
    /// A participant's consent cannot cover the requested grant.
    ConsentInactive(ParticipantPseudonym),
    /// A class lacks explicit retention.
    RetentionMissing(DataClass),
    /// Active or revoked scope cannot be mutated/re-granted.
    AuthorizationTerminal,
    /// Completion was attempted before a deletion request.
    DeletionNotRequested,
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue(name) => write!(formatter, "invalid {name}"),
            Self::ProhibitedPurpose(_) => formatter.write_str("purpose is prohibited"),
            Self::InvalidTimeRange => formatter.write_str("invalid time range"),
            Self::SpaceNotAuthorized => formatter.write_str("space is not authorized"),
            Self::ParticipantSetEmpty => formatter.write_str("participant set is empty"),
            Self::ConsentMissing(_) => formatter.write_str("required consent is missing"),
            Self::ConsentInactive(_) => formatter.write_str("required consent is inactive"),
            Self::RetentionMissing(_) => formatter.write_str("retention rule is missing"),
            Self::AuthorizationTerminal => formatter.write_str("authorization scope is terminal"),
            Self::DeletionNotRequested => formatter.write_str("deletion was not requested"),
        }
    }
}

impl Error for DomainError {}

/// Marker retained for architecture boundary tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainBoundary;
