//! Frozen-run respiration evaluation adapter using shared validation metrics.

use crate::estimator::RespirationOutcome;
use std::collections::BTreeSet;
use ste_experiment_validation::metrics::{AgreementMetrics, FailureRate, agreement, failure_rate};

/// Immutable dataset evidence origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceOrigin {
    /// Simulated data useful for plumbing and known-answer tests only.
    Synthetic,
    /// Human protocol data with a real reference belt.
    HumanRespiratoryBelt,
}

/// Frozen partition role accepted by evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationSplit {
    /// Hyperparameter/model selection validation split.
    Validation,
    /// Final untouched test split.
    Test,
    /// Training data is never an evaluation split.
    Training,
}

/// One attempted estimate and its optional belt reference.
#[derive(Clone, Debug, PartialEq)]
pub struct EvaluationPair {
    /// Stable session identifier.
    pub session_id: String,
    /// Stable acquisition day identifier.
    pub day_id: String,
    /// Estimator output, including explicit abstention.
    pub estimate: RespirationOutcome,
    /// Time-aligned real/simulated belt value when available.
    pub belt_bpm: Option<f64>,
    /// Reference alignment/quality gate result.
    pub belt_reference_valid: bool,
}

/// Complete frozen validation run. No repository state is read implicitly.
#[derive(Clone, Debug, PartialEq)]
pub struct FrozenRespirationRun {
    /// Immutable run identifier.
    pub run_id: String,
    /// Whether inputs, package, split, and metrics were frozen before evaluation.
    pub frozen: bool,
    /// Declared evaluation split.
    pub split: EvaluationSplit,
    /// Evidence origin.
    pub origin: EvidenceOrigin,
    /// Training sessions used by the package.
    pub training_sessions: BTreeSet<String>,
    /// Training acquisition days used by the package.
    pub training_days: BTreeSet<String>,
    /// Evaluation attempts.
    pub pairs: Vec<EvaluationPair>,
}

/// Preregistered evaluation gates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RespirationEvaluationGates {
    /// Required emitted-estimate fraction.
    pub minimum_coverage: f64,
    /// Maximum allowed MAE in breaths per minute.
    pub maximum_mae_bpm: f64,
    /// Maximum allowed RMSE in breaths per minute.
    pub maximum_rmse_bpm: f64,
    /// Explicit interval critical value.
    pub failure_rate_critical_value: f64,
}

/// Why a capability remains disabled after evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisabledReason {
    /// Run/artifacts were not frozen before evaluation.
    NotFrozen,
    /// Training partition was presented as evaluation.
    WrongSplit,
    /// Session or acquisition day crosses training/evaluation partitions.
    HeldOutLeakage,
    /// Evidence is synthetic and cannot promote a human capability.
    SyntheticEvidenceOnly,
    /// No sufficient valid real respiratory-belt pairs exist.
    NoRealBeltEvidence,
    /// Emitted-estimate coverage is below the preregistered gate.
    CoverageGateFailed,
    /// Agreement error exceeds a preregistered gate.
    AgreementGateFailed,
    /// Run or gate values are malformed/non-finite.
    InvalidRun,
}

/// Evaluation outcome never directly promotes a model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationStatus {
    /// Capability remains disabled with preserved negative reason.
    Disabled(DisabledReason),
    /// Human evidence passed numerical gates and is eligible for separate review.
    EligibleForIndependentReview,
}

/// Immutable evaluation report with explicit coverage and failures.
#[derive(Clone, Debug, PartialEq)]
pub struct RespirationEvaluationReport {
    /// Run identity.
    pub run_id: String,
    /// Attempt count including abstentions and missing references.
    pub attempted: u64,
    /// Emitted estimates paired to valid references.
    pub paired: u64,
    /// Paired fraction of attempts.
    pub coverage: f64,
    /// Agreement only when at least two valid pairs exist.
    pub agreement: Option<AgreementMetrics>,
    /// Failure/abstention/missing-reference rate and interval.
    pub failure_rate: Option<FailureRate>,
    /// Non-promoting final status.
    pub status: EvaluationStatus,
}

