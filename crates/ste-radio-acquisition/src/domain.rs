//! Radio acquisition aggregate, value objects, commands, and events.

use std::{error::Error, fmt, fmt::Write as _};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ste_contracts::ValidatedCsiFrameV1;
use uuid::Uuid;

/// Validated capture profile.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CaptureProfile {
    /// Center frequency.
    pub center_frequency_hz: u64,
    /// Channel bandwidth.
    pub bandwidth_hz: u64,
    /// Receive antenna count.
    pub antenna_count: u16,
    /// Expected subcarrier count.
    pub subcarrier_count: u16,
}

impl CaptureProfile {
    /// Validates a plausible supported Wi-Fi capture shape.
    pub fn new(
        center: u64,
        bandwidth: u64,
        antennas: u16,
        subcarriers: u16,
    ) -> Result<Self, AcquisitionError> {
        if !(2_300_000_000..=7_200_000_000).contains(&center)
            || !(1_000_000..=320_000_000).contains(&bandwidth)
            || antennas == 0
            || antennas > 16
            || subcarriers == 0
            || subcarriers > 4096
        {
            return Err(AcquisitionError::ImplausibleProfile);
        }
        Ok(Self {
            center_frequency_hz: center,
            bandwidth_hz: bandwidth,
            antenna_count: antennas,
            subcarrier_count: subcarriers,
        })
    }
}

/// Exact interface/link identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CaptureLink {
    interface: String,
    link_id: String,
}

impl CaptureLink {
    /// Creates a non-empty link.
    pub fn new(
        interface: impl Into<String>,
        link_id: impl Into<String>,
    ) -> Result<Self, AcquisitionError> {
        let result = Self {
            interface: interface.into(),
            link_id: link_id.into(),
        };
        if result.interface.trim().is_empty() || result.link_id.trim().is_empty() {
            return Err(AcquisitionError::InvalidMetadata);
        }
        Ok(result)
    }
}

/// Hardware, firmware, OS, and radio-peer provenance.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HardwareProvenance {
    board: String,
    chipset: String,
    firmware: String,
    os_kernel: String,
    access_point: String,
}

impl HardwareProvenance {
    /// Creates complete non-empty provenance.
    pub fn new(
        board: impl Into<String>,
        chipset: impl Into<String>,
        firmware: impl Into<String>,
        os_kernel: impl Into<String>,
        access_point: impl Into<String>,
    ) -> Result<Self, AcquisitionError> {
        let value = Self {
            board: board.into(),
            chipset: chipset.into(),
            firmware: firmware.into(),
            os_kernel: os_kernel.into(),
            access_point: access_point.into(),
        };
        if [
            &value.board,
            &value.chipset,
            &value.firmware,
            &value.os_kernel,
            &value.access_point,
        ]
        .iter()
        .any(|part| part.trim().is_empty())
        {
            return Err(AcquisitionError::InvalidMetadata);
        }
        Ok(value)
    }
}

/// Versioned calibration and operating geometry references.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CalibrationMetadata {
    profile_id: String,
    geometry_id: String,
}

impl CalibrationMetadata {
    /// Creates complete calibration metadata.
    pub fn new(
        profile_id: impl Into<String>,
        geometry_id: impl Into<String>,
    ) -> Result<Self, AcquisitionError> {
        let value = Self {
            profile_id: profile_id.into(),
            geometry_id: geometry_id.into(),
        };
        if value.profile_id.trim().is_empty() || value.geometry_id.trim().is_empty() {
            return Err(AcquisitionError::InvalidMetadata);
        }
        Ok(value)
    }
}

/// Untrusted adapter frame before validation.
#[derive(Clone, Debug, PartialEq)]
pub struct CsiFrameInput {
    /// Source sequence.
    pub sequence: u64,
    /// Monotonic event time.
    pub monotonic_time_ns: u64,
    /// Per-frame center frequency.
    pub center_frequency_hz: u64,
    /// Per-frame bandwidth.
    pub bandwidth_hz: u64,
    /// Per-frame receive antenna count.
    pub antenna_count: u16,
    /// Real/imaginary subcarrier components.
    pub subcarriers: Vec<(f64, f64)>,
}

/// Monotonic frame-sequence state preserving missing frames.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FrameSequence {
    last: Option<u64>,
}

/// Capture quality counters.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CaptureHealth {
    /// Accepted frames.
    pub accepted: u64,
    /// Rejected malformed/implausible frames.
    pub rejected: u64,
    /// Missing source sequence positions.
    pub missing: u64,
    /// Frames retained but not published due to bounded backpressure.
    pub backpressured: u64,
}

/// Validated cross-context frame plus local provenance/gap metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedFrame {
    /// Stable V1 published DTO.
    pub contract: ValidatedCsiFrameV1,
    /// Missing sequence count immediately before this frame.
    pub gap_before: u64,
    /// Complete payload-minimized provenance reference.
    pub provenance_ref: String,
}

/// Acquisition aggregate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CaptureSession {
    id: Uuid,
    profile: CaptureProfile,
    link: CaptureLink,
    provenance: HardwareProvenance,
    calibration: CalibrationMetadata,
    started_at_ns: u64,
    sequence: FrameSequence,
    health: CaptureHealth,
}

