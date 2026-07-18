//! Authenticated operator boundaries for the local STE process.

use std::fmt;
use std::path::{Path, PathBuf};

use ste_consent_governance::domain::{AuthorizationRequest, PolicyDecision};
use ste_observability::{HealthSnapshot, SupportBundleBuilder};
use ste_radio_acquisition::replay::{ReplayLimits, ReplayReport, parse_pcap, parse_rvcsi};
use ste_runtime::{GovernanceGate, PrivilegedCommand, RequestOrigin};
use ste_signal_observation::dsp::{DspGraphSpec, PrimitiveCsiFrame};
use ste_signal_observation::{
    AlgorithmVersion, ContentAddressedEvidenceRepository, DspVersion, ObservationReplay,
    ObservationWindowId, PartitionRole, ReplayEvidenceFrame, WindowBounds, WindowPolicy,
};

/// Local payload-free diagnostics or support-preview request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticsCommand {
    /// Stable local health snapshot.
    Health,
    /// Exact manifest preview; no bundle export.
    SupportPreview,
}

impl DiagnosticsCommand {
    /// Parses `health` or `support preview`.
    pub fn parse<I, S>(arguments: I) -> Result<Self, DiagnosticsError>
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
            ["health"] | ["health", "--json"] => Ok(Self::Health),
            ["support", "preview"] | ["support", "preview", "--json"] => Ok(Self::SupportPreview),
            _ => Err(DiagnosticsError::InvalidArguments),
        }
    }
}

/// Injected diagnostics boundary.
pub trait DiagnosticsOperations {
    /// Returns stable JSON health without governed payload fields.
    fn health_json(&self) -> Result<String, DiagnosticsError>;
    /// Returns only the exact redacted support manifest preview.
    fn support_preview_json(&self) -> Result<String, DiagnosticsError>;
}

/// Concrete adapter over the local Rust observability APIs.
pub struct LocalDiagnostics<'a> {
    health: &'a HealthSnapshot,
    support: &'a SupportBundleBuilder<'a>,
}

impl<'a> LocalDiagnostics<'a> {
    /// Composes snapshot and preview builder without export authority.
    #[must_use]
    pub const fn new(health: &'a HealthSnapshot, support: &'a SupportBundleBuilder<'a>) -> Self {
        Self { health, support }
    }
}

impl DiagnosticsOperations for LocalDiagnostics<'_> {
    fn health_json(&self) -> Result<String, DiagnosticsError> {
        serde_json::to_string(self.health).map_err(|_| DiagnosticsError::Encoding)
    }

    fn support_preview_json(&self) -> Result<String, DiagnosticsError> {
        let preview = self
            .support
            .preview()
            .map_err(|_| DiagnosticsError::Encoding)?;
        serde_json::to_string(&preview.manifest).map_err(|_| DiagnosticsError::Encoding)
    }
}

/// Stable diagnostics boundary failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticsError {
    /// Unsupported arguments.
    InvalidArguments,
    /// Current policy denied the request.
    AuthorizationRequired,
    /// Redacted JSON/manifest encoding failed.
    Encoding,
}

/// Executes diagnostics only after a fresh governance decision.
pub fn execute_diagnostics<E, D>(
    command: DiagnosticsCommand,
    request: &AuthorizationRequest,
    origin: RequestOrigin,
    gate: &GovernanceGate<E>,
    diagnostics: &D,
) -> Result<String, DiagnosticsError>
where
    E: Fn(&AuthorizationRequest) -> PolicyDecision,
    D: DiagnosticsOperations,
{
    let privileged = match command {
        DiagnosticsCommand::Health => PrivilegedCommand::ViewDiagnostics,
        DiagnosticsCommand::SupportPreview => PrivilegedCommand::PreviewSupportBundle,
    };
    gate.authorize_command(request, origin, privileged)
        .map_err(|_| DiagnosticsError::AuthorizationRequired)?;
    match command {
        DiagnosticsCommand::Health => diagnostics.health_json(),
        DiagnosticsCommand::SupportPreview => diagnostics.support_preview_json(),
    }
}
use ste_storage::lifecycle::LifecycleManager;
use ste_storage::{DataClass, EventUpcaster, JournalStore};

