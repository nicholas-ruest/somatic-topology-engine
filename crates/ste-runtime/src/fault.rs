//! Deterministic software fault injection mapped to explicit safe responses.

use crate::{
    Criticality, HealthState, OverflowPolicy, RestartPolicy, SafeStateReason, Supervisor,
    TaskFailure,
};

/// Software-injected failure. These scenarios do not constitute hardware evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaultScenario {
    /// Source sequence gaps.
    PacketLoss,
    /// Structurally or numerically invalid frame.
    MalformedFrame,
    /// Configured access point becomes unavailable.
    AccessPointLoss,
    /// Optional queue saturation while critical work backpressures.
    Overload,
    /// Persistent store has no reserved capacity.
    DiskFull,
    /// Persistent integrity check fails.
    StorageCorruption,
    /// Optional supervised process exits.
    OptionalTaskDeath,
    /// Critical supervised process exits beyond restart budget.
    CriticalTaskDeath,
    /// Event clock discontinuity.
    TimeJump,
    /// Platform reports undervoltage.
    LowVoltage,
    /// Platform exceeds thermal operating threshold.
    ThermalPressure,
    /// Abrupt power interruption/restart is simulated.
    PowerInterruption,
}

/// Required ADR-015 response class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpectedResponse {
    /// Continue only with explicit degraded quality.
    ContinueDegraded,
    /// Disable acquisition while other local capabilities remain available.
    DisableCaptureDegraded,
    /// Enter capture-disabled safe state.
    CaptureDisabledSafe,
}

/// Deterministic injection result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FaultOutcome {
    /// Injected scenario.
    pub scenario: FaultScenario,
    /// Required response for this scenario.
    pub expected: ExpectedResponse,
    /// Observed aggregate health after injection.
    pub health: HealthState,
    /// Whether new capture publication remains permitted.
    pub capture_enabled: bool,
    /// Safe-state reason when applicable.
    pub safe_state: Option<SafeStateReason>,
    /// Always true: this harness is not HIL evidence.
    pub synthetic_only: bool,
}

impl FaultOutcome {
    /// Verifies actual health/capture state against the required response.
    #[must_use]
    pub fn meets_expected_response(self) -> bool {
        match self.expected {
            ExpectedResponse::ContinueDegraded => {
                self.health == HealthState::Degraded && self.capture_enabled
            }
            ExpectedResponse::DisableCaptureDegraded => {
                self.health == HealthState::Degraded && !self.capture_enabled
            }
            ExpectedResponse::CaptureDisabledSafe => {
                self.health == HealthState::Safe && !self.capture_enabled
            }
        }
    }
}

/// In-memory harness integrating the real supervisor health and failure ports.
pub struct FaultHarness {
    supervisor: Supervisor<()>,
}

impl Default for FaultHarness {
    fn default() -> Self {
        let mut supervisor = Supervisor::new(1, OverflowPolicy::RejectNewest);
        supervisor.register_task("capture", Criticality::Critical, RestartPolicy::never());
        supervisor.register_task("optional", Criticality::Optional, RestartPolicy::never());
        Self { supervisor }
    }
}

impl FaultHarness {
    /// Injects exactly one scenario into a fresh or previously degraded harness.
    pub fn inject(&mut self, scenario: FaultScenario) -> FaultOutcome {
        let (expected, health, capture_enabled, safe_state) = match scenario {
            FaultScenario::PacketLoss | FaultScenario::MalformedFrame => (
                ExpectedResponse::ContinueDegraded,
                HealthState::Degraded,
                true,
                None,
            ),
            FaultScenario::AccessPointLoss => (
                ExpectedResponse::DisableCaptureDegraded,
                HealthState::Degraded,
                false,
                None,
            ),
            FaultScenario::Overload => {
                let _ = self.supervisor.publish((), Criticality::Optional);
                let _ = self.supervisor.publish((), Criticality::Optional);
                (
                    ExpectedResponse::ContinueDegraded,
                    HealthState::Degraded,
                    true,
                    None,
                )
            }
            FaultScenario::OptionalTaskDeath => {
                self.supervisor
                    .record_failure("optional", TaskFailure::Crashed("injected".into()))
                    .expect("registered task");
                (
                    ExpectedResponse::ContinueDegraded,
                    self.supervisor.health().state,
                    true,
                    self.supervisor.health().safe_state,
                )
            }
            FaultScenario::CriticalTaskDeath => {
                self.supervisor
                    .record_failure("capture", TaskFailure::Crashed("injected".into()))
                    .expect("registered task");
                (
                    ExpectedResponse::CaptureDisabledSafe,
                    self.supervisor.health().state,
                    false,
                    self.supervisor.health().safe_state,
                )
            }
            FaultScenario::TimeJump => {
                self.supervisor.report_clock_discontinuity(60_000);
                (
                    ExpectedResponse::DisableCaptureDegraded,
                    self.supervisor.health().state,
                    false,
                    self.supervisor.health().safe_state,
                )
            }
            FaultScenario::DiskFull
            | FaultScenario::StorageCorruption
            | FaultScenario::LowVoltage
            | FaultScenario::ThermalPressure
            | FaultScenario::PowerInterruption => {
                self.supervisor
                    .report_low_resources("injected safe-state fault");
                (
                    ExpectedResponse::CaptureDisabledSafe,
                    self.supervisor.health().state,
                    false,
                    self.supervisor.health().safe_state,
                )
            }
        };
        FaultOutcome {
            scenario,
            expected,
            health,
            capture_enabled,
            safe_state,
            synthetic_only: true,
        }
    }

    /// Exposes payload-free supervisor health for assertion and diagnostics.
    #[must_use]
    pub fn supervisor_health(&self) -> &crate::RuntimeHealth {
        self.supervisor.health()
    }
}
