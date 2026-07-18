//! Authenticated operator boundaries for the local STE process.

use std::fmt;

use ste_consent_governance::domain::{AuthorizationRequest, PolicyDecision};
use ste_runtime::{GovernanceGate, PrivilegedCommand, RequestOrigin};
use ste_storage::lifecycle::LifecycleManager;
use ste_storage::{DataClass, EventUpcaster, JournalStore};

/// Storage lifecycle command parsed from the supported operator surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageCommand {
    /// Verify journal metadata and checksums without exposing payloads.
    InspectJournal,
    /// Rebuild projections; dry-run is the default.
    RebuildProjections {
        /// Whether mutations are suppressed.
        dry_run: bool,
    },
    /// Create an encrypted portable export manifest.
    ExportManifest {
        /// Destination selected by the authenticated operator.
        output: String,
    },
    /// Recover to the last verified record; dry-run is the default.
    Recover {
        /// Whether mutations are suppressed.
        dry_run: bool,
    },
    /// Propagate participant deletion; dry-run is the default.
    DeleteParticipant {
        /// Pseudonymous participant selector.
        participant: String,
        /// Whether mutations are suppressed.
        dry_run: bool,
    },
    /// Cryptographically erase data and restore safe defaults.
    FactoryReset {
        /// Explicit destructive-operation confirmation.
        confirmed: bool,
    },
    /// Permanently retire device data and identity.
    Decommission {
        /// Explicit destructive-operation confirmation.
        confirmed: bool,
    },
}

impl StorageCommand {
    /// Parses arguments following `ste storage`. Mutating repair/deletion
    /// commands remain dry-run unless `--apply` is explicitly supplied.
    pub fn parse<I, S>(arguments: I) -> Result<Self, StorageCommandError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args = arguments
            .into_iter()
            .map(|value| value.as_ref().to_owned())
            .collect::<Vec<_>>();
        match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["journal", "inspect"] => Ok(Self::InspectJournal),
            ["journal", "rebuild"] => Ok(Self::RebuildProjections { dry_run: true }),
            ["journal", "rebuild", "--apply"] => Ok(Self::RebuildProjections { dry_run: false }),
            ["export", output] if !output.trim().is_empty() => Ok(Self::ExportManifest {
                output: (*output).to_owned(),
            }),
            ["recover"] => Ok(Self::Recover { dry_run: true }),
            ["recover", "--apply"] => Ok(Self::Recover { dry_run: false }),
            ["delete", participant] if !participant.trim().is_empty() => {
                Ok(Self::DeleteParticipant {
                    participant: (*participant).to_owned(),
                    dry_run: true,
                })
            }
            ["delete", participant, "--apply"] if !participant.trim().is_empty() => {
                Ok(Self::DeleteParticipant {
                    participant: (*participant).to_owned(),
                    dry_run: false,
                })
            }
            ["factory-reset"] => Ok(Self::FactoryReset { confirmed: false }),
            ["factory-reset", "--confirm"] => Ok(Self::FactoryReset { confirmed: true }),
            ["decommission"] => Ok(Self::Decommission { confirmed: false }),
            ["decommission", "--confirm"] => Ok(Self::Decommission { confirmed: true }),
            _ => Err(StorageCommandError::InvalidArguments),
        }
    }
}

/// Narrow adapter port implemented using `ste-storage` in the composition root.
pub trait StorageOperations {
    /// Verifies journal structure and checksums.
    fn inspect_journal(&self) -> Result<String, StorageCommandError>;
    /// Rebuilds or previews rebuilding derived projections.
    fn rebuild_projections(&self, dry_run: bool) -> Result<String, StorageCommandError>;
    /// Produces an encrypted portable export manifest.
    fn export_manifest(&self, output: &str) -> Result<String, StorageCommandError>;
    /// Recovers or previews recovery to the last verified state.
    fn recover(&self, dry_run: bool) -> Result<String, StorageCommandError>;
    /// Propagates or previews participant deletion.
    fn delete_participant(
        &self,
        participant: &str,
        dry_run: bool,
    ) -> Result<String, StorageCommandError>;
    /// Performs cryptographic erasure and restores capture-disabled defaults.
    fn factory_reset(&self) -> Result<String, StorageCommandError>;
    /// Retires device data, keys, and identity.
    fn decommission(&self) -> Result<String, StorageCommandError>;
}

