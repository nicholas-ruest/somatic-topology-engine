//! Bounded-cardinality local metrics.
use std::collections::{BTreeMap, BTreeSet};
/// Metric registration/update failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetricError {
    /// Unknown metric.
    Unknown,
    /// Label key not declared.
    ForbiddenLabel,
    /// Cardinality budget exhausted.
    CardinalityExceeded,
}
/// Local metric registry with per-metric label schemas and series caps.
#[derive(Default)]
pub struct MetricRegistry {
    specs: BTreeMap<String, (BTreeSet<String>, usize)>,
    values: BTreeMap<(String, Vec<(String, String)>), f64>,
    dropped: u64,
}
impl MetricRegistry {
    /// Registers one metric schema.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        labels: impl IntoIterator<Item = String>,
        max_series: usize,
    ) {
        self.specs
            .insert(name.into(), (labels.into_iter().collect(), max_series));
    }
    /// Records a value or rejects unbounded labels.
    pub fn record(
        &mut self,
        name: &str,
        labels: BTreeMap<String, String>,
        value: f64,
    ) -> Result<(), MetricError> {
        let Some((allowed, max)) = self.specs.get(name) else {
            return Err(MetricError::Unknown);
        };
        if !value.is_finite() || labels.keys().any(|k| !allowed.contains(k)) {
            self.dropped += 1;
            return Err(MetricError::ForbiddenLabel);
        }
        let key = (name.to_owned(), labels.into_iter().collect());
        let existing = self.values.contains_key(&key);
        if !existing && self.values.keys().filter(|(n, _)| n == name).count() >= *max {
            self.dropped += 1;
            return Err(MetricError::CardinalityExceeded);
        }
        self.values.insert(key, value);
        Ok(())
    }
    /// Rejected update count.
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }
    /// Current bounded series count.
    #[must_use]
    pub fn series(&self) -> usize {
        self.values.len()
    }
}
