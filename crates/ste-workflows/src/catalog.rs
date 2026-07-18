//! Closed catalog: workflow behavior cannot be supplied as browser scripts.
use serde::{Deserialize, Serialize};

/// Every privileged operation named by ADR-060.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowType {
    Commissioning,
    SiteAcceptance,
    Requalification,
    ConsentGrant,
    ConsentRenew,
    ConsentRevoke,
    ImmediateCaptureStop,
    Calibration,
    CaptureDiagnostics,
    ReplayDiagnostics,
    ModelRegister,
    ModelEvaluate,
    ModelPromote,
    ModelActivate,
    ModelHealth,
    ModelSuspend,
    ModelRollback,
    ModelRevoke,
    CapabilityInspect,
    CapabilityStage,
    CapabilityActivate,
    CapabilitySuspend,
    PersonalizationAnchor,
    PersonalizationCorrection,
    PersonalizationExport,
    PersonalizationDelete,
    PersonalizationErasure,
    PersonalizationRebuild,
    AdaptationPromote,
    AdaptationRollback,
    StudyValidate,
    DatasetValidate,
    ProtocolValidate,
    ValidationReportExport,
    NamedPromotion,
    NamedRejection,
    HardwareProbe,
    OledSimulate,
    RgbSimulate,
    TouchSimulate,
    PhysicalOff,
    PeripheralRecovery,
    SupportBundlePreview,
    SupportBundleExport,
    UpdateStage,
    UpdateActivate,
    UpdateHealth,
    UpdateRollback,
    KeyRotation,
    BackupVerify,
    RestoreVerify,
    DataDelete,
    Recovery,
    FactoryReset,
    Decommission,
    IncidentDeclare,
    EvidencePreserve,
    CapabilityEmergencySuspend,
    IncidentNotify,
    Recall,
    CapaTask,
}

impl WorkflowType {
    /// Resource class used for concurrency isolation.
    #[must_use]
    pub const fn lock_class(self) -> Option<&'static str> {
        match self {
            Self::UpdateActivate | Self::UpdateRollback => Some("update"),
            Self::RestoreVerify | Self::Recovery => Some("restore"),
            Self::FactoryReset | Self::Decommission => Some("destructive-device"),
            Self::ModelActivate | Self::ModelRollback => Some("active-model"),
            Self::Calibration => Some("calibration"),
            _ => None,
        }
    }

    /// Whether this operation needs an authoritative confirmation challenge.
    #[must_use]
    pub const fn destructive(self) -> bool {
        matches!(
            self,
            Self::ConsentRevoke
                | Self::PersonalizationDelete
                | Self::PersonalizationErasure
                | Self::ModelRevoke
                | Self::DataDelete
                | Self::FactoryReset
                | Self::Decommission
                | Self::KeyRotation
        )
    }

    /// Whether cancellation is safe before an external effect commits.
    #[must_use]
    pub const fn cancellable(self) -> bool {
        !matches!(
            self,
            Self::ImmediateCaptureStop | Self::PhysicalOff | Self::CapabilityEmergencySuspend
        )
    }
}
