//! Typed service-level objectives and deterministic regression evaluation.

use std::error::Error;
use std::fmt;

macro_rules! bounded_value {
    ($name:ident, $label:literal, $minimum:expr, $maximum:expr) => {
        #[doc = $label]
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
        pub struct $name(f64);

        impl $name {
            /// Creates a finite value within the unit's supported range.
            pub fn new(value: f64) -> Result<Self, SloError> {
                if value.is_finite() && ($minimum..=$maximum).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(SloError::InvalidMeasurement($label))
                }
            }

            /// Returns the value in the unit named by this type.
            #[must_use]
            pub const fn get(self) -> f64 {
                self.0
            }
        }
    };
}

bounded_value!(Ratio, "ratio", 0.0, 1.0);
bounded_value!(Milliseconds, "milliseconds", 0.0, f64::MAX);
bounded_value!(Percent, "percent", 0.0, 100.0);
bounded_value!(Kibibytes, "kibibytes", 0.0, f64::MAX);
bounded_value!(KibibytesPerHour, "kibibytes per hour", 0.0, f64::MAX);
bounded_value!(Celsius, "degrees Celsius", -100.0, 250.0);
bounded_value!(Watts, "watts", 0.0, 10_000.0);

/// Complete owned runtime SLO budget. Minimums represent required coverage;
/// all other fields are maximum allowed measurements.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SloBudgets {
    /// Required fraction of expected capture frames received.
    pub capture_continuity_min: Ratio,
    /// Required fraction of windows that pass validity gates.
    pub valid_window_coverage_min: Ratio,
    /// Maximum queue residence delay.
    pub queue_delay_max: Milliseconds,
    /// Maximum age of the displayed projection.
    pub projection_freshness_max: Milliseconds,
    /// Maximum cold startup duration.
    pub startup_max: Milliseconds,
    /// Maximum recovery duration after a recoverable failure.
    pub recovery_max: Milliseconds,
    /// Maximum sustained CPU utilization.
    pub cpu_max: Percent,
    /// Maximum resident memory high-water mark.
    pub rss_max: Kibibytes,
    /// Maximum persistent-storage growth rate.
    pub storage_growth_max: KibibytesPerHour,
    /// Maximum supported device temperature.
    pub temperature_max: Celsius,
    /// Maximum supported power draw.
    pub power_max: Watts,
}

/// Measurements corresponding one-to-one with [`SloBudgets`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SloMeasurements {
    /// Observed capture continuity.
    pub capture_continuity: Ratio,
    /// Observed valid-window coverage.
    pub valid_window_coverage: Ratio,
    /// Observed queue delay.
    pub queue_delay: Milliseconds,
    /// Observed projection freshness.
    pub projection_freshness: Milliseconds,
    /// Observed startup duration.
    pub startup: Milliseconds,
    /// Observed recovery duration.
    pub recovery: Milliseconds,
    /// Observed CPU utilization.
    pub cpu: Percent,
    /// Observed resident memory.
    pub rss: Kibibytes,
    /// Observed storage growth.
    pub storage_growth: KibibytesPerHour,
    /// Observed device temperature.
    pub temperature: Celsius,
    /// Observed power draw.
    pub power: Watts,
}

/// Stable metric identifier for reports and release evidence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SloMetric {
    /// Capture continuity ratio.
    CaptureContinuity,
    /// Valid-window coverage ratio.
    ValidWindowCoverage,
    /// Queue delay.
    QueueDelay,
    /// Projection age.
    ProjectionFreshness,
    /// Startup time.
    Startup,
    /// Recovery time.
    Recovery,
    /// CPU utilization.
    Cpu,
    /// Resident memory.
    Rss,
    /// Storage growth rate.
    StorageGrowth,
    /// Device temperature.
    Temperature,
    /// Device power draw.
    Power,
}

/// One deterministic budget evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SloResult {
    /// Evaluated metric.
    pub metric: SloMetric,
    /// Measurement in the metric's documented unit.
    pub measured: f64,
    /// Required minimum or allowed maximum.
    pub budget: f64,
    /// Whether the measurement meets the budget.
    pub passed: bool,
}

/// Complete report; all metrics must pass for release-gate success.
#[derive(Clone, Debug, PartialEq)]
pub struct SloReport {
    /// Fixed-order metric results.
    pub results: Vec<SloResult>,
}

impl SloReport {
    /// Returns true only when every owned objective passes.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.results.iter().all(|result| result.passed)
    }
}

