//! Private adapters exposed only through application-owned port types.
#![allow(missing_docs)]

use crate::application::{
    DiagnosticRecord, DomainAuditRecord, GovernanceRecords, ParticipantHistoryRecord,
    RepositoryError, SecurityRecord, SensingAuthorizationRepository,
};
use crate::domain::{SensingAuthorization, SensingAuthorizationId};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Default)]
pub struct InMemoryAuthorizationRepository {
    values: Mutex<HashMap<String, SensingAuthorization>>,
}

impl SensingAuthorizationRepository for InMemoryAuthorizationRepository {
    fn load(
        &self,
        id: &SensingAuthorizationId,
    ) -> Result<Option<SensingAuthorization>, RepositoryError> {
        Ok(self
            .values
            .lock()
            .map_err(|_| RepositoryError::Unavailable)?
            .get(id.as_str())
            .cloned())
    }

    fn save(&self, authorization: &SensingAuthorization) -> Result<(), RepositoryError> {
        self.values
            .lock()
            .map_err(|_| RepositoryError::Unavailable)?
            .insert(
                authorization.id().as_str().to_owned(),
                authorization.clone(),
            );
        Ok(())
    }
}

pub struct FileAuthorizationRepository {
    root: PathBuf,
}

impl FileAuthorizationRepository {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, RepositoryError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|_| RepositoryError::Unavailable)?;
        Ok(Self { root })
    }

    fn path(&self, id: &SensingAuthorizationId) -> PathBuf {
        self.root.join(hex_name(id.as_str())).with_extension("json")
    }
}

impl SensingAuthorizationRepository for FileAuthorizationRepository {
    fn load(
        &self,
        id: &SensingAuthorizationId,
    ) -> Result<Option<SensingAuthorization>, RepositoryError> {
        let path = self.path(id);
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(RepositoryError::Unavailable),
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| RepositoryError::Corrupt)
    }

    fn save(&self, authorization: &SensingAuthorization) -> Result<(), RepositoryError> {
        let bytes = serde_json::to_vec(authorization).map_err(|_| RepositoryError::Corrupt)?;
        let target = self.path(authorization.id());
        let temporary = target.with_extension("json.new");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temporary)
            .map_err(|_| RepositoryError::Unavailable)?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|_| RepositoryError::Unavailable)?;
        fs::rename(temporary, target).map_err(|_| RepositoryError::Unavailable)?;
        OpenOptions::new()
            .read(true)
            .open(&self.root)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| RepositoryError::Unavailable)
    }
}

fn hex_name(value: &str) -> String {
    value.as_bytes().iter().fold(
        String::with_capacity(value.len() * 2),
        |mut output, byte| {
            write!(output, "{byte:02x}").expect("writing to a String cannot fail");
            output
        },
    )
}

#[derive(Default)]
pub struct SeparatedInMemoryRecords {
    domain_audit: Mutex<Vec<DomainAuditRecord>>,
    participant_history: Mutex<Vec<ParticipantHistoryRecord>>,
    security: Mutex<Vec<SecurityRecord>>,
    diagnostics: Mutex<Vec<DiagnosticRecord>>,
}

impl SeparatedInMemoryRecords {
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        (
            self.domain_audit.lock().map_or(0, |v| v.len()),
            self.participant_history.lock().map_or(0, |v| v.len()),
            self.security.lock().map_or(0, |v| v.len()),
            self.diagnostics.lock().map_or(0, |v| v.len()),
        )
    }
}

impl GovernanceRecords for SeparatedInMemoryRecords {
    fn domain_audit(&self, record: DomainAuditRecord) {
        if let Ok(mut records) = self.domain_audit.lock() {
            records.push(record);
        }
    }
    fn participant_history(&self, record: ParticipantHistoryRecord) {
        if let Ok(mut records) = self.participant_history.lock() {
            records.push(record);
        }
    }
    fn security(&self, record: SecurityRecord) {
        if let Ok(mut records) = self.security.lock() {
            records.push(record);
        }
    }
    fn diagnostic(&self, record: DiagnosticRecord) {
        if let Ok(mut records) = self.diagnostics.lock() {
            records.push(record);
        }
    }
}

#[allow(dead_code)]
fn _is_directory(path: &Path) -> bool {
    path.is_dir()
}