/// Explicit service that assembles and persists an encrypted portable export.
/// Its composition owns the authorized manifest, plaintext source, key provider,
/// and destination handling; the CLI never invents those values.
pub trait EncryptedExportOperations {
    /// Writes an authenticated encrypted export to the approved destination.
    fn export_encrypted(&self, output: &str) -> Result<String, StorageCommandError>;
}

/// Concrete adapter over the Rust journal and lifecycle APIs. All policy input
/// is injected by the composition root rather than inferred from CLI flags.
pub struct SteStorageOperations<'a, J: ?Sized, X: ?Sized> {
    journal: &'a J,
    upcaster: &'a dyn EventUpcaster,
    data_class: DataClass,
    lifecycle: &'a LifecycleManager,
    exporter: &'a X,
    deletion_classes: &'a [DataClass],
    operation_time: u64,
}

impl<'a, J: ?Sized, X: ?Sized> SteStorageOperations<'a, J, X> {
    /// Creates an adapter from explicit, already-authorized storage context.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        journal: &'a J,
        upcaster: &'a dyn EventUpcaster,
        data_class: DataClass,
        lifecycle: &'a LifecycleManager,
        exporter: &'a X,
        deletion_classes: &'a [DataClass],
        operation_time: u64,
    ) -> Self {
        Self {
            journal,
            upcaster,
            data_class,
            lifecycle,
            exporter,
            deletion_classes,
            operation_time,
        }
    }
}

impl<J, X> StorageOperations for SteStorageOperations<'_, J, X>
where
    J: JournalStore + ?Sized,
    X: EncryptedExportOperations + ?Sized,
{
    fn inspect_journal(&self) -> Result<String, StorageCommandError> {
        let report = self
            .journal
            .inspect(self.data_class)
            .map_err(|_| StorageCommandError::StorageFailure)?;
        Ok(format!(
            "journal verified: records={}, last_sequence={}, torn_tail={}",
            report.verified_records,
            report
                .last_sequence
                .map_or_else(|| "none".into(), |value| value.to_string()),
            report.torn_tail
        ))
    }

    fn rebuild_projections(&self, dry_run: bool) -> Result<String, StorageCommandError> {
        if dry_run {
            return self
                .inspect_journal()
                .map(|report| format!("dry-run: {report}"));
        }
        let report = self
            .journal
            .rebuild(self.data_class, self.upcaster)
            .map_err(|_| StorageCommandError::StorageFailure)?;
        Ok(format!(
            "projection input rebuilt from {} verified records",
            report.records.len()
        ))
    }

    fn export_manifest(&self, output: &str) -> Result<String, StorageCommandError> {
        self.exporter.export_encrypted(output)
    }

    fn recover(&self, dry_run: bool) -> Result<String, StorageCommandError> {
        let report = self
            .journal
            .rebuild(self.data_class, self.upcaster)
            .map_err(|_| StorageCommandError::StorageFailure)?;
        Ok(format!(
            "{}recovery verified through sequence {}",
            if dry_run { "dry-run: " } else { "" },
            report
                .last_verified_sequence
                .map_or_else(|| "none".into(), |value| value.to_string())
        ))
    }

    fn delete_participant(
        &self,
        participant: &str,
        dry_run: bool,
    ) -> Result<String, StorageCommandError> {
        if dry_run {
            return Ok(format!(
                "dry-run: deletion would visit {} data classes",
                self.deletion_classes.len()
            ));
        }
        let receipt = self
            .lifecycle
            .delete_everywhere(participant, self.deletion_classes, self.operation_time)
            .map_err(|_| StorageCommandError::StorageFailure)?;
        Ok(format!(
            "deletion completed across {} store/class steps; cryptographic_erasure={}",
            receipt.steps.len(),
            receipt.cryptographic_erasure
        ))
    }

    fn factory_reset(&self) -> Result<String, StorageCommandError> {
        self.lifecycle
            .factory_reset()
            .map_err(|_| StorageCommandError::StorageFailure)?;
        Ok("factory reset complete; capture disabled".into())
    }

    fn decommission(&self) -> Result<String, StorageCommandError> {
        self.lifecycle
            .decommission()
            .map_err(|_| StorageCommandError::StorageFailure)?;
        Ok("device decommissioned".into())
    }
}

