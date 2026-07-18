//! Fail-closed runtime enforcement around sensing and privileged operations.

use std::cell::Cell;

use ste_consent_governance::domain::{AuthorizationRequest, DenialReason, PolicyDecision};

/// Origin of a request crossing a privileged runtime boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestOrigin {
    /// Layered configuration.
    Configuration,
    /// Signed feature/capability policy.
    FeaturePolicy,
    /// Authenticated administrator action.
    Administrator,
    /// In-process acquisition adapter.
    CaptureAdapter,
    /// Optional, unprivileged sidecar.
    Sidecar,
    /// Authenticated local operator boundary.
    LocalOperator,
}

/// Operations that require a fresh, exact policy decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivilegedCommand {
    /// Begin publishing capture frames.
    StartCapture,
    /// Change a live capture profile.
    ChangeCaptureProfile,
    /// Export governed sensing data.
    ExportSensitiveData,
    /// Delete governed sensing data.
    DeleteSensitiveData,
    /// Inspect journal integrity and metadata.
    InspectJournal,
    /// Rebuild derived projections from verified journal records.
    RebuildProjection,
    /// Recover storage to the last verified state.
    RecoverStorage,
    /// Erase governed data and restore capture-disabled defaults.
    FactoryReset,
    /// Erase keys/data and permanently retire the device identity.
    Decommission,
}

/// Safe participant-visible/runtime state of the governance gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafeGovernanceState {
    /// No operation has received an active exact authorization.
    Unauthorized(DenialReason),
    /// The most recent operation received a fresh policy authorization.
    Authorized,
}

/// Stable policy-enforcement failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateError {
    /// Governance denied the exact request.
    Denied(DenialReason),
}

/// Capability returned only after a fresh privileged-command decision.
#[derive(Debug, Eq, PartialEq)]
pub struct PrivilegedGrant {
    command: PrivilegedCommand,
    origin: RequestOrigin,
}

impl PrivilegedGrant {
    /// Authorized command.
    #[must_use]
    pub const fn command(&self) -> PrivilegedCommand {
        self.command
    }

    /// Auditable origin of the authorized command.
    #[must_use]
    pub const fn origin(&self) -> RequestOrigin {
        self.origin
    }
}

/// Runtime policy enforcement point. The evaluator is normally the governance
/// context's `PolicyDecisionPoint`; accepting a function also keeps boundary
/// tests independent of repository infrastructure.
pub struct GovernanceGate<E> {
    evaluate: E,
    safe_state: Cell<SafeGovernanceState>,
}

impl<E> GovernanceGate<E>
where
    E: Fn(&AuthorizationRequest) -> PolicyDecision,
{
    /// Creates a capture-disabled gate.
    #[must_use]
    pub fn new(evaluate: E) -> Self {
        Self {
            evaluate,
            safe_state: Cell::new(SafeGovernanceState::Unauthorized(DenialReason::NotGranted)),
        }
    }

    /// Publishes only after evaluating the complete request immediately before
    /// the sink call. No authorization result is cached across frames.
    pub fn publish_capture<T, P>(
        &self,
        request: &AuthorizationRequest,
        _origin: RequestOrigin,
        frame: T,
        publish: P,
    ) -> Result<(), GateError>
    where
        P: FnOnce(T),
    {
        self.require_authorized(request)?;
        publish(frame);
        Ok(())
    }

    /// Returns a non-cloneable command capability after a fresh exact policy
    /// decision. Callers cannot construct or reuse it for a different command.
    pub fn authorize_command(
        &self,
        request: &AuthorizationRequest,
        origin: RequestOrigin,
        command: PrivilegedCommand,
    ) -> Result<PrivilegedGrant, GateError> {
        self.require_authorized(request)?;
        Ok(PrivilegedGrant { command, origin })
    }

    /// Returns a payload-free state suitable for unauthorized UI/runtime views.
    #[must_use]
    pub fn safe_state(&self) -> SafeGovernanceState {
        self.safe_state.get()
    }

    fn require_authorized(&self, request: &AuthorizationRequest) -> Result<(), GateError> {
        // Defense in depth: even a defective or compromised adapter/PDP cannot
        // authorize a purpose the domain declares permanently prohibited.
        if request.purpose.is_prohibited() {
            return self.deny(DenialReason::ProhibitedPurpose);
        }
        match (self.evaluate)(request) {
            PolicyDecision::Authorized => {
                self.safe_state.set(SafeGovernanceState::Authorized);
                Ok(())
            }
            PolicyDecision::Denied(reason) => self.deny(reason),
        }
    }

    fn deny(&self, reason: DenialReason) -> Result<(), GateError> {
        self.safe_state
            .set(SafeGovernanceState::Unauthorized(reason));
        Err(GateError::Denied(reason))
    }
}
