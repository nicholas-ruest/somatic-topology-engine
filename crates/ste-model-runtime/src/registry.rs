//! Fail-closed, atomic model registry lifecycle.

use crate::package::VerifiedPackage;
use std::{collections::BTreeMap, error::Error, fmt};

/// Complete model lifecycle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryState {
    /// Verified but isolated from serving.
    Quarantined,
    /// Known-answer, security, and resource evaluation passed.
    Evaluated,
    /// Scientific promotion and named approval recorded.
    Promoted,
    /// Atomically selected for serving.
    Active,
    /// Temporarily disabled by health/safety control.
    Suspended,
    /// Superseded but retained for rollback evidence.
    Retired,
    /// Permanently prohibited from serving.
    Revoked,
}

/// Immutable evidence for a lifecycle transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryDecision {
    /// Model affected.
    pub model_id: String,
    /// Previous state.
    pub from: RegistryState,
    /// New state.
    pub to: RegistryState,
    /// Evidence bundle digest.
    pub evidence: [u8; 32],
    /// Named approval identity.
    pub approved_by: String,
    /// Deterministic transition sequence.
    pub sequence: u64,
}

#[derive(Clone, Debug)]
struct Entry {
    package: VerifiedPackage,
    state: RegistryState,
}

/// In-memory domain registry; persistence adapters must retain the decision log.
#[derive(Clone, Debug, Default)]
pub struct ModelRegistry {
    entries: BTreeMap<String, Entry>,
    decisions: Vec<RegistryDecision>,
    active: Option<String>,
    previous_active: Option<String>,
    sequence: u64,
}