impl CaptureSession {
    /// Starts a deterministic session definition; publication remains policy-gated.
    pub fn start(
        profile: CaptureProfile,
        link: CaptureLink,
        provenance: HardwareProvenance,
        calibration: CalibrationMetadata,
        started_at_ns: u64,
    ) -> Result<Self, AcquisitionError> {
        let bytes =
            serde_json::to_vec(&(&profile, &link, &provenance, &calibration, started_at_ns))
                .map_err(|_| AcquisitionError::InvalidMetadata)?;
        let digest = Sha256::digest(bytes);
        let mut id = [0_u8; 16];
        id.copy_from_slice(&digest[..16]);
        Ok(Self {
            id: Uuid::from_bytes(id),
            profile,
            link,
            provenance,
            calibration,
            started_at_ns,
            sequence: FrameSequence::default(),
            health: CaptureHealth::default(),
        })
    }

    /// Validates finite values, shape, radio parameters, and sequence ordering without mutation.
    pub fn validate_frame(&self, input: CsiFrameInput) -> Result<ValidatedFrame, AcquisitionError> {
        if input
            .subcarriers
            .iter()
            .any(|(real, imaginary)| !real.is_finite() || !imaginary.is_finite())
        {
            return Err(AcquisitionError::NonFiniteCsi);
        }
        if input.sequence == 0
            || self
                .sequence
                .last
                .is_some_and(|last| input.sequence <= last)
        {
            return Err(AcquisitionError::NonMonotonicSequence);
        }
        if input.center_frequency_hz != self.profile.center_frequency_hz
            || input.bandwidth_hz != self.profile.bandwidth_hz
            || input.antenna_count != self.profile.antenna_count
            || input.subcarriers.len() != usize::from(self.profile.subcarrier_count)
        {
            return Err(AcquisitionError::FrameProfileMismatch);
        }
        let gap_before = self
            .sequence
            .last
            .map_or(input.sequence - 1, |last| input.sequence - last - 1);
        let payload = serde_json::to_vec(&input.subcarriers)
            .map_err(|_| AcquisitionError::InvalidMetadata)?;
        let payload_ref = encode_hex(&Sha256::digest(payload));
        let provenance_ref = format!(
            "board={};chipset={};firmware={};kernel={};ap={};interface={};link={};calibration={};geometry={};channel={};bandwidth={}",
            self.provenance.board,
            self.provenance.chipset,
            self.provenance.firmware,
            self.provenance.os_kernel,
            self.provenance.access_point,
            self.link.interface,
            self.link.link_id,
            self.calibration.profile_id,
            self.calibration.geometry_id,
            self.profile.center_frequency_hz,
            self.profile.bandwidth_hz
        );
        Ok(ValidatedFrame {
            contract: ValidatedCsiFrameV1 {
                capture_session_id: self.id,
                sequence: input.sequence,
                monotonic_time_ns: input.monotonic_time_ns,
                center_frequency_hz: input.center_frequency_hz,
                bandwidth_hz: input.bandwidth_hz,
                antenna_count: input.antenna_count,
                subcarrier_count: self.profile.subcarrier_count,
                payload_ref,
            },
            gap_before,
            provenance_ref,
        })
    }

    /// Validates and records acceptance/gaps.
    pub fn accept(&mut self, input: CsiFrameInput) -> Result<ValidatedFrame, AcquisitionError> {
        match self.validate_frame(input) {
            Ok(frame) => {
                self.sequence.last = Some(frame.contract.sequence);
                self.health.accepted += 1;
                self.health.missing += frame.gap_before;
                Ok(frame)
            }
            Err(error) => {
                self.health.rejected += 1;
                Err(error)
            }
        }
    }

    /// Records bounded publication backpressure without losing acceptance provenance.
    pub fn record_backpressure(&mut self) {
        self.health.backpressured += 1;
    }

    /// Returns current health counters.
    #[must_use]
    pub const fn health(&self) -> CaptureHealth {
        self.health
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(value, "{byte:02x}").expect("String write cannot fail");
    }
    value
}

/// Aggregate commands.
#[derive(Clone, Debug, PartialEq)]
pub enum CaptureCommand {
    /// Begin a configured session.
    Start,
    /// Validate and accept an untrusted frame.
    AcceptFrame(CsiFrameInput),
    /// Stop the session.
    Stop,
}

/// Domain events.
#[derive(Clone, Debug, PartialEq)]
pub enum CaptureEvent {
    /// Session began.
    SessionStarted,
    /// Frame passed all gates.
    FrameAccepted {
        /// Source sequence.
        sequence: u64,
        /// Missing positions before this frame.
        gap_before: u64,
    },
    /// Frame failed validation.
    FrameRejected,
    /// Session stopped.
    SessionStopped,
}

/// Acquisition invariant failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcquisitionError {
    /// Current governance policy denied capture.
    Unauthorized,
    /// Capture profile is physically or operationally implausible.
    ImplausibleProfile,
    /// Required provenance/calibration metadata is absent.
    InvalidMetadata,
    /// CSI contained NaN or infinity.
    NonFiniteCsi,
    /// Sequence repeated or moved backwards.
    NonMonotonicSequence,
    /// Frame shape/channel differs from the session profile.
    FrameProfileMismatch,
    /// Durable accepted-frame journal failed.
    JournalFailure,
}

impl fmt::Display for AcquisitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl Error for AcquisitionError {}

/// Marker retained for architecture tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainBoundary;
