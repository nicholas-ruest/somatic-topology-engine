//! Validation-study, dataset-governance, and capability-promotion domain model.

use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

/// Compatibility marker proving the public bounded-context layering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainBoundary;

fn required(value: impl Into<String>, label: &'static str) -> Result<String, DomainError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 256 {
        return Err(DomainError::InvalidValue(label));
    }
    Ok(value)
}

/// Stable content digest for immutable evidence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ArtifactDigest([u8; 32]);
impl ArtifactDigest {
    /// Wraps an already verified SHA-256 digest.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    /// Returns the digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Subject population governed by a study.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Cohort {
    /// Generated inputs, not human-validity evidence.
    Synthetic,
    /// Human participants requiring authority.
    Human,
}

/// Frozen preregistration definition and digest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Protocol {
    name: String,
    digest: ArtifactDigest,
}
impl Protocol {
    /// Creates a content-addressed protocol.
    pub fn new(name: impl Into<String>, digest: ArtifactDigest) -> Result<Self, DomainError> {
        Ok(Self {
            name: required(name, "protocol name")?,
            digest,
        })
    }
}

/// Ethics and consent authority for a human study.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuthorityEvidence {
    ethics_approval: String,
    consent_authority: String,
    jurisdiction: String,
    valid_from: u64,
    valid_until: u64,
}
impl AuthorityEvidence {
    /// Creates authority evidence with an exclusive expiry.
    pub fn new(
        ethics: impl Into<String>,
        consent: impl Into<String>,
        jurisdiction: impl Into<String>,
        valid_from: u64,
        valid_until: u64,
    ) -> Result<Self, DomainError> {
        if valid_from >= valid_until {
            return Err(DomainError::InvalidAuthorityWindow);
        }
        Ok(Self {
            ethics_approval: required(ethics, "ethics approval")?,
            consent_authority: required(consent, "consent authority")?,
            jurisdiction: required(jurisdiction, "jurisdiction")?,
            valid_from,
            valid_until,
        })
    }
}

/// Mutually exclusive dataset partition role.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum PartitionRole {
    /// Model fitting.
    Train,
    /// Engineering iteration.
    Development,
    /// Threshold calibration.
    Calibration,
    /// Locked confirmation.
    Test,
    /// Post-deployment evaluation.
    PostDeployment,
}

/// One indivisible dataset unit and all leakage grouping keys.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DatasetRecord {
    artifact_id: String,
    participant: String,
    session: String,
    room: String,
    day: String,
    role: PartitionRole,
}
impl DatasetRecord {
    /// Creates a partitioned artifact.
    pub fn new(
        artifact: impl Into<String>,
        participant: impl Into<String>,
        session: impl Into<String>,
        room: impl Into<String>,
        day: impl Into<String>,
        role: PartitionRole,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            artifact_id: required(artifact, "artifact identifier")?,
            participant: required(participant, "participant pseudonym")?,
            session: required(session, "session identifier")?,
            room: required(room, "room identifier")?,
            day: required(day, "collection day")?,
            role,
        })
    }
}

/// Enforced confirmatory split strategy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DatasetSplit {
    /// All four grouping keys are role-exclusive.
    ParticipantSessionRoomDay,
}

/// Complete immutable dataset lineage and governance manifest.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DatasetManifest {
    name: String,
    purpose: String,
    consent: String,
    license: String,
    acquisition_profile: String,
    transformations: String,
    retention: String,
    missingness: f64,
    digest: ArtifactDigest,
    records: Vec<DatasetRecord>,
}
impl DatasetManifest {
    /// Creates a complete manifest.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        purpose: impl Into<String>,
        consent: impl Into<String>,
        license: impl Into<String>,
        acquisition: impl Into<String>,
        transformations: impl Into<String>,
        retention: impl Into<String>,
        missingness: f64,
        digest: ArtifactDigest,
        records: Vec<DatasetRecord>,
    ) -> Result<Self, DomainError> {
        if !missingness.is_finite() || !(0.0..=1.0).contains(&missingness) {
            return Err(DomainError::InvalidMissingness);
        }
        if records.is_empty() {
            return Err(DomainError::EmptyDataset);
        }
        let unique = records
            .iter()
            .map(|r| &r.artifact_id)
            .collect::<BTreeSet<_>>();
        if unique.len() != records.len() {
            return Err(DomainError::DuplicateArtifact);
        }
        Ok(Self {
            name: required(name, "dataset name")?,
            purpose: required(purpose, "dataset purpose")?,
            consent: required(consent, "consent provenance")?,
            license: required(license, "dataset license")?,
            acquisition_profile: required(acquisition, "acquisition profile")?,
            transformations: required(transformations, "transformations")?,
            retention: required(retention, "retention policy")?,
            missingness,
            digest,
            records,
        })
    }

    /// Rejects any participant, session, room, or day crossing roles.
    pub fn validate_split(&self) -> Result<DatasetSplit, DomainError> {
        for (index, left) in self.records.iter().enumerate() {
            if self.records[index + 1..].iter().any(|right| {
                left.role != right.role
                    && (left.participant == right.participant
                        || left.session == right.session
                        || left.room == right.room
                        || left.day == right.day)
            }) {
                return Err(DomainError::SplitLeakage);
            }
        }
        Ok(DatasetSplit::ParticipantSessionRoomDay)
    }
}

