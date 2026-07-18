//! Replay and local persistence adapters behind application-owned ports.

use crate::application::{CsiCaptureSource, FrameJournal, SessionRepository};
use crate::domain::{AcquisitionError, CaptureSession, CsiFrameInput, ValidatedFrame};
use crate::replay::{ReplayError, ReplayFrame, ReplayLimits, parse_pcap, parse_rvcsi};
use std::collections::VecDeque;
use std::sync::Mutex;

/// Deterministic replay source with all file parsing completed before use.
pub struct ReplayCaptureSource {
    frames: VecDeque<ReplayFrame>,
}

impl ReplayCaptureSource {
    /// Builds a source from bounded RVCSI interchange bytes.
    pub fn from_rvcsi(bytes: &[u8], limits: ReplayLimits) -> Result<Self, ReplayError> {
        Ok(Self {
            frames: parse_rvcsi(bytes, limits)?.frames.into(),
        })
    }

    /// Builds a source from bounded classic-PCAP bytes.
    pub fn from_pcap(bytes: &[u8], limits: ReplayLimits) -> Result<Self, ReplayError> {
        Ok(Self {
            frames: parse_pcap(bytes, limits)?.frames.into(),
        })
    }
}

impl CsiCaptureSource for ReplayCaptureSource {
    fn next_frame(&mut self) -> Result<Option<CsiFrameInput>, AcquisitionError> {
        Ok(self.frames.pop_front().map(Into::into))
    }
}

impl From<ReplayFrame> for CsiFrameInput {
    fn from(frame: ReplayFrame) -> Self {
        Self {
            sequence: frame.sequence,
            monotonic_time_ns: frame.event_time_ns,
            center_frequency_hz: frame.center_hz,
            bandwidth_hz: u64::from(frame.bandwidth_hz),
            antenna_count: u16::from(frame.antenna_count),
            subcarriers: frame.subcarriers,
        }
    }
}

/// Anti-corruption adapter accepting only stable primitive fields from rvCSI.
/// No upstream rvCSI/Nexmon type crosses this module boundary.
pub struct RvCsiAntiCorruptionAdapter;

impl RvCsiAntiCorruptionAdapter {
    /// Converts interleaved upstream `f32` I/Q into the domain's owned `f64` input.
    pub fn adapt_interleaved(
        sequence: u64,
        monotonic_time_ns: u64,
        center_frequency_hz: u64,
        bandwidth_hz: u64,
        antenna_count: u16,
        interleaved_iq: &[f32],
        max_subcarriers: usize,
    ) -> Result<CsiFrameInput, AcquisitionError> {
        if interleaved_iq.len() % 2 != 0
            || interleaved_iq.is_empty()
            || interleaved_iq.len() / 2 > max_subcarriers
        {
            return Err(AcquisitionError::FrameProfileMismatch);
        }
        let mut subcarriers = Vec::with_capacity(interleaved_iq.len() / 2);
        for pair in interleaved_iq.chunks_exact(2) {
            let real = f64::from(pair[0]);
            let imaginary = f64::from(pair[1]);
            if !real.is_finite() || !imaginary.is_finite() {
                return Err(AcquisitionError::NonFiniteCsi);
            }
            subcarriers.push((real, imaginary));
        }
        Ok(CsiFrameInput {
            sequence,
            monotonic_time_ns,
            center_frequency_hz,
            bandwidth_hz,
            antenna_count,
            subcarriers,
        })
    }
}

/// Thread-safe accepted-frame journal test/development adapter.
#[derive(Default)]
pub struct InMemoryFrameJournal {
    frames: Mutex<Vec<ValidatedFrame>>,
}

impl InMemoryFrameJournal {
    /// Returns an owned snapshot in append order.
    #[must_use]
    pub fn snapshot(&self) -> Vec<ValidatedFrame> {
        self.frames
            .lock()
            .map_or_else(|_| Vec::new(), |frames| frames.clone())
    }
}

impl FrameJournal for InMemoryFrameJournal {
    fn append(&self, frame: &ValidatedFrame) -> Result<(), AcquisitionError> {
        self.frames
            .lock()
            .map_err(|_| AcquisitionError::JournalFailure)?
            .push(frame.clone());
        Ok(())
    }
}

/// Thread-safe aggregate repository test/development adapter.
#[derive(Default)]
pub struct InMemorySessionRepository {
    latest: Mutex<Option<CaptureSession>>,
}

impl InMemorySessionRepository {
    /// Returns the last atomically saved aggregate.
    #[must_use]
    pub fn latest(&self) -> Option<CaptureSession> {
        self.latest.lock().ok().and_then(|session| session.clone())
    }
}

impl SessionRepository for InMemorySessionRepository {
    fn save(&self, session: &CaptureSession) -> Result<(), AcquisitionError> {
        *self
            .latest
            .lock()
            .map_err(|_| AcquisitionError::JournalFailure)? = Some(session.clone());
        Ok(())
    }
}