/// Supported deterministic offline capture framing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayFormat {
    /// STE's bounded rvCSI interchange framing.
    Rvcsi,
    /// Classic PCAP containing bounded rvCSI records.
    Pcap,
}

/// Authorized replay request parsed from `ste replay` arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCommand {
    /// Governed local capture path.
    pub input: PathBuf,
    /// Explicit parser selection.
    pub format: ReplayFormat,
    /// Stable machine-readable output request.
    pub json: bool,
}

impl ReplayCommand {
    /// Parses `<path> --format rvcsi|pcap [--json]`.
    pub fn parse<I, S>(arguments: I) -> Result<Self, ReplayCommandError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args = arguments
            .into_iter()
            .map(|value| value.as_ref().to_owned())
            .collect::<Vec<_>>();
        if args.len() < 3 || args.get(1).map(String::as_str) != Some("--format") {
            return Err(ReplayCommandError::InvalidArguments);
        }
        let format = match args[2].as_str() {
            "rvcsi" => ReplayFormat::Rvcsi,
            "pcap" => ReplayFormat::Pcap,
            _ => return Err(ReplayCommandError::InvalidArguments),
        };
        if args.len() > 4 || (args.len() == 4 && args[3] != "--json") {
            return Err(ReplayCommandError::InvalidArguments);
        }
        let input = PathBuf::from(&args[0]);
        if input.as_os_str().is_empty() {
            return Err(ReplayCommandError::InvalidArguments);
        }
        Ok(Self {
            input,
            format,
            json: args.get(3).is_some_and(|value| value == "--json"),
        })
    }
}

/// Injected bounded file reader for replay input.
pub trait ReplayInput {
    /// Reads no more than the supplied byte limit.
    fn read_bounded(&self, path: &Path, maximum: usize) -> Result<Vec<u8>, ReplayCommandError>;
}

/// Local filesystem implementation with pre/post read bounds.
pub struct FilesystemReplayInput;

impl ReplayInput for FilesystemReplayInput {
    fn read_bounded(&self, path: &Path, maximum: usize) -> Result<Vec<u8>, ReplayCommandError> {
        let metadata = std::fs::metadata(path).map_err(|_| ReplayCommandError::InputUnavailable)?;
        if metadata.len() > maximum as u64 {
            return Err(ReplayCommandError::InputTooLarge);
        }
        let bytes = std::fs::read(path).map_err(|_| ReplayCommandError::InputUnavailable)?;
        if bytes.len() > maximum {
            return Err(ReplayCommandError::InputTooLarge);
        }
        Ok(bytes)
    }
}

/// Stable replay outcome preserving every rejection and sequence gap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplaySummary {
    /// Structurally accepted frames.
    pub accepted: u64,
    /// Non-finite rejected records.
    pub rejected_non_finite: u64,
    /// Implausible rejected records.
    pub rejected_implausible: u64,
    /// Malformed rejected records.
    pub rejected_malformed: u64,
    /// Missing sequence values.
    pub missing: u64,
}

impl From<&ReplayReport> for ReplaySummary {
    fn from(report: &ReplayReport) -> Self {
        Self {
            accepted: report.frames.len() as u64,
            rejected_non_finite: report.rejected_non_finite,
            rejected_implausible: report.rejected_implausible,
            rejected_malformed: report.rejected_malformed,
            missing: report.gaps.iter().map(|gap| gap.missing).sum(),
        }
    }
}

/// Replay failure without capture contents or paths in its display.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayCommandError {
    /// CLI syntax is invalid.
    InvalidArguments,
    /// Current policy denied governed replay access.
    AuthorizationRequired,
    /// Capture could not be read.
    InputUnavailable,
    /// Capture exceeds the bounded parser budget.
    InputTooLarge,
    /// Capture framing or records failed bounded parsing.
    InvalidCapture,
}