/// Human-readable limitations bound to a manifest.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DatasetCard {
    manifest: DatasetManifest,
    population: String,
    limitations: String,
}
impl DatasetCard {
    /// Creates a complete card after validating its split.
    pub fn new(
        manifest: DatasetManifest,
        population: impl Into<String>,
        limitations: impl Into<String>,
    ) -> Result<Self, DomainError> {
        manifest.validate_split()?;
        Ok(Self {
            manifest,
            population: required(population, "population description")?,
            limitations: required(limitations, "dataset limitations")?,
        })
    }
    /// Returns the immutable manifest.
    #[must_use]
    pub const fn manifest(&self) -> &DatasetManifest {
        &self.manifest
    }
}

/// Editable validation aggregate before freeze.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ValidationStudy {
    id: String,
    protocol: Protocol,
    cohort: Cohort,
    dataset: Option<DatasetManifest>,
}
impl ValidationStudy {
    /// Registers a draft study.
    pub fn draft(
        id: impl Into<String>,
        protocol: Protocol,
        cohort: Cohort,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id: required(id, "study identifier")?,
            protocol,
            cohort,
            dataset: None,
        })
    }
    /// Returns the stable opaque study identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Adds a validated dataset while editable.
    pub fn with_dataset(mut self, dataset: DatasetManifest) -> Result<Self, DomainError> {
        dataset.validate_split()?;
        self.dataset = Some(dataset);
        Ok(self)
    }
    /// Irreversibly freezes protocol inputs.
    pub fn freeze(self, authority: Option<AuthorityEvidence>) -> Result<FrozenStudy, DomainError> {
        if self.cohort == Cohort::Human && authority.is_none() {
            return Err(DomainError::AuthorityRequired);
        }
        Ok(FrozenStudy {
            id: self.id,
            protocol: self.protocol,
            cohort: self.cohort,
            dataset: self.dataset,
            authority,
        })
    }
}

/// Immutable study definition capable of starting runs.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FrozenStudy {
    id: String,
    protocol: Protocol,
    cohort: Cohort,
    dataset: Option<DatasetManifest>,
    authority: Option<AuthorityEvidence>,
}
impl FrozenStudy {
    /// Stable study identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Starts a run bound to all reproducibility digests.
    pub fn start_run(
        &self,
        id: impl Into<String>,
        code: ArtifactDigest,
        firmware: ArtifactDigest,
        model: ArtifactDigest,
        environment: ArtifactDigest,
    ) -> Result<StudyRun, DomainError> {
        Ok(StudyRun {
            id: required(id, "run identifier")?,
            study_id: self.id.clone(),
            code,
            data: self.dataset.as_ref().map(|d| d.digest),
            firmware,
            model,
            calibration: self.protocol.digest,
            environment,
            result: None,
        })
    }
}

/// Confirmatory outcome, including negative evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum StudyResult {
    /// Mandatory gates passed.
    Passed {
        /// Content digest of the immutable result set.
        evidence: ArtifactDigest,
    },
    /// At least one mandatory gate failed.
    Rejected {
        /// Mandatory-gate failure reason.
        reason: String,
        /// Content digest of the immutable negative result set.
        evidence: ArtifactDigest,
    },
}
impl StudyResult {
    /// Creates a preserved negative result.
    pub fn rejected(
        reason: impl Into<String>,
        evidence: ArtifactDigest,
    ) -> Result<Self, DomainError> {
        Ok(Self::Rejected {
            reason: required(reason, "rejection reason")?,
            evidence,
        })
    }
}