/// Fail-closed operator error; backend details and sensitive payloads are not
/// included in its stable display representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageCommandError {
    /// Current exact governance policy denied the operation.
    AuthorizationRequired,
    /// A destructive operation omitted explicit confirmation.
    ConfirmationRequired,
    /// Command arguments were incomplete, ambiguous, or unsafe.
    InvalidArguments,
    /// Storage failed without exposing sensitive internal details.
    StorageFailure,
}

impl fmt::Display for StorageCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::AuthorizationRequired => "active authorization required",
            Self::ConfirmationRequired => "explicit confirmation required",
            Self::InvalidArguments => "invalid storage command arguments",
            Self::StorageFailure => "storage operation failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for StorageCommandError {}

/// Evaluates governance immediately before dispatching one storage operation.
pub fn execute_storage_command<E, S>(
    command: StorageCommand,
    request: &AuthorizationRequest,
    origin: RequestOrigin,
    gate: &GovernanceGate<E>,
    storage: &S,
) -> Result<String, StorageCommandError>
where
    E: Fn(&AuthorizationRequest) -> PolicyDecision,
    S: StorageOperations,
{
    let privileged = match &command {
        StorageCommand::InspectJournal => PrivilegedCommand::InspectJournal,
        StorageCommand::RebuildProjections { .. } => PrivilegedCommand::RebuildProjection,
        StorageCommand::ExportManifest { .. } => PrivilegedCommand::ExportSensitiveData,
        StorageCommand::Recover { .. } => PrivilegedCommand::RecoverStorage,
        StorageCommand::DeleteParticipant { .. } => PrivilegedCommand::DeleteSensitiveData,
        StorageCommand::FactoryReset { confirmed: false }
        | StorageCommand::Decommission { confirmed: false } => {
            return Err(StorageCommandError::ConfirmationRequired);
        }
        StorageCommand::FactoryReset { confirmed: true } => PrivilegedCommand::FactoryReset,
        StorageCommand::Decommission { confirmed: true } => PrivilegedCommand::Decommission,
    };
    gate.authorize_command(request, origin, privileged)
        .map_err(|_| StorageCommandError::AuthorizationRequired)?;

    match command {
        StorageCommand::InspectJournal => storage.inspect_journal(),
        StorageCommand::RebuildProjections { dry_run } => storage.rebuild_projections(dry_run),
        StorageCommand::ExportManifest { output } => storage.export_manifest(&output),
        StorageCommand::Recover { dry_run } => storage.recover(dry_run),
        StorageCommand::DeleteParticipant {
            participant,
            dry_run,
        } => storage.delete_participant(&participant, dry_run),
        StorageCommand::FactoryReset { confirmed: true } => storage.factory_reset(),
        StorageCommand::Decommission { confirmed: true } => storage.decommission(),
        StorageCommand::FactoryReset { confirmed: false }
        | StorageCommand::Decommission { confirmed: false } => {
            Err(StorageCommandError::ConfirmationRequired)
        }
    }
}