/// Rechecks governance before reading or parsing governed capture bytes.
pub fn execute_replay<E, R>(
    command: &ReplayCommand,
    request: &AuthorizationRequest,
    origin: RequestOrigin,
    gate: &GovernanceGate<E>,
    input: &R,
) -> Result<ReplaySummary, ReplayCommandError>
where
    E: Fn(&AuthorizationRequest) -> PolicyDecision,
    R: ReplayInput,
{
    gate.authorize_command(request, origin, PrivilegedCommand::ReplayCapture)
        .map_err(|_| ReplayCommandError::AuthorizationRequired)?;
    let limits = ReplayLimits::default();
    let bytes = input.read_bounded(&command.input, limits.max_input_bytes)?;
    let report = match command.format {
        ReplayFormat::Rvcsi => parse_rvcsi(&bytes, limits),
        ReplayFormat::Pcap => parse_pcap(&bytes, limits),
    }
    .map_err(|_| ReplayCommandError::InvalidCapture)?;
    Ok(ReplaySummary::from(&report))
}

/// Observation replay request over a governed radio capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationReplayCommand {
    /// Underlying bounded radio replay request.
    pub radio: ReplayCommand,
    /// Non-production dataset partition role.
    pub partition: PartitionRole,
}

impl ObservationReplayCommand {
    /// Parses `<path> --format rvcsi|pcap --partition development|validation|test`.
    pub fn parse<I, S>(arguments: I) -> Result<Self, ReplayCommandError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args = arguments
            .into_iter()
            .map(|value| value.as_ref().to_owned())
            .collect::<Vec<_>>();
        if args.len() != 5 || args[3] != "--partition" {
            return Err(ReplayCommandError::InvalidArguments);
        }
        let partition = match args[4].as_str() {
            "development" => PartitionRole::Development,
            "validation" => PartitionRole::Validation,
            "test" => PartitionRole::Test,
            _ => return Err(ReplayCommandError::InvalidArguments),
        };
        let radio = ReplayCommand::parse(&args[..3])?;
        Ok(Self { radio, partition })
    }
}

/// Immutable observation replay result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationReplayResult {
    /// Content digest of the stored artifact.
    pub artifact_digest: String,
    /// Count of radio frames contributing source references.
    pub source_frames: usize,
}

/// Runs the pinned signal-only graph and idempotently stores its artifact.
pub fn execute_observation_replay<E, R>(
    command: &ObservationReplayCommand,
    request: &AuthorizationRequest,
    origin: RequestOrigin,
    gate: &GovernanceGate<E>,
    input: &R,
    repository: &ContentAddressedEvidenceRepository,
) -> Result<ObservationReplayResult, ReplayCommandError>
where
    E: Fn(&AuthorizationRequest) -> PolicyDecision,
    R: ReplayInput,
{
    gate.authorize_command(request, origin, PrivilegedCommand::ReplayCapture)
        .map_err(|_| ReplayCommandError::AuthorizationRequired)?;
    let limits = ReplayLimits::default();
    let bytes = input.read_bounded(&command.radio.input, limits.max_input_bytes)?;
    let report = match command.radio.format {
        ReplayFormat::Rvcsi => parse_rvcsi(&bytes, limits),
        ReplayFormat::Pcap => parse_pcap(&bytes, limits),
    }
    .map_err(|_| ReplayCommandError::InvalidCapture)?;
    if report.rejected_malformed > 0
        || report.rejected_implausible > 0
        || report.rejected_non_finite > 0
    {
        return Err(ReplayCommandError::InvalidCapture);
    }
    if report.frames.len() < 2 {
        return Err(ReplayCommandError::InvalidCapture);
    }
    let start = report
        .frames
        .first()
        .expect("checked non-empty")
        .event_time_ns;
    let end = report
        .frames
        .last()
        .expect("checked non-empty")
        .event_time_ns
        .checked_add(1)
        .ok_or(ReplayCommandError::InvalidCapture)?;
    let delta = report.frames[1].event_time_ns.saturating_sub(start);
    if delta == 0 {
        return Err(ReplayCommandError::InvalidCapture);
    }
    let sample_rate_hz = 1_000_000_000.0 / delta as f64;
    let frames = report
        .frames
        .iter()
        .map(|frame| {
            let source_ref = format!("radio-frame:{}", frame.sequence);
            ReplayEvidenceFrame {
                source_ref: source_ref.clone(),
                frame: PrimitiveCsiFrame {
                    source_ref,
                    event_time_ns: frame.event_time_ns,
                    subcarriers: frame.subcarriers.clone(),
                },
            }
        })
        .collect::<Vec<_>>();
    let artifact = ObservationReplay::replay(
        ObservationWindowId::new("cli-observation-replay")
            .map_err(|_| ReplayCommandError::InvalidCapture)?,
        WindowBounds::new(start, end).map_err(|_| ReplayCommandError::InvalidCapture)?,
        WindowPolicy::new("cli-fixed-v1", 2, frames.len(), 0.2, 0.2)
            .map_err(|_| ReplayCommandError::InvalidCapture)?,
        AlgorithmVersion::new("features-v1").map_err(|_| ReplayCommandError::InvalidCapture)?,
        DspVersion::new("dsp-v1").map_err(|_| ReplayCommandError::InvalidCapture)?,
        "radio-calibration-v1".into(),
        command.partition,
        DspGraphSpec {
            version: 1,
            sample_rate_hz,
            window_len: frames.len(),
            saturation_magnitude: 1.0e9,
            presence_threshold: 0.0,
            periodicity_min_lag: 1,
            periodicity_max_lag: (frames.len() - 1).min(64),
        },
        &frames,
    )
    .map_err(|_| ReplayCommandError::InvalidCapture)?;
    repository
        .put(&artifact)
        .map_err(|_| ReplayCommandError::InvalidCapture)?;
    Ok(ObservationReplayResult {
        artifact_digest: artifact.digest().into(),
        source_frames: frames.len(),
    })
}

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

