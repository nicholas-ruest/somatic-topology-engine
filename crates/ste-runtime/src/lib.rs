//! STE process composition root.

pub mod config;
pub mod fault;
mod governance_gate;
mod health;
pub mod slo;
mod supervisor;
mod synthetic;

pub use governance_gate::{
    GateError, GovernanceGate, PrivilegedCommand, PrivilegedGrant, RequestOrigin,
    SafeGovernanceState,
};
pub use health::{HealthState, RuntimeHealth, SafeStateReason};
pub use supervisor::{
    CancellationToken, ChannelError, CircuitState, Criticality, OverflowPolicy, RestartPolicy,
    Supervisor, SupervisorError, TaskFailure, TaskStatus,
};
pub use synthetic::{ClockOverflow, DeterministicClock, SyntheticEvent, SyntheticPipeline};

/// Context catalog used by diagnostics without introducing context-to-context
/// dependencies.
#[must_use]
pub fn bounded_contexts() -> [&'static str; 8] {
    [
        ste_radio_acquisition::CONTEXT_NAME,
        ste_signal_observation::CONTEXT_NAME,
        ste_physiology_estimation::CONTEXT_NAME,
        ste_state_inference::CONTEXT_NAME,
        ste_personalization_memory::CONTEXT_NAME,
        ste_experiment_validation::CONTEXT_NAME,
        ste_device_interaction::CONTEXT_NAME,
        ste_consent_governance::CONTEXT_NAME,
    ]
}
