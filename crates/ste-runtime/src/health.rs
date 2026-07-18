//! Runtime health and fail-safe state reporting.
#![allow(missing_docs)]

/// Aggregate runtime health.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HealthState {
    /// Every registered capability is available.
    #[default]
    Healthy,
    /// A non-critical capability is unavailable or timing is suspect.
    Degraded,
    /// The runtime deliberately entered a fail-safe state.
    Safe,
}

/// Reason the runtime entered its safe state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafeStateReason {
    CoordinatedShutdown,
    CriticalTaskFailed,
    LowResources,
}

/// Bounded, payload-free operational health snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuntimeHealth {
    pub state: HealthState,
    pub safe_state: Option<SafeStateReason>,
    pub shed_optional_events: u64,
    pub shed_critical_events: u64,
    pub clock_discontinuities: u64,
    pub low_resource_events: u64,
    pub last_issue: Option<String>,
}
