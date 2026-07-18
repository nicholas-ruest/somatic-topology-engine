//! Deterministic, versioned DSP graph over primitive complex CSI samples.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::f64::consts::{PI, TAU};
use std::fmt;

/// Stable graph configuration. Changing algorithm semantics requires a new version.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct DspGraphSpec {
    /// Algorithm graph major version. Version 1 is the only accepted value.
    pub version: u16,
    /// Nominal frame rate used for gaps and periodicity.
    pub sample_rate_hz: f64,
    /// Expected exact window length.
    pub window_len: usize,
    /// Aggregate magnitude considered saturated.
    pub saturation_magnitude: f64,
    /// Minimum normalized amplitude score considered present.
    pub presence_threshold: f64,
    /// Smallest periodic lag in frames.
    pub periodicity_min_lag: usize,
    /// Largest periodic lag in frames.
    pub periodicity_max_lag: usize,
}

impl DspGraphSpec {
    /// Validates all resource and numerical bounds.
    pub fn validate(self) -> Result<Self, DspError> {
        if self.version != 1
            || !self.sample_rate_hz.is_finite()
            || self.sample_rate_hz <= 0.0
            || self.sample_rate_hz > 10_000.0
            || self.window_len < 2
            || self.window_len > 1_000_000
            || !self.saturation_magnitude.is_finite()
            || self.saturation_magnitude <= 0.0
            || !self.presence_threshold.is_finite()
            || !(0.0..=1.0).contains(&self.presence_threshold)
            || self.periodicity_min_lag == 0
            || self.periodicity_min_lag > self.periodicity_max_lag
            || self.periodicity_max_lag >= self.window_len
        {
            return Err(DspError::InvalidSpec);
        }
        Ok(self)
    }
}

/// Primitive-only frame; no rvCSI or acquisition implementation type crosses this port.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveCsiFrame {
    /// Immutable acquisition/provenance reference for this source frame.
    pub source_ref: String,
    /// Monotonic event time.
    pub event_time_ns: u64,
    /// Owned real/imaginary subcarrier pairs.
    pub subcarriers: Vec<(f64, f64)>,
}

/// Label-free observation features and data-quality evidence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DspObservation {
    /// Inclusive event-time window start.
    pub window_start_ns: u64,
    /// Inclusive event-time window end.
    pub window_end_ns: u64,
    /// Ordered immutable source references used by this window.
    pub source_refs: Vec<String>,
    /// Mean squared first difference of aggregate magnitude and unwrapped phase.
    pub motion_energy: f64,
    /// Bounded amplitude-derived evidence score, not a person identity.
    pub presence_score: f64,
    /// Maximum normalized autocorrelation in the configured lag band.
    pub periodicity: f64,
    /// Dominant periodic frequency when periodic evidence has non-zero energy.
    pub dominant_frequency_hz: Option<f64>,
    /// Least-squares aggregate magnitude slope per second.
    pub drift_per_second: f64,
    /// High-frequency residual energy divided by total centered energy.
    pub interference_ratio: f64,
    /// Missing nominal frame positions divided by expected positions.
    pub missingness: f64,
    /// Fraction of frames at or above saturation threshold.
    pub saturation_fraction: f64,
    /// Count of observed principal-phase wrap transitions.
    pub phase_wraps: u64,
    /// Count of valid supplied frames.
    pub observed_frames: u64,
    /// Count of inferred missing nominal frame positions.
    pub missing_frames: u64,
}

/// Explicit comparison tolerance for replay evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToleranceProfile {
    /// Absolute tolerance near zero.
    pub absolute: f64,
    /// Scale-relative tolerance.
    pub relative: f64,
}

impl ToleranceProfile {
    /// Same-architecture deterministic replay threshold.
    pub const STRICT_REPLAY: Self = Self {
        absolute: 1.0e-12,
        relative: 1.0e-10,
    };
    /// Allowed cross-architecture floating-point variation.
    pub const CROSS_ARCHITECTURE: Self = Self {
        absolute: 1.0e-9,
        relative: 1.0e-7,
    };