/// Governed validation-study operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationCommand {
    /// Validate a serialized dataset manifest and its locked split.
    ValidateDataset {
        /// Governed manifest path.
        input: PathBuf,
    },
    /// Export a deidentified, immutable validation report.
    Export {
        /// Opaque frozen-study identifier.
        study_id: String,
    },
    /// Promote a capability using evidence already verified by the service.
    Promote {
        /// Opaque frozen-study identifier.
        study_id: String,
        /// Versioned capability identifier.
        capability: String,
    },
    /// Preserve an explicit negative decision and reason.
    Reject {
        /// Opaque frozen-study identifier.
        study_id: String,
        /// Versioned capability identifier.
        capability: String,
        /// Mandatory-gate rejection reason.
        reason: String,
    },
}

impl ValidationCommand {
    /// Parses the narrow validation command surface.
    pub fn parse<I, S>(arguments: I) -> Result<Self, ValidationCommandError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args = arguments
            .into_iter()
            .map(|v| v.as_ref().to_owned())
            .collect::<Vec<_>>();
        let required = |value: &String| !value.trim().is_empty() && value.len() <= 256;
        match args.as_slice() {
            [verb, input] if verb == "validate-dataset" && required(input) => {
                Ok(Self::ValidateDataset {
                    input: PathBuf::from(input),
                })
            }
            [verb, study] if verb == "export" && required(study) => Ok(Self::Export {
                study_id: study.clone(),
            }),
            [verb, study, capability]
                if verb == "promote" && required(study) && required(capability) =>
            {
                Ok(Self::Promote {
                    study_id: study.clone(),
                    capability: capability.clone(),
                })
            }
            [verb, study, capability, reason]
                if verb == "reject"
                    && required(study)
                    && required(capability)
                    && required(reason) =>
            {
                Ok(Self::Reject {
                    study_id: study.clone(),
                    capability: capability.clone(),
                    reason: reason.clone(),
                })
            }
            _ => Err(ValidationCommandError::InvalidArguments),
        }
    }
}

/// Application service behind the authenticated local operator boundary.
pub trait ValidationOperations {
    /// Validates manifest completeness and cross-partition leakage.
    fn validate_dataset(&self, input: &Path) -> Result<String, ValidationCommandError>;
    /// Produces a deidentified report with an immutable evidence digest.
    fn export(&self, study_id: &str) -> Result<String, ValidationCommandError>;
    /// Promotes only after the service verifies immutable passing evidence.
    fn promote(&self, study_id: &str, capability: &str) -> Result<String, ValidationCommandError>;
    /// Appends an immutable rejection, including negative evidence.
    fn reject(
        &self,
        study_id: &str,
        capability: &str,
        reason: &str,
    ) -> Result<String, ValidationCommandError>;
}