impl ModelRegistry {
    /// Registers an already verified package into quarantine.
    pub fn register(&mut self, package: VerifiedPackage) -> Result<(), RegistryError> {
        let id = package.package().metadata().model_id.clone();
        if self.entries.contains_key(&id) {
            return Err(RegistryError::AlreadyRegistered);
        }
        self.entries.insert(
            id,
            Entry {
                package,
                state: RegistryState::Quarantined,
            },
        );
        Ok(())
    }
    /// Records successful operational evaluation.
    pub fn evaluate(
        &mut self,
        id: &str,
        evidence: [u8; 32],
        approver: impl Into<String>,
    ) -> Result<(), RegistryError> {
        self.transition(
            id,
            RegistryState::Quarantined,
            RegistryState::Evaluated,
            evidence,
            approver,
        )
    }
    /// Records immutable scientific promotion evidence.
    pub fn promote(
        &mut self,
        id: &str,
        evidence: [u8; 32],
        approver: impl Into<String>,
    ) -> Result<(), RegistryError> {
        self.transition(
            id,
            RegistryState::Evaluated,
            RegistryState::Promoted,
            evidence,
            approver,
        )
    }
    /// Atomically activates a promoted model and retires the prior active model for rollback.
    pub fn activate(
        &mut self,
        id: &str,
        evidence: [u8; 32],
        approver: impl Into<String>,
    ) -> Result<(), RegistryError> {
        if self.state(id)? != RegistryState::Promoted {
            return Err(RegistryError::InvalidTransition);
        }
        let approver = approver.into();
        if approver.trim().is_empty() {
            return Err(RegistryError::MissingApproval);
        }
        let prior = self.active.clone();
        if let Some(old) = &prior {
            if old != id && self.state(old)? != RegistryState::Active {
                return Err(RegistryError::InvalidTransition);
            }
        }
        if let Some(old) = &prior {
            if old != id {
                self.entries.get_mut(old).expect("validated entry").state = RegistryState::Retired;
                self.record(
                    old,
                    RegistryState::Active,
                    RegistryState::Retired,
                    evidence,
                    &approver,
                );
            }
        }
        self.entries.get_mut(id).expect("validated entry").state = RegistryState::Active;
        self.record(
            id,
            RegistryState::Promoted,
            RegistryState::Active,
            evidence,
            &approver,
        );
        self.previous_active = prior;
        self.active = Some(id.to_owned());
        Ok(())
    }
    /// Suspends an active model immediately.
    pub fn suspend(
        &mut self,
        id: &str,
        evidence: [u8; 32],
        approver: impl Into<String>,
    ) -> Result<(), RegistryError> {
        self.transition(
            id,
            RegistryState::Active,
            RegistryState::Suspended,
            evidence,
            approver,
        )?;
        if self.active.as_deref() == Some(id) {
            self.active = None;
        }
        Ok(())
    }
    /// Permanently revokes a model from any non-revoked state.
    pub fn revoke(
        &mut self,
        id: &str,
        evidence: [u8; 32],
        approver: impl Into<String>,
    ) -> Result<(), RegistryError> {
        let from = self.state(id)?;
        if from == RegistryState::Revoked {
            return Err(RegistryError::InvalidTransition);
        }
        self.transition(id, from, RegistryState::Revoked, evidence, approver)?;
        if self.active.as_deref() == Some(id) {
            self.active = None;
        }
        Ok(())
    }
    /// Restores the immediately previous, non-revoked model atomically.
    pub fn rollback(
        &mut self,
        evidence: [u8; 32],
        approver: impl Into<String>,
    ) -> Result<(), RegistryError> {
        let prior = self
            .previous_active
            .clone()
            .ok_or(RegistryError::NoRollbackTarget)?;
        let state = self.state(&prior)?;
        if state == RegistryState::Revoked {
            return Err(RegistryError::Revoked);
        }
        if !matches!(state, RegistryState::Retired | RegistryState::Suspended) {
            return Err(RegistryError::InvalidTransition);
        }
        let approver = approver.into();
        if approver.trim().is_empty() {
            return Err(RegistryError::MissingApproval);
        }
        let current = self.active.clone();
        if let Some(id) = &current {
            if self.state(id)? != RegistryState::Active {
                return Err(RegistryError::InvalidTransition);
            }
        }
        if let Some(id) = &current {
            self.entries.get_mut(id).expect("validated entry").state = RegistryState::Suspended;
            self.record(
                id,
                RegistryState::Active,
                RegistryState::Suspended,
                evidence,
                &approver,
            );
        }
        self.entries.get_mut(&prior).expect("validated entry").state = RegistryState::Active;
        self.record(&prior, state, RegistryState::Active, evidence, &approver);
        self.active = Some(prior);
        Ok(())
    }
    /// Current lifecycle state.
    pub fn state(&self, id: &str) -> Result<RegistryState, RegistryError> {
        self.entries
            .get(id)
            .map(|e| e.state)
            .ok_or(RegistryError::NotFound)
    }
    /// Returns the active verified package; revoked and inactive models are unservable.
    #[must_use]
    pub fn active(&self) -> Option<&VerifiedPackage> {
        self.active
            .as_ref()
            .and_then(|id| self.entries.get(id))
            .filter(|e| e.state == RegistryState::Active)
            .map(|e| &e.package)
    }
    /// Returns a verified package for pre-activation tests without making it servable.
    pub fn package(&self, id: &str) -> Result<&VerifiedPackage, RegistryError> {
        self.entries
            .get(id)
            .map(|entry| &entry.package)
            .ok_or(RegistryError::NotFound)
    }
    /// Immutable decision history.
    #[must_use]
    pub fn decisions(&self) -> &[RegistryDecision] {
        &self.decisions
    }
    fn transition(
        &mut self,
        id: &str,
        from: RegistryState,
        to: RegistryState,
        evidence: [u8; 32],
        approver: impl Into<String>,
    ) -> Result<(), RegistryError> {
        if self.state(id)? != from {
            return Err(RegistryError::InvalidTransition);
        }
        let approver = approver.into();
        if approver.trim().is_empty() {
            return Err(RegistryError::MissingApproval);
        }
        self.sequence += 1;
        self.entries
            .get_mut(id)
            .ok_or(RegistryError::NotFound)?
            .state = to;
        self.decisions.push(RegistryDecision {
            model_id: id.to_owned(),
            from,
            to,
            evidence,
            approved_by: approver,
            sequence: self.sequence,
        });
        Ok(())
    }

    fn record(
        &mut self,
        id: &str,
        from: RegistryState,
        to: RegistryState,
        evidence: [u8; 32],
        approver: &str,
    ) {
        self.sequence += 1;
        self.decisions.push(RegistryDecision {
            model_id: id.to_owned(),
            from,
            to,
            evidence,
            approved_by: approver.to_owned(),
            sequence: self.sequence,
        });
    }
}

/// Stable registry failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryError {
    /// Model is absent.
    NotFound,
    /// Model ID is already registered.
    AlreadyRegistered,
    /// Lifecycle edge is prohibited.
    InvalidTransition,
    /// Named approval was absent.
    MissingApproval,
    /// No previous active model exists.
    NoRollbackTarget,
    /// Rollback target was revoked.
    Revoked,
}
impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "model registry rejected operation: {self:?}")
    }
}
impl Error for RegistryError {}