/// Reproducible run whose result is assignable once.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StudyRun {
    id: String,
    study_id: String,
    code: ArtifactDigest,
    data: Option<ArtifactDigest>,
    firmware: ArtifactDigest,
    model: ArtifactDigest,
    calibration: ArtifactDigest,
    environment: ArtifactDigest,
    result: Option<StudyResult>,
}
impl StudyRun {
    /// Returns the stable opaque run identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Returns the immutable parent-study identifier.
    #[must_use]
    pub fn study_id(&self) -> &str {
        &self.study_id
    }
    /// Records a result exactly once.
    pub fn complete(mut self, result: StudyResult) -> Result<Self, DomainError> {
        if self.result.is_some() {
            return Err(DomainError::RunAlreadyCompleted);
        }
        self.result = Some(result);
        Ok(self)
    }
    /// Returns the completed result.
    #[must_use]
    pub fn result(&self) -> &StudyResult {
        self.result.as_ref().expect("completed run invariant")
    }
}

/// Immutable capability-promotion decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PromotionDecision {
    /// Capability promoted against exact evidence.
    Promoted {
        /// Versioned capability identifier.
        capability: String,
        /// Frozen study supporting the decision.
        study_id: String,
        /// Immutable passed-gate evidence.
        evidence: ArtifactDigest,
        /// Deterministic decision timestamp.
        decided_at: u64,
    },
    /// Capability rejected with negative evidence preserved.
    Rejected {
        /// Versioned capability identifier.
        capability: String,
        /// Frozen study supporting the decision.
        study_id: String,
        /// Mandatory-gate failure reason.
        reason: String,
        /// Deterministic decision timestamp.
        decided_at: u64,
    },
}
impl PromotionDecision {
    /// Creates a promotion.
    pub fn promoted(
        capability: impl Into<String>,
        study: impl Into<String>,
        evidence: ArtifactDigest,
        decided_at: u64,
    ) -> Result<Self, DomainError> {
        Ok(Self::Promoted {
            capability: required(capability, "capability")?,
            study_id: required(study, "study identifier")?,
            evidence,
            decided_at,
        })
    }
    /// Creates a rejection.
    pub fn rejected(
        capability: impl Into<String>,
        study: impl Into<String>,
        reason: impl Into<String>,
        decided_at: u64,
    ) -> Result<Self, DomainError> {
        Ok(Self::Rejected {
            capability: required(capability, "capability")?,
            study_id: required(study, "study identifier")?,
            reason: required(reason, "rejection reason")?,
            decided_at,
        })
    }
    /// Returns the immutable study identifier supporting this decision.
    #[must_use]
    pub fn study_id(&self) -> &str {
        match self {
            Self::Promoted { study_id, .. } | Self::Rejected { study_id, .. } => study_id,
        }
    }
    fn capability(&self) -> &str {
        match self {
            Self::Promoted { capability, .. } | Self::Rejected { capability, .. } => capability,
        }
    }
    fn decided_at(&self) -> u64 {
        match self {
            Self::Promoted { decided_at, .. } | Self::Rejected { decided_at, .. } => *decided_at,
        }
    }
}

/// Append-only promotion history.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PromotionRegistry {
    decisions: Vec<PromotionDecision>,
}
impl PromotionRegistry {
    /// Appends without replacing prior evidence.
    pub fn append(&mut self, decision: PromotionDecision) -> Result<(), DomainError> {
        if self
            .decisions
            .last()
            .is_some_and(|last| last.decided_at() >= decision.decided_at())
        {
            return Err(DomainError::NonMonotonicDecision);
        }
        self.decisions.push(decision);
        Ok(())
    }
    /// Entire chronological history for a capability.
    #[must_use]
    pub fn history(&self, capability: &str) -> Vec<&PromotionDecision> {
        self.decisions
            .iter()
            .filter(|d| d.capability() == capability)
            .collect()
    }
}

/// Stable invariant failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    /// Required value invalid.
    InvalidValue(&'static str),
    /// Authority window invalid.
    InvalidAuthorityWindow,
    /// Human authority missing.
    AuthorityRequired,
    /// Missingness invalid.
    InvalidMissingness,
    /// Dataset empty.
    EmptyDataset,
    /// Artifact duplicate.
    DuplicateArtifact,
    /// Grouping key crossed roles.
    SplitLeakage,
    /// Completed result was edited.
    RunAlreadyCompleted,
    /// Decision time was not monotonic.
    NonMonotonicDecision,
}
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "experiment validation invariant failed: {self:?}")
    }
}
impl Error for DomainError {}