    /// Compares every numerical feature and exact counter.
    #[must_use]
    pub fn observations_match(self, left: &DspObservation, right: &DspObservation) -> bool {
        let close =
            |a: f64, b: f64| (a - b).abs() <= self.absolute + self.relative * a.abs().max(b.abs());
        close(left.motion_energy, right.motion_energy)
            && close(left.presence_score, right.presence_score)
            && close(left.periodicity, right.periodicity)
            && match (left.dominant_frequency_hz, right.dominant_frequency_hz) {
                (Some(left), Some(right)) => close(left, right),
                (None, None) => true,
                _ => false,
            }
            && close(left.drift_per_second, right.drift_per_second)
            && close(left.interference_ratio, right.interference_ratio)
            && close(left.missingness, right.missingness)
            && close(left.saturation_fraction, right.saturation_fraction)
            && left.phase_wraps == right.phase_wraps
            && left.observed_frames == right.observed_frames
            && left.missing_frames == right.missing_frames
            && left.window_start_ns == right.window_start_ns
            && left.window_end_ns == right.window_end_ns
            && left.source_refs == right.source_refs
    }
}

/// Executes DSP graph version 1 with deterministic iteration/reduction order.
pub fn execute_dsp(
    spec: DspGraphSpec,
    frames: &[PrimitiveCsiFrame],
) -> Result<DspObservation, DspError> {
    let source_refs = frames
        .iter()
        .map(|frame| frame.source_ref.clone())
        .collect::<Vec<_>>();
    execute_dsp_with_source_refs(spec, frames, &source_refs)
}

/// Executes the graph while preserving caller-supplied immutable source references.
/// This is the honest integration path into an `ObservationWindow` summary; it
/// does not fabricate per-frame domain evidence from window-level features.
pub fn execute_dsp_with_source_refs(
    spec: DspGraphSpec,
    frames: &[PrimitiveCsiFrame],
    source_refs: &[String],
) -> Result<DspObservation, DspError> {
    if source_refs.len() != frames.len()
        || source_refs
            .iter()
            .any(|reference| reference.trim().is_empty() || reference.len() > 256)
    {
        return Err(DspError::SourceReferenceMismatch);
    }
    execute_dsp_internal(spec, frames, source_refs.to_vec())
}

fn execute_dsp_internal(
    spec: DspGraphSpec,
    frames: &[PrimitiveCsiFrame],
    source_refs: Vec<String>,
) -> Result<DspObservation, DspError> {
    let spec = spec.validate()?;
    if frames.len() != spec.window_len {
        return Err(DspError::WindowShape);
    }
    let nominal_ns = 1_000_000_000.0 / spec.sample_rate_hz;
    let mut times = Vec::with_capacity(frames.len());
    let mut magnitudes = Vec::with_capacity(frames.len());
    let mut phases = Vec::with_capacity(frames.len());
    let mut wraps = 0_u64;
    let mut missing = 0_u64;
    let mut saturated = 0_u64;
    let mut previous_principal_phase: Option<f64> = None;
    let mut phase_offset = 0.0_f64;

    for (index, frame) in frames.iter().enumerate() {
        if frame.event_time_ns == 0 || frame.subcarriers.is_empty() {
            return Err(DspError::MalformedFrame);
        }
        if index > 0 {
            let previous = frames[index - 1].event_time_ns;
            if frame.event_time_ns <= previous {
                return Err(DspError::NonMonotonicTime);
            }
            let delta = (frame.event_time_ns - previous) as f64;
            let steps = (delta / nominal_ns).round().max(1.0) as u64;
            missing = missing.saturating_add(steps.saturating_sub(1));
        }
        let (magnitude, phase) = aggregate_complex(&frame.subcarriers)?;
        if magnitude >= spec.saturation_magnitude {
            saturated += 1;
        }
        if let Some(previous) = previous_principal_phase {
            let principal_delta = phase - previous;
            if principal_delta.abs() > PI {
                wraps += 1;
            }
            if principal_delta > PI {
                phase_offset -= TAU;
            } else if principal_delta < -PI {
                phase_offset += TAU;
            }
        }
        phases.push(phase + phase_offset);
        previous_principal_phase = Some(phase);
        times.push(frame.event_time_ns as f64 / 1_000_000_000.0);
        magnitudes.push(magnitude);
    }

    let amplitude_diff = mean_squared_difference(&magnitudes);
    let phase_diff = mean_squared_difference(&phases);
    let motion_energy = finite(amplitude_diff + phase_diff)?;
    let mean_magnitude = mean(&magnitudes);
    let presence_score = finite(mean_magnitude / (1.0 + mean_magnitude))?.clamp(0.0, 1.0);
    let (periodicity, dominant_lag) = maximum_autocorrelation(
        &magnitudes,
        spec.periodicity_min_lag,
        spec.periodicity_max_lag,
    );
    let drift_per_second = linear_slope(&times, &magnitudes);
    let interference_ratio = interference_ratio(&magnitudes);
    let observed = frames.len() as u64;
    let expected = observed.saturating_add(missing);
    let missingness = if expected == 0 {
        0.0
    } else {
        missing as f64 / expected as f64
    };
    let saturation_fraction = saturated as f64 / observed as f64;
    let result = DspObservation {
        window_start_ns: frames[0].event_time_ns,
        window_end_ns: frames[frames.len() - 1].event_time_ns,
        source_refs,
        motion_energy,
        presence_score: if presence_score >= spec.presence_threshold {
            presence_score
        } else {
            0.0
        },
        periodicity,
        dominant_frequency_hz: dominant_lag.map(|lag| spec.sample_rate_hz / lag as f64),
        drift_per_second,
        interference_ratio,
        missingness,
        saturation_fraction,
        phase_wraps: wraps,
        observed_frames: observed,
        missing_frames: missing,
    };
    if all_finite(&result) {
        Ok(result)
    } else {
        Err(DspError::NumericalFailure)
    }
}

