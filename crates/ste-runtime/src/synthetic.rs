//! Deterministic clock and synthetic replay pipeline.
#![allow(missing_docs)]

use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockOverflow;

impl fmt::Display for ClockOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("deterministic clock overflow")
    }
}

impl Error for ClockOverflow {}

#[derive(Clone, Debug)]
pub struct DeterministicClock {
    current_millis: u64,
    step_millis: u64,
}

impl DeterministicClock {
    #[must_use]
    pub const fn new(current_millis: u64, step_millis: u64) -> Self {
        Self {
            current_millis,
            step_millis,
        }
    }

    #[must_use]
    pub const fn now_millis(&self) -> u64 {
        self.current_millis
    }

    pub fn tick(&mut self) -> Result<u64, ClockOverflow> {
        self.current_millis = self
            .current_millis
            .checked_add(self.step_millis)
            .ok_or(ClockOverflow)?;
        Ok(self.current_millis)
    }

    pub fn advance_steps(&mut self, steps: u64) -> Result<u64, ClockOverflow> {
        let delta = self.step_millis.checked_mul(steps).ok_or(ClockOverflow)?;
        self.current_millis = self
            .current_millis
            .checked_add(delta)
            .ok_or(ClockOverflow)?;
        Ok(self.current_millis)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntheticEvent {
    pub event_time_millis: u64,
    pub value: i64,
}

impl SyntheticEvent {
    #[must_use]
    pub const fn new(event_time_millis: u64, value: i64) -> Self {
        Self {
            event_time_millis,
            value,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SyntheticPipeline {
    clock: DeterministicClock,
}

impl SyntheticPipeline {
    #[must_use]
    pub const fn new(clock: DeterministicClock) -> Self {
        Self { clock }
    }

    #[must_use]
    pub fn replay(mut self, inputs: &[i64]) -> Vec<SyntheticEvent> {
        let mut output = Vec::with_capacity(inputs.len());
        for (index, value) in inputs.iter().copied().enumerate() {
            if index > 0 && self.clock.tick().is_err() {
                break;
            }
            output.push(SyntheticEvent::new(self.clock.now_millis(), value));
        }
        output
    }
}