/// Stable fail-closed validation command error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationCommandError {
    /// Syntax was incomplete or ambiguous.
    InvalidArguments,
    /// A fresh exact authorization was not granted.
    AuthorizationRequired,
    /// Dataset, evidence, or decision failed validation.
    ValidationFailed,
}

/// Reauthorizes immediately before every validation operation.
pub fn execute_validation_command<E, V>(
    command: &ValidationCommand,
    request: &AuthorizationRequest,
    origin: RequestOrigin,
    gate: &GovernanceGate<E>,
    operations: &V,
) -> Result<String, ValidationCommandError>
where
    E: Fn(&AuthorizationRequest) -> PolicyDecision,
    V: ValidationOperations,
{
    let privileged = match command {
        ValidationCommand::ValidateDataset { .. } | ValidationCommand::Export { .. } => {
            PrivilegedCommand::AccessValidationEvidence
        }
        ValidationCommand::Promote { .. } => PrivilegedCommand::PromoteValidatedCapability,
        ValidationCommand::Reject { .. } => PrivilegedCommand::RejectValidatedCapability,
    };
    gate.authorize_command(request, origin, privileged)
        .map_err(|_| ValidationCommandError::AuthorizationRequired)?;
    match command {
        ValidationCommand::ValidateDataset { input } => operations.validate_dataset(input),
        ValidationCommand::Export { study_id } => operations.export(study_id),
        ValidationCommand::Promote {
            study_id,
            capability,
        } => operations.promote(study_id, capability),
        ValidationCommand::Reject {
            study_id,
            capability,
            reason,
        } => operations.reject(study_id, capability, reason),
    }
}

/// Governed, non-medical respiration validation query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RespirationCommand {
    /// Reports exact promotion state; never returns an estimate.
    Status {
        /// Exact model package identifier.
        model_id: String,
    },
    /// Runs the immutable scientific/resource gate report.
    Validate {
        /// Exact frozen validation run identifier.
        run_id: String,
    },
}

impl RespirationCommand {
    /// Parses `status <model-id>` or `validate <run-id>`.
    pub fn parse<I, S>(arguments: I) -> Result<Self, RespirationCommandError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args = arguments
            .into_iter()
            .map(|value| value.as_ref().to_owned())
            .collect::<Vec<_>>();
        match args.as_slice() {
            [verb, id] if verb == "status" && !id.trim().is_empty() && id.len() <= 256 => {
                Ok(Self::Status {
                    model_id: id.clone(),
                })
            }
            [verb, id] if verb == "validate" && !id.trim().is_empty() && id.len() <= 256 => {
                Ok(Self::Validate { run_id: id.clone() })
            }
            _ => Err(RespirationCommandError::InvalidArguments),
        }
    }
}

/// Read-only validation service; it cannot override the promotion registry.
pub trait RespirationOperations {
    /// Returns explicit non-medical enabled/disabled state.
    fn status(&self, model_id: &str) -> Result<String, RespirationCommandError>;
    /// Returns the exact immutable gate result and available agreement evidence.
    fn validate(&self, run_id: &str) -> Result<String, RespirationCommandError>;
}

/// Stable respiration CLI failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RespirationCommandError {
    /// Command syntax invalid.
    InvalidArguments,
    /// Fresh exact authorization absent.
    AuthorizationRequired,
    /// Immutable report unavailable or invalid.
    ValidationUnavailable,
}

/// Authorizes immediately before dispatching a non-medical respiration query.
pub fn execute_respiration_command<E, R>(
    command: &RespirationCommand,
    request: &AuthorizationRequest,
    origin: RequestOrigin,
    gate: &GovernanceGate<E>,
    operations: &R,
) -> Result<String, RespirationCommandError>
where
    E: Fn(&AuthorizationRequest) -> PolicyDecision,
    R: RespirationOperations,
{
    gate.authorize_command(request, origin, PrivilegedCommand::AccessValidationEvidence)
        .map_err(|_| RespirationCommandError::AuthorizationRequired)?;
    match command {
        RespirationCommand::Status { model_id } => operations.status(model_id),
        RespirationCommand::Validate { run_id } => operations.validate(run_id),
    }
}
