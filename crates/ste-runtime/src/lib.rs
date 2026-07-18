//! STE process composition root.

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
