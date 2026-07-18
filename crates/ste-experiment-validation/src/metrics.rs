//! Deterministic finite validation metrics with explicit sample counts.

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

/// Bland–Altman-style agreement and absolute-error summary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AgreementMetrics {
    /// Paired sample count.
    pub count: u64,
    /// Mean estimate-minus-reference error.
    pub bias: f64,
    /// Mean absolute error.
    pub mae: f64,
    /// Root mean squared error.
    pub rmse: f64,
    /// Bias minus 1.96 sample standard deviations.
    pub lower_limit_of_agreement: f64,
    /// Bias plus 1.96 sample standard deviations.
    pub upper_limit_of_agreement: f64,
}

/// Computes finite paired agreement without dropping invalid rows.
pub fn agreement(pairs: &[(f64, f64)]) -> Result<AgreementMetrics, MetricError> {
    if pairs.len() < 2 {
        return Err(MetricError::InsufficientSamples);
    }
    validate_finite(pairs.iter().flat_map(|pair| [pair.0, pair.1]))?;
    let errors = pairs
        .iter()
        .map(|(estimate, reference)| estimate - reference)
        .collect::<Vec<_>>();
    let bias = mean(&errors);
    let mae = mean(&errors.iter().map(|error| error.abs()).collect::<Vec<_>>());
    let rmse = mean(&errors.iter().map(|error| error * error).collect::<Vec<_>>()).sqrt();
    let standard_deviation = (errors
        .iter()
        .map(|error| (error - bias).powi(2))
        .sum::<f64>()
        / (errors.len() - 1) as f64)
        .sqrt();
    let result = AgreementMetrics {
        count: errors.len() as u64,
        bias,
        mae,
        rmse,
        lower_limit_of_agreement: bias - 1.96 * standard_deviation,
        upper_limit_of_agreement: bias + 1.96 * standard_deviation,
    };
    validate_finite([
        result.bias,
        result.mae,
        result.rmse,
        result.lower_limit_of_agreement,
        result.upper_limit_of_agreement,
    ])?;
    Ok(result)
}

/// Binary confidence calibration summary.
#[derive(Clone, Debug, PartialEq)]
pub struct CalibrationMetrics {
    /// Included sample count.
    pub count: u64,
    /// Mean squared probability error.
    pub brier_score: f64,
    /// Weighted absolute confidence/accuracy gap.
    pub expected_calibration_error: f64,
    /// Largest non-empty bin gap.
    pub maximum_calibration_error: f64,
    /// Explicit non-empty bin summaries.
    pub bins: Vec<CalibrationBin>,
}

/// One deterministic equal-width calibration bin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalibrationBin {
    /// Inclusive lower bound except at zero.
    pub lower: f64,
    /// Inclusive upper bound.
    pub upper: f64,
    /// Samples in this bin.
    pub count: u64,
    /// Mean confidence.
    pub mean_confidence: f64,
    /// Observed positive fraction.
    pub observed_frequency: f64,
}

/// Computes equal-width binary calibration with confidence in `[0,1]`.
pub fn calibration(
    samples: &[(f64, bool)],
    bin_count: usize,
) -> Result<CalibrationMetrics, MetricError> {
    if samples.is_empty() || bin_count == 0 || bin_count > 1_000 {
        return Err(MetricError::InvalidConfiguration);
    }
    if samples
        .iter()
        .any(|(confidence, _)| !confidence.is_finite() || !(0.0..=1.0).contains(confidence))
    {
        return Err(MetricError::NonFiniteOrOutOfRange);
    }
    let mut buckets = vec![Vec::<(f64, bool)>::new(); bin_count];
    for &(confidence, outcome) in samples {
        let index = ((confidence * bin_count as f64).floor() as usize).min(bin_count - 1);
        buckets[index].push((confidence, outcome));
    }
    let mut bins = Vec::new();
    let mut expected = 0.0_f64;
    let mut maximum = 0.0_f64;
    for (index, bucket) in buckets.into_iter().enumerate() {
        if bucket.is_empty() {
            continue;
        }
        let confidence = bucket.iter().map(|sample| sample.0).sum::<f64>() / bucket.len() as f64;
        let observed = bucket.iter().filter(|sample| sample.1).count() as f64 / bucket.len() as f64;
        let gap = (confidence - observed).abs();
        expected += gap * bucket.len() as f64 / samples.len() as f64;
        maximum = maximum.max(gap);
        bins.push(CalibrationBin {
            lower: index as f64 / bin_count as f64,
            upper: (index + 1) as f64 / bin_count as f64,
            count: bucket.len() as u64,
            mean_confidence: confidence,
            observed_frequency: observed,
        });
    }
    let brier_score = samples
        .iter()
        .map(|(confidence, outcome)| (confidence - f64::from(*outcome as u8)).powi(2))
        .sum::<f64>()
        / samples.len() as f64;
    let result = CalibrationMetrics {
        count: samples.len() as u64,
        brier_score,
        expected_calibration_error: expected,
        maximum_calibration_error: maximum,
        bins,
    };
    validate_finite([
        result.brier_score,
        result.expected_calibration_error,
        result.maximum_calibration_error,
    ])?;
    Ok(result)
}

/// One operating point on a selective-risk curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectiveRiskPoint {
    /// Fraction of all predictions retained.
    pub coverage: f64,
    /// Mean supplied loss among retained predictions.
    pub selective_risk: f64,
    /// Minimum confidence retained at this point.
    pub confidence_threshold: f64,
}

