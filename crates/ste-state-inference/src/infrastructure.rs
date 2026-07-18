//! Immutable assessment persistence and safe projection mapping.

use crate::{
    application::StateAssessmentRepository,
    domain::{AbstentionReason, AssessmentOutcome, DisplayProjectionV1},
};
use serde::Serialize;
use std::{collections::BTreeMap, error::Error, fmt};

/// Exclusive in-process immutable assessment repository.
#[derive(Default)]
pub struct AtomicStateRepository {
    outcomes: BTreeMap<String, AssessmentOutcome>,
}
impl AtomicStateRepository {
    /// Reads immutable evidence by opaque assessment identity.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&AssessmentOutcome> {
        self.outcomes.get(id)
    }
    /// Number of estimates and abstentions retained.
    #[must_use]
    pub fn len(&self) -> usize {
        self.outcomes.len()
    }
    /// Whether the repository contains no outcomes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.outcomes.is_empty()
    }
}
impl StateAssessmentRepository for AtomicStateRepository {
    type Error = StateRepositoryError;
    fn append(&mut self, id: &str, outcome: &AssessmentOutcome) -> Result<(), Self::Error> {
        if id.trim().is_empty() || id.len() > 256 {
            return Err(StateRepositoryError::InvalidIdentifier);
        }
        match self.outcomes.get(id) {
            Some(existing) if existing == outcome => Ok(()),
            Some(_) => Err(StateRepositoryError::ImmutableConflict),
            None => {
                self.outcomes.insert(id.to_owned(), outcome.clone());
                Ok(())
            }
        }
    }
}

/// Sole user-facing mapping: an approved coarse projection or typed unavailable state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "availability", rename_all = "snake_case")]
pub enum SafeDisplayProjection {
    /// Fixed, versioned approved projection.
    Available {
        /// Approved projection payload.
        projection: DisplayProjectionV1,
    },
    /// No latent-state claim is supportable.
    Unavailable {
        /// Opaque assessment identity.
        assessment_id: String,
        /// Stable reason code.
        reason: AbstentionReason,
    },
}
impl TryFrom<&AssessmentOutcome> for SafeDisplayProjection {
    type Error = StateRepositoryError;
    fn try_from(outcome: &AssessmentOutcome) -> Result<Self, Self::Error> {
        match outcome {
            AssessmentOutcome::Estimated(assessment) => Ok(Self::Available {
                projection: DisplayProjectionV1::try_from(assessment)
                    .map_err(|_| StateRepositoryError::NotProjectable)?,
            }),
            AssessmentOutcome::Abstained {
                assessment_id,
                reason,
            } => Ok(Self::Unavailable {
                assessment_id: assessment_id.clone(),
                reason: *reason,
            }),
        }
    }
}

/// Payload-free repository/projection error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateRepositoryError {
    /// Invalid assessment identity.
    InvalidIdentifier,
    /// Conflicting immutable outcome.
    ImmutableConflict,
    /// Domain result is not approved for projection.
    NotProjectable,
}
impl fmt::Display for StateRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "state projection unavailable: {self:?}")
    }
}
impl Error for StateRepositoryError {}