fn aggregate_complex(samples: &[(f64, f64)]) -> Result<(f64, f64), DspError> {
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for &(sample_real, sample_imaginary) in samples {
        if !sample_real.is_finite() || !sample_imaginary.is_finite() {
            return Err(DspError::NonFiniteInput);
        }
        real += sample_real;
        imaginary += sample_imaginary;
    }
    let count = samples.len() as f64;
    let real = real / count;
    let imaginary = imaginary / count;
    Ok((
        finite(real.hypot(imaginary))?,
        finite(imaginary.atan2(real))?,
    ))
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn mean_squared_difference(values: &[f64]) -> f64 {
    values
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).powi(2))
        .sum::<f64>()
        / (values.len() - 1) as f64
}

fn maximum_autocorrelation(values: &[f64], minimum: usize, maximum: usize) -> (f64, Option<usize>) {
    let centered = values
        .iter()
        .map(|value| value - mean(values))
        .collect::<Vec<_>>();
    let energy = centered.iter().map(|value| value * value).sum::<f64>();
    if energy <= f64::EPSILON {
        return (0.0, None);
    }
    let mut best_score = 0.0_f64;
    let mut best_lag = None;
    for lag in minimum..=maximum {
        let score = centered[..centered.len() - lag]
            .iter()
            .zip(&centered[lag..])
            .map(|(left, right)| left * right)
            .sum::<f64>()
            / energy;
        if score > best_score {
            best_score = score;
            best_lag = Some(lag);
        }
    }
    (best_score.clamp(0.0, 1.0), best_lag)
}

fn linear_slope(times: &[f64], values: &[f64]) -> f64 {
    let mean_time = mean(times);
    let mean_value = mean(values);
    let numerator = times
        .iter()
        .zip(values)
        .map(|(time, value)| (time - mean_time) * (value - mean_value))
        .sum::<f64>();
    let denominator = times
        .iter()
        .map(|time| (time - mean_time).powi(2))
        .sum::<f64>();
    if denominator <= f64::EPSILON {
        0.0
    } else {
        numerator / denominator
    }
}

fn interference_ratio(values: &[f64]) -> f64 {
    let centered_energy = {
        let average = mean(values);
        values
            .iter()
            .map(|value| (value - average).powi(2))
            .sum::<f64>()
    };
    if centered_energy <= f64::EPSILON {
        return 0.0;
    }
    let high_frequency = values
        .windows(3)
        .map(|window| (window[2] - 2.0 * window[1] + window[0]).powi(2))
        .sum::<f64>();
    (high_frequency / (6.0 * centered_energy)).clamp(0.0, 1.0)
}

fn all_finite(value: &DspObservation) -> bool {
    [
        value.motion_energy,
        value.presence_score,
        value.periodicity,
        value.dominant_frequency_hz.unwrap_or(0.0),
        value.drift_per_second,
        value.interference_ratio,
        value.missingness,
        value.saturation_fraction,
    ]
    .iter()
    .all(|number| number.is_finite())
}

fn finite(value: f64) -> Result<f64, DspError> {
    value
        .is_finite()
        .then_some(value)
        .ok_or(DspError::NumericalFailure)
}

/// Stable DSP validation or numerical failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DspError {
    /// Graph version or parameter is unsupported.
    InvalidSpec,
    /// Window length differs from the declared bounded graph.
    WindowShape,
    /// Supplied provenance references do not map exactly one-to-one to frames.
    SourceReferenceMismatch,
    /// Frame has no samples or no timestamp.
    MalformedFrame,
    /// Event time repeated or moved backward.
    NonMonotonicTime,
    /// Input contains NaN or infinity.
    NonFiniteInput,
    /// A checked numerical operation produced a non-finite result.
    NumericalFailure,
}

impl fmt::Display for DspError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for DspError {}
