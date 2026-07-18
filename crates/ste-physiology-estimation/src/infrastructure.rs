//! Immutable persistence and Experiment Validation policy adapters.

use std::{collections::BTreeMap, error::Error, fmt};

use ste_experiment_validation::domain::{PromotionDecision, PromotionRegistry};

use crate::application::{PhysiologyAssessmentRepository, ValidationRegistry};
use crate::domain::AssessmentOutcome;

/// Exclusive in-process repository preserving estimates and abstentions.
#[derive(Default)]
pub struct AtomicPhysiologyRepository {
    assessments: BTreeMap<String, AssessmentOutcome>,
}

impl AtomicPhysiologyRepository {
    /// Number of immutable outcomes, including abstentions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.assessments.len()
    }

    /// Returns whether no outcomes have been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.assessments.is_empty()
    }

    /// Reads an immutable outcome without changing repository state.
    #[must_use]
    pub fn get(&self, assessment_id: &str) -> Option<&AssessmentOutcome> {
        self.assessments.get(assessment_id)
    }
}

impl PhysiologyAssessmentRepository for AtomicPhysiologyRepository {
    type Error = PhysiologyRepositoryError;

    fn append(
        &mut self,
        assessment_id: &str,
        outcome: &AssessmentOutcome,
    ) -> Result<(), Self::Error> {
        if assessment_id.trim().is_empty() || assessment_id.len() > 256 {
            return Err(PhysiologyRepositoryError::InvalidIdentifier);
        }
        match self.assessments.get(assessment_id) {
            Some(existing) if existing == outcome => Ok(()),
            Some(_) => Err(PhysiologyRepositoryError::ImmutableConflict),
            None => {
                self.assessments
                    .insert(assessment_id.to_owned(), outcome.clone());
                Ok(())
            }
        }
    }
}

/// Read-only adapter over the validation bounded context's append-only registry.
pub struct ExperimentValidationRegistry<'a> {
    registry: &'a PromotionRegistry,
    capability: &'a str,
    model_id: &'a str,
}

impl<'a> ExperimentValidationRegistry<'a> {
    /// Binds one exact model package to one exact promoted capability.
    #[must_use]
    pub const fn new(
        registry: &'a PromotionRegistry,
        capability: &'a str,
        model_id: &'a str,
    ) -> Self {
        Self {
            registry,
            capability,
            model_id,
        }
    }
}

impl ValidationRegistry for ExperimentValidationRegistry<'_> {
    type Error = PhysiologyRepositoryError;

    fn respiration_is_promoted(&self, model_id: &str) -> Result<bool, Self::Error> {
        if model_id != self.model_id || self.capability != "respiration-v1" {
            return Ok(false);
        }
        Ok(matches!(
            self.registry.history(self.capability).last(),
            Some(PromotionDecision::Promoted { .. })
        ))
    }
}

/// Payload-free persistence/policy adapter error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysiologyRepositoryError {
    /// Assessment identifier was absent or unsafe.
    InvalidIdentifier,
    /// An immutable identifier was reused with different evidence.
    ImmutableConflict,
}
impl fmt::Display for PhysiologyRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentifier => "invalid physiology assessment identifier",
            Self::ImmutableConflict => "immutable physiology assessment conflict",
        })
    }
}
impl Error for PhysiologyRepositoryError {}
