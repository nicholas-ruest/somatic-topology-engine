//! Deterministic event-time debounce policy and replay.

use std::error::Error;
use std::fmt;

/// Versioned state-transition debounce policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemporalPolicy {
    /// Policy schema/semantic version.
    pub version: u16,
    /// Consecutive identical candidates required for a transition.
    pub required_consecutive: u16,
    /// Minimum event-time dwell after the previous transition.
    pub minimum_dwell_ns: u64,
    /// Gap that clears a partial candidate run.
    pub maximum_gap_ns: u64,
    /// Maximum evidence age at processing time.
    pub maximum_evidence_age_ns: u64,
}

impl TemporalPolicy {
    /// Validates bounded, non-zero temporal semantics.
    pub fn validate(self) -> Result<Self, TemporalError> {
        if self.version == 0
            || self.required_consecutive == 0
            || self.maximum_gap_ns == 0
            || self.maximum_evidence_age_ns == 0
        {
            Err(TemporalError::InvalidPolicy)
        } else {
            Ok(self)
        }
    }
}

/// One policy-approved candidate; the opaque state ID is defined elsewhere.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalCandidate {
    /// Source assessment identity.
    pub assessment_id: String,
    /// Event time of its full evidence horizon.
    pub event_time_ns: u64,
    /// Opaque approved state/band identifier.
    pub state_id: String,
}

/// Debounce result for one candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemporalOutcome {
    /// Candidate was accepted but has not met transition policy.
    Held {
        /// Current stable state, absent before initial transition.
        current_state: Option<String>,
        /// Consecutive samples accumulated for the candidate.
        candidate_count: u16,
    },
    /// Stable state changed after satisfying consecutive and dwell rules.
    Transitioned {
        /// Prior stable state, absent for initial establishment.
        from: Option<String>,
        /// New stable state.
        to: String,
        /// Source assessment causing the transition.
        assessment_id: String,
        /// Deterministic event time.
        event_time_ns: u64,
        /// Policy version used.
        policy_version: u16,
    },
}

/// Stateful deterministic temporal reducer.
#[derive(Clone, Debug)]
pub struct TemporalDebouncer {
    policy: TemporalPolicy,
    current: Option<String>,
    candidate: Option<String>,
    candidate_count: u16,
    last_event_ns: Option<u64>,
    last_transition_ns: Option<u64>,
}

impl TemporalDebouncer {
    /// Creates an empty reducer from a valid versioned policy.
    pub fn new(policy: TemporalPolicy) -> Result<Self, TemporalError> {
        Ok(Self {
            policy: policy.validate()?,
            current: None,
            candidate: None,
            candidate_count: 0,
            last_event_ns: None,
            last_transition_ns: None,
        })
    }

    /// Applies one monotonically ordered candidate using explicit processing time.
    pub fn apply(
        &mut self,
        input: TemporalCandidate,
        processed_at_ns: u64,
    ) -> Result<TemporalOutcome, TemporalError> {
        if input.assessment_id.trim().is_empty()
            || input.state_id.trim().is_empty()
            || input.event_time_ns == 0
            || processed_at_ns < input.event_time_ns
        {
            return Err(TemporalError::InvalidCandidate);
        }
        if processed_at_ns - input.event_time_ns > self.policy.maximum_evidence_age_ns {
            return Err(TemporalError::StaleEvidence);
        }
        if self
            .last_event_ns
            .is_some_and(|previous| input.event_time_ns <= previous)
        {
            return Err(TemporalError::NonMonotonicEventTime);
        }

        let gap_reset = self
            .last_event_ns
            .is_some_and(|previous| input.event_time_ns - previous > self.policy.maximum_gap_ns);
        if gap_reset || self.candidate.as_deref() != Some(&input.state_id) {
            self.candidate = Some(input.state_id.clone());
            self.candidate_count = 1;
        } else {
            self.candidate_count = self.candidate_count.saturating_add(1);
        }
        self.last_event_ns = Some(input.event_time_ns);

        let dwell_satisfied = self
            .last_transition_ns
            .is_none_or(|previous| input.event_time_ns - previous >= self.policy.minimum_dwell_ns);
        if self.candidate_count >= self.policy.required_consecutive
            && self.current.as_deref() != Some(&input.state_id)
            && dwell_satisfied
        {
            let from = self.current.replace(input.state_id.clone());
            self.last_transition_ns = Some(input.event_time_ns);
            self.candidate_count = 0;
            self.candidate = None;
            Ok(TemporalOutcome::Transitioned {
                from,
                to: input.state_id,
                assessment_id: input.assessment_id,
                event_time_ns: input.event_time_ns,
                policy_version: self.policy.version,
            })
        } else {
            Ok(TemporalOutcome::Held {
                current_state: self.current.clone(),
                candidate_count: self.candidate_count,
            })
        }
    }

    /// Returns current stable state.
    #[must_use]
    pub fn current_state(&self) -> Option<&str> {
        self.current.as_deref()
    }
}

/// Replays candidates with a fixed processing delay and returns every outcome.
pub fn replay_temporal(
    policy: TemporalPolicy,
    candidates: &[TemporalCandidate],
    processing_delay_ns: u64,
) -> Result<Vec<TemporalOutcome>, TemporalError> {
    let mut reducer = TemporalDebouncer::new(policy)?;
    candidates
        .iter()
        .cloned()
        .map(|candidate| {
            let processed = candidate
                .event_time_ns
                .checked_add(processing_delay_ns)
                .ok_or(TemporalError::InvalidCandidate)?;
            reducer.apply(candidate, processed)
        })
        .collect()
}

/// Temporal-policy validation or replay failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalError {
    /// Policy contains a zero/unsupported field.
    InvalidPolicy,
    /// Candidate identity/time is malformed or overflows.
    InvalidCandidate,
    /// Candidate event time repeated or moved backwards.
    NonMonotonicEventTime,
    /// Evidence exceeded maximum age.
    StaleEvidence,
}

impl fmt::Display for TemporalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for TemporalError {}
