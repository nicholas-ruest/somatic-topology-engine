#![forbid(unsafe_code)]
//! Signed edge-model packages, inference ports, and atomic registry lifecycle.

pub mod capability;
pub mod package;
pub mod registry;
pub mod uncertainty;

/// Runtime-neutral inference port implemented by verified Rust model engines.
pub trait InferenceModel {
    /// Engine-specific failure.
    type Error;
    /// Evaluates a finite ordered feature vector.
    fn infer(&self, features: &[f64]) -> Result<Vec<f64>, Self::Error>;
}