/// Sorts by confidence descending and reports every retained-prefix operating point.
pub fn selective_risk_coverage(
    confidence_and_loss: &[(f64, f64)],
) -> Result<Vec<SelectiveRiskPoint>, MetricError> {
    if confidence_and_loss.is_empty() {
        return Err(MetricError::InsufficientSamples);
    }
    if confidence_and_loss.iter().any(|(confidence, loss)| {
        !confidence.is_finite()
            || !(0.0..=1.0).contains(confidence)
            || !loss.is_finite()
            || *loss < 0.0
    }) {
        return Err(MetricError::NonFiniteOrOutOfRange);
    }
    let mut ordered = confidence_and_loss
        .iter()
        .copied()
        .enumerate()
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        right
            .1
            .0
            .partial_cmp(&left.1.0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });
    let mut loss_sum = 0.0;
    let result = ordered
        .iter()
        .enumerate()
        .map(|(index, (_, (confidence, loss)))| {
            loss_sum += loss;
            SelectiveRiskPoint {
                coverage: (index + 1) as f64 / ordered.len() as f64,
                selective_risk: loss_sum / (index + 1) as f64,
                confidence_threshold: *confidence,
            }
        })
        .collect::<Vec<_>>();
    validate_finite(result.iter().flat_map(|point| {
        [
            point.coverage,
            point.selective_risk,
            point.confidence_threshold,
        ]
    }))?;
    Ok(result)
}

/// Two-sided normal-approximation confidence interval for a finite sample mean.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConfidenceInterval {
    /// Sample mean.
    pub estimate: f64,
    /// Lower endpoint.
    pub lower: f64,
    /// Upper endpoint.
    pub upper: f64,
    /// Confidence level represented by the caller-supplied critical value.
    pub critical_value: f64,
}

/// Computes a mean interval with explicit critical value (for example `1.96`).
pub fn mean_confidence_interval(
    values: &[f64],
    critical_value: f64,
) -> Result<ConfidenceInterval, MetricError> {
    if values.len() < 2 || !critical_value.is_finite() || critical_value <= 0.0 {
        return Err(MetricError::InsufficientSamples);
    }
    validate_finite(values.iter().copied())?;
    let estimate = mean(values);
    let variance = values
        .iter()
        .map(|value| (value - estimate).powi(2))
        .sum::<f64>()
        / (values.len() - 1) as f64;
    let margin = critical_value * (variance / values.len() as f64).sqrt();
    let result = ConfidenceInterval {
        estimate,
        lower: estimate - margin,
        upper: estimate + margin,
        critical_value,
    };
    validate_finite([result.estimate, result.lower, result.upper])?;
    Ok(result)
}

/// Failure-rate summary using a Wilson score interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FailureRate {
    /// Failed attempts.
    pub failures: u64,
    /// Total attempts.
    pub attempts: u64,
    /// Observed failure fraction.
    pub rate: f64,
    /// Wilson lower endpoint.
    pub lower: f64,
    /// Wilson upper endpoint.
    pub upper: f64,
}

/// Computes failure fraction and finite Wilson interval.
pub fn failure_rate(
    failures: u64,
    attempts: u64,
    critical_value: f64,
) -> Result<FailureRate, MetricError> {
    if attempts == 0 || failures > attempts || !critical_value.is_finite() || critical_value <= 0.0
    {
        return Err(MetricError::InvalidConfiguration);
    }
    let n = attempts as f64;
    let rate = failures as f64 / n;
    let z2 = critical_value * critical_value;
    let center = (rate + z2 / (2.0 * n)) / (1.0 + z2 / n);
    let margin =
        critical_value * ((rate * (1.0 - rate) / n + z2 / (4.0 * n * n)).sqrt()) / (1.0 + z2 / n);
    let result = FailureRate {
        failures,
        attempts,
        rate,
        lower: (center - margin).clamp(0.0, 1.0),
        upper: (center + margin).clamp(0.0, 1.0),
    };
    validate_finite([result.rate, result.lower, result.upper])?;
    Ok(result)
}

/// Direction for a preregistered baseline metric.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImprovementDirection {
    /// Larger metric values are better.
    HigherIsBetter,
    /// Smaller metric values are better.
    LowerIsBetter,
}

/// Explicit candidate-versus-baseline comparison.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineComparison {
    /// Baseline metric.
    pub baseline: f64,
    /// Candidate metric.
    pub candidate: f64,
    /// Signed improvement in the preregistered direction.
    pub improvement: f64,
    /// Relative improvement; `None` when baseline is zero.
    pub relative_improvement: Option<f64>,
}

/// Compares finite values without guessing whether higher or lower is better.
pub fn compare_baseline(
    baseline: f64,
    candidate: f64,
    direction: ImprovementDirection,
) -> Result<BaselineComparison, MetricError> {
    validate_finite([baseline, candidate])?;
    let improvement = match direction {
        ImprovementDirection::HigherIsBetter => candidate - baseline,
        ImprovementDirection::LowerIsBetter => baseline - candidate,
    };
    let result = BaselineComparison {
        baseline,
        candidate,
        improvement,
        relative_improvement: (baseline != 0.0).then_some(improvement / baseline.abs()),
    };
    validate_finite(
        [Some(result.improvement), result.relative_improvement]
            .into_iter()
            .flatten(),
    )?;
    Ok(result)
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn validate_finite(values: impl IntoIterator<Item = f64>) -> Result<(), MetricError> {
    if values.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(MetricError::NonFiniteOrOutOfRange)
    }
}

/// Stable metric input/configuration error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricError {
    /// Not enough observations for the defined statistic.
    InsufficientSamples,
    /// NaN, infinity, probability outside `[0,1]`, or negative loss.
    NonFiniteOrOutOfRange,
    /// Bin count, attempt count, or critical value is invalid.
    InvalidConfiguration,
}

impl fmt::Display for MetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for MetricError {}