/// Evaluates only explicit frozen inputs using Phase 09 metric implementations.
#[must_use]
pub fn evaluate_respiration_run(
    run: &FrozenRespirationRun,
    gates: RespirationEvaluationGates,
) -> RespirationEvaluationReport {
    let attempted = run.pairs.len() as u64;
    let invalid_gates = !gates.minimum_coverage.is_finite()
        || !(0.0..=1.0).contains(&gates.minimum_coverage)
        || !gates.maximum_mae_bpm.is_finite()
        || gates.maximum_mae_bpm < 0.0
        || !gates.maximum_rmse_bpm.is_finite()
        || gates.maximum_rmse_bpm < 0.0
        || !gates.failure_rate_critical_value.is_finite()
        || gates.failure_rate_critical_value <= 0.0;
    let evaluation_sessions = run
        .pairs
        .iter()
        .map(|pair| pair.session_id.clone())
        .collect::<BTreeSet<_>>();
    let evaluation_days = run
        .pairs
        .iter()
        .map(|pair| pair.day_id.clone())
        .collect::<BTreeSet<_>>();
    let leakage = !run.training_sessions.is_disjoint(&evaluation_sessions)
        || !run.training_days.is_disjoint(&evaluation_days);

    let paired_values = run
        .pairs
        .iter()
        .filter_map(|pair| {
            let estimate = match &pair.estimate {
                RespirationOutcome::Estimated(estimate) => Some(estimate.breaths_per_minute),
                RespirationOutcome::Abstained(_) => None,
            }?;
            let reference = pair.belt_bpm?;
            (pair.belt_reference_valid
                && estimate.is_finite()
                && reference.is_finite()
                && reference > 0.0)
                .then_some((estimate, reference))
        })
        .collect::<Vec<_>>();
    let paired = paired_values.len() as u64;
    let coverage = if attempted == 0 {
        0.0
    } else {
        paired as f64 / attempted as f64
    };
    let agreement_metrics = agreement(&paired_values).ok();
    let failures = attempted.saturating_sub(paired);
    let failure_metrics = (attempted > 0)
        .then(|| failure_rate(failures, attempted, gates.failure_rate_critical_value).ok())
        .flatten();

    let disabled = if run.run_id.trim().is_empty() || attempted == 0 || invalid_gates {
        Some(DisabledReason::InvalidRun)
    } else if !run.frozen {
        Some(DisabledReason::NotFrozen)
    } else if run.split == EvaluationSplit::Training {
        Some(DisabledReason::WrongSplit)
    } else if leakage
        || run
            .pairs
            .iter()
            .any(|pair| pair.session_id.trim().is_empty() || pair.day_id.trim().is_empty())
    {
        Some(DisabledReason::HeldOutLeakage)
    } else if run.origin == EvidenceOrigin::Synthetic {
        Some(DisabledReason::SyntheticEvidenceOnly)
    } else if agreement_metrics.is_none() {
        Some(DisabledReason::NoRealBeltEvidence)
    } else if coverage < gates.minimum_coverage {
        Some(DisabledReason::CoverageGateFailed)
    } else if agreement_metrics.as_ref().is_some_and(|metrics| {
        metrics.mae > gates.maximum_mae_bpm || metrics.rmse > gates.maximum_rmse_bpm
    }) {
        Some(DisabledReason::AgreementGateFailed)
    } else {
        None
    };
    RespirationEvaluationReport {
        run_id: run.run_id.clone(),
        attempted,
        paired,
        coverage,
        agreement: agreement_metrics,
        failure_rate: failure_metrics,
        status: disabled.map_or(
            EvaluationStatus::EligibleForIndependentReview,
            EvaluationStatus::Disabled,
        ),
    }
}
