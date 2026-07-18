//! Atomic validation persistence and reproducible de-identified reports.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::application::{EvidenceExportReader, ValidationStudyRepository};
use crate::domain::{FrozenStudy, PromotionDecision, StudyRun, ValidationStudy};

/// Atomic repository with immutable frozen/run/decision records.
#[derive(Default)]
pub struct AtomicValidationRepository {
    drafts: BTreeMap<String, Vec<u8>>,
    frozen: BTreeMap<String, Vec<u8>>,
    runs: BTreeMap<String, StoredRun>,
    promotions: Vec<StoredDecision>,
}

struct StoredRun {
    study_id: String,
    bytes: Vec<u8>,
}
struct StoredDecision {
    study_id: String,
    bytes: Vec<u8>,
}

impl AtomicValidationRepository {
    /// Number of immutable runs, including rejected/negative results.
    #[must_use]
    pub fn run_count(&self) -> usize {
        self.runs.len()
    }
    /// Number of append-only promotion/rejection decisions.
    #[must_use]
    pub fn promotion_count(&self) -> usize {
        self.promotions.len()
    }

    fn insert<T: Serialize>(
        map: &mut BTreeMap<String, Vec<u8>>,
        key: &str,
        value: &T,
    ) -> Result<(), RepositoryError> {
        let bytes = serde_json::to_vec(value).map_err(|_| RepositoryError::Encoding)?;
        match map.get(key) {
            Some(existing) if existing == &bytes => Ok(()),
            Some(_) => Err(RepositoryError::ImmutableConflict),
            None => {
                map.insert(key.to_owned(), bytes);
                Ok(())
            }
        }
    }
}

impl ValidationStudyRepository for AtomicValidationRepository {
    type Error = RepositoryError;

    fn save_draft(&mut self, study: &ValidationStudy) -> Result<(), Self::Error> {
        Self::insert(&mut self.drafts, study.id(), study)
    }
    fn save_frozen(&mut self, study: &FrozenStudy) -> Result<(), Self::Error> {
        Self::insert(&mut self.frozen, study.id(), study)
    }
    fn append_run(&mut self, run: &StudyRun) -> Result<(), Self::Error> {
        let bytes = serde_json::to_vec(run).map_err(|_| RepositoryError::Encoding)?;
        match self.runs.get(run.id()) {
            Some(existing) if existing.bytes == bytes => Ok(()),
            Some(_) => Err(RepositoryError::ImmutableConflict),
            None => {
                self.runs.insert(
                    run.id().to_owned(),
                    StoredRun {
                        study_id: run.study_id().to_owned(),
                        bytes,
                    },
                );
                Ok(())
            }
        }
    }
    fn append_promotion(&mut self, decision: &PromotionDecision) -> Result<(), Self::Error> {
        let stored = StoredDecision {
            study_id: decision.study_id().to_owned(),
            bytes: serde_json::to_vec(decision).map_err(|_| RepositoryError::Encoding)?,
        };
        if self
            .promotions
            .last()
            .is_some_and(|existing| existing.bytes == stored.bytes)
        {
            return Ok(());
        }
        self.promotions.push(stored);
        Ok(())
    }
}

impl EvidenceExportReader for AtomicValidationRepository {
    type Error = RepositoryError;
    type Export = DeidentifiedEvidenceExport;

    fn deidentified_export(&self, study_id: &str) -> Result<Self::Export, Self::Error> {
        if !self.frozen.contains_key(study_id) {
            return Err(RepositoryError::NotFound);
        }
        let runs = self
            .runs
            .values()
            .filter(|run| run.study_id == study_id)
            .map(|run| {
                serde_json::from_slice::<Value>(&run.bytes).map_err(|_| RepositoryError::Encoding)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let decisions = self
            .promotions
            .iter()
            .filter(|decision| decision.study_id == study_id)
            .map(|decision| {
                serde_json::from_slice::<Value>(&decision.bytes)
                    .map_err(|_| RepositoryError::Encoding)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(DeidentifiedEvidenceExport {
            version: 1,
            study_id: study_id.to_owned(),
            runs,
            decisions,
        })
    }
}

/// Read-only export containing run digests/results but no dataset records or grouping keys.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DeidentifiedEvidenceExport {
    /// Export schema.
    pub version: u16,
    /// Study identifier, not participant identity.
    pub study_id: String,
    /// Reproducibility digests and immutable positive/negative run results.
    pub runs: Vec<Value>,
    /// Append-only promotion/rejection history for this study.
    pub decisions: Vec<Value>,
}

/// Canonical report bytes and their SHA-256 content address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReproducibleValidationReport {
    bytes: Vec<u8>,
    digest: String,
}
impl ReproducibleValidationReport {
    /// Generates byte-identical JSON from byte-identical export evidence.
    pub fn generate(export: &DeidentifiedEvidenceExport) -> Result<Self, RepositoryError> {
        let bytes = serde_json::to_vec(export).map_err(|_| RepositoryError::Encoding)?;
        Ok(Self {
            digest: hex(&Sha256::digest(&bytes)),
            bytes,
        })
    }
    /// Immutable canonical JSON bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Lowercase SHA-256 digest.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
}
fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(output, "{byte:02x}").expect("String write cannot fail");
    }
    output
}

/// Payload-free repository failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepositoryError {
    /// Serialization failed.
    Encoding,
    /// Immutable identifier was reused with different content.
    ImmutableConflict,
    /// Requested frozen study was absent.
    NotFound,
}
impl fmt::Display for RepositoryError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(match self {
            Self::Encoding => "validation evidence encoding failed",
            Self::ImmutableConflict => "immutable validation evidence conflict",
            Self::NotFound => "validation study not found",
        })
    }
}
impl Error for RepositoryError {}