impl SloBudgets {
    /// Evaluates every objective with inclusive boundaries.
    #[must_use]
    pub fn evaluate(self, measured: SloMeasurements) -> SloReport {
        let minimum = |metric, value: f64, budget: f64| SloResult {
            metric,
            measured: value,
            budget,
            passed: value >= budget,
        };
        let maximum = |metric, value: f64, budget: f64| SloResult {
            metric,
            measured: value,
            budget,
            passed: value <= budget,
        };
        SloReport {
            results: vec![
                minimum(
                    SloMetric::CaptureContinuity,
                    measured.capture_continuity.get(),
                    self.capture_continuity_min.get(),
                ),
                minimum(
                    SloMetric::ValidWindowCoverage,
                    measured.valid_window_coverage.get(),
                    self.valid_window_coverage_min.get(),
                ),
                maximum(
                    SloMetric::QueueDelay,
                    measured.queue_delay.get(),
                    self.queue_delay_max.get(),
                ),
                maximum(
                    SloMetric::ProjectionFreshness,
                    measured.projection_freshness.get(),
                    self.projection_freshness_max.get(),
                ),
                maximum(
                    SloMetric::Startup,
                    measured.startup.get(),
                    self.startup_max.get(),
                ),
                maximum(
                    SloMetric::Recovery,
                    measured.recovery.get(),
                    self.recovery_max.get(),
                ),
                maximum(SloMetric::Cpu, measured.cpu.get(), self.cpu_max.get()),
                maximum(SloMetric::Rss, measured.rss.get(), self.rss_max.get()),
                maximum(
                    SloMetric::StorageGrowth,
                    measured.storage_growth.get(),
                    self.storage_growth_max.get(),
                ),
                maximum(
                    SloMetric::Temperature,
                    measured.temperature.get(),
                    self.temperature_max.get(),
                ),
                maximum(SloMetric::Power, measured.power.get(), self.power_max.get()),
            ],
        }
    }
}

/// Allowed degradation relative to a previously accepted measurement set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegressionTolerance {
    /// Relative degradation allowed for any metric.
    pub relative: Ratio,
    /// Absolute degradation allowed in each metric's own unit.
    pub absolute: f64,
}

/// One baseline comparison.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegressionResult {
    /// Compared metric.
    pub metric: SloMetric,
    /// Signed degradation: positive means worse, negative means better.
    pub degradation: f64,
    /// Maximum allowed positive degradation.
    pub allowed: f64,
    /// Whether the comparison is within tolerance.
    pub passed: bool,
}

/// Compares candidate measurements to baseline with metric-aware direction.
pub fn compare_regression(
    baseline: SloMeasurements,
    candidate: SloMeasurements,
    tolerance: RegressionTolerance,
) -> Result<Vec<RegressionResult>, SloError> {
    if !tolerance.absolute.is_finite() || tolerance.absolute < 0.0 {
        return Err(SloError::InvalidTolerance);
    }
    let values = [
        (
            SloMetric::CaptureContinuity,
            baseline.capture_continuity.get(),
            candidate.capture_continuity.get(),
            true,
        ),
        (
            SloMetric::ValidWindowCoverage,
            baseline.valid_window_coverage.get(),
            candidate.valid_window_coverage.get(),
            true,
        ),
        (
            SloMetric::QueueDelay,
            baseline.queue_delay.get(),
            candidate.queue_delay.get(),
            false,
        ),
        (
            SloMetric::ProjectionFreshness,
            baseline.projection_freshness.get(),
            candidate.projection_freshness.get(),
            false,
        ),
        (
            SloMetric::Startup,
            baseline.startup.get(),
            candidate.startup.get(),
            false,
        ),
        (
            SloMetric::Recovery,
            baseline.recovery.get(),
            candidate.recovery.get(),
            false,
        ),
        (
            SloMetric::Cpu,
            baseline.cpu.get(),
            candidate.cpu.get(),
            false,
        ),
        (
            SloMetric::Rss,
            baseline.rss.get(),
            candidate.rss.get(),
            false,
        ),
        (
            SloMetric::StorageGrowth,
            baseline.storage_growth.get(),
            candidate.storage_growth.get(),
            false,
        ),
        (
            SloMetric::Temperature,
            baseline.temperature.get(),
            candidate.temperature.get(),
            false,
        ),
        (
            SloMetric::Power,
            baseline.power.get(),
            candidate.power.get(),
            false,
        ),
    ];
    Ok(values
        .into_iter()
        .map(|(metric, baseline, candidate, higher_is_better)| {
            let degradation = if higher_is_better {
                baseline - candidate
            } else {
                candidate - baseline
            };
            let allowed = tolerance.absolute + baseline.abs() * tolerance.relative.get();
            RegressionResult {
                metric,
                degradation,
                allowed,
                passed: degradation <= allowed,
            }
        })
        .collect())
}

/// Invalid typed SLO input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SloError {
    /// Non-finite or out-of-range measurement.
    InvalidMeasurement(&'static str),
    /// Regression tolerance is non-finite or negative.
    InvalidTolerance,
}

impl fmt::Display for SloError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for SloError {}
