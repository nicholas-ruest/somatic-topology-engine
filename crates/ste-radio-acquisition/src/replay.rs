//! Deterministic, bounded parsers for recorded CSI inputs.

use std::error::Error;
use std::fmt;

const RVCSI_MAGIC: &[u8; 8] = b"RVCSIv1\0";
const PCAP_MAGIC_LE: u32 = 0xa1b2c3d4;
const PCAP_MAGIC_BE: u32 = 0xd4c3b2a1;

/// Hard parser budgets applied before allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayLimits {
    /// Maximum complete input size.
    pub max_input_bytes: usize,
    /// Maximum accepted and rejected records combined.
    pub max_frames: usize,
    /// Maximum complex subcarriers in one frame.
    pub max_subcarriers: usize,
    /// Maximum framed record or PCAP packet length.
    pub max_record_bytes: usize,
}

impl Default for ReplayLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 16 * 1024 * 1024,
            max_frames: 100_000,
            max_subcarriers: 4_096,
            max_record_bytes: 256 * 1024,
        }
    }
}

/// Validated replay representation, independent of upstream rvCSI structs.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayFrame {
    /// Monotonic source sequence.
    pub sequence: u64,
    /// Source event time in nanoseconds.
    pub event_time_ns: u64,
    /// Radio center frequency.
    pub center_hz: u64,
    /// Channel bandwidth.
    pub bandwidth_hz: u32,
    /// Reported antenna count.
    pub antenna_count: u8,
    /// Complex samples as finite real/imaginary pairs.
    pub subcarriers: Vec<(f64, f64)>,
}

/// Explicit sequence discontinuity preserved by replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequenceGap {
    /// Last accepted sequence before the gap.
    pub after_sequence: u64,
    /// First accepted sequence after the gap.
    pub next_sequence: u64,
    /// Count of absent sequence values.
    pub missing: u64,
}

/// Deterministic parse outcome and quality counters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReplayReport {
    /// Frames passing structural and semantic validation.
    pub frames: Vec<ReplayFrame>,
    /// Sequence gaps between accepted frames.
    pub gaps: Vec<SequenceGap>,
    /// Frames rejected for non-finite samples.
    pub rejected_non_finite: u64,
    /// Frames rejected for implausible metadata/sample values.
    pub rejected_implausible: u64,
    /// Packets/records rejected as malformed but safely bounded.
    pub rejected_malformed: u64,
}

impl ReplayReport {
    fn accept(&mut self, frame: ReplayFrame) {
        if let Some(previous) = self.frames.last() {
            if frame.sequence > previous.sequence.saturating_add(1) {
                self.gaps.push(SequenceGap {
                    after_sequence: previous.sequence,
                    next_sequence: frame.sequence,
                    missing: frame.sequence - previous.sequence - 1,
                });
            }
        }
        self.frames.push(frame);
    }
}

/// Stable hostile-input failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayError {
    /// Complete input exceeds configured budget.
    InputTooLarge,
    /// Header or record is truncated.
    Truncated,
    /// File signature/version is unsupported.
    UnsupportedFormat,
    /// Declared record length exceeds configured budget.
    RecordTooLarge,
    /// Record count exceeds configured budget.
    TooManyFrames,
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InputTooLarge => "replay input exceeds byte budget",
            Self::Truncated => "replay input is truncated",
            Self::UnsupportedFormat => "unsupported replay format",
            Self::RecordTooLarge => "replay record exceeds byte budget",
            Self::TooManyFrames => "replay exceeds frame budget",
        })
    }
}

impl Error for ReplayError {}

/// Parses the documented deterministic RVCSI interchange framing.
pub fn parse_rvcsi(bytes: &[u8], limits: ReplayLimits) -> Result<ReplayReport, ReplayError> {
    check_input(bytes, limits)?;
    if bytes.get(..RVCSI_MAGIC.len()) != Some(RVCSI_MAGIC) {
        return Err(ReplayError::UnsupportedFormat);
    }
    let mut offset = RVCSI_MAGIC.len();
    let mut report = ReplayReport::default();
    let mut count = 0_usize;
    while offset < bytes.len() {
        let length = read_u32(bytes, &mut offset, Endian::Little)? as usize;
        check_record(length, &mut count, limits)?;
        let end = offset
            .checked_add(length)
            .ok_or(ReplayError::RecordTooLarge)?;
        let record = bytes.get(offset..end).ok_or(ReplayError::Truncated)?;
        offset = end;
        parse_record_into(record, &mut report, limits);
    }
    Ok(report)
}

/// Parses classic PCAP and treats each captured packet as one bounded rvCSI record.
pub fn parse_pcap(bytes: &[u8], limits: ReplayLimits) -> Result<ReplayReport, ReplayError> {
    check_input(bytes, limits)?;
    if bytes.len() < 24 {
        return Err(ReplayError::Truncated);
    }
    let raw_magic = u32::from_le_bytes(bytes[..4].try_into().expect("four-byte slice"));
    let endian = match raw_magic {
        PCAP_MAGIC_LE => Endian::Little,
        PCAP_MAGIC_BE => Endian::Big,
        _ => return Err(ReplayError::UnsupportedFormat),
    };
    let mut offset = 24_usize;
    let mut count = 0_usize;
    let mut report = ReplayReport::default();
    while offset < bytes.len() {
        let header_end = offset.checked_add(16).ok_or(ReplayError::Truncated)?;
        let header = bytes
            .get(offset..header_end)
            .ok_or(ReplayError::Truncated)?;
        let mut header_offset = 8;
        let captured = read_u32(header, &mut header_offset, endian)? as usize;
        check_record(captured, &mut count, limits)?;
        offset = header_end;
        let packet_end = offset
            .checked_add(captured)
            .ok_or(ReplayError::RecordTooLarge)?;
        let packet = bytes
            .get(offset..packet_end)
            .ok_or(ReplayError::Truncated)?;
        offset = packet_end;
        parse_record_into(packet, &mut report, limits);
    }
    Ok(report)
}

fn parse_record_into(record: &[u8], report: &mut ReplayReport, limits: ReplayLimits) {
    match parse_record(record, limits) {
        Ok(frame)
            if report
                .frames
                .last()
                .is_some_and(|previous| frame.sequence <= previous.sequence) =>
        {
            report.rejected_implausible += 1;
        }
        Ok(frame) => report.accept(frame),
        Err(RecordRejection::Malformed) => report.rejected_malformed += 1,
        Err(RecordRejection::NonFinite) => report.rejected_non_finite += 1,
        Err(RecordRejection::Implausible) => report.rejected_implausible += 1,
    }
}

fn parse_record(record: &[u8], limits: ReplayLimits) -> Result<ReplayFrame, RecordRejection> {
    const FIXED: usize = 8 + 8 + 8 + 4 + 1 + 2;
    if record.len() < FIXED {
        return Err(RecordRejection::Malformed);
    }
    let mut offset = 0;
    let sequence = read_u64_record(record, &mut offset)?;
    let event_time_ns = read_u64_record(record, &mut offset)?;
    let center_hz = read_u64_record(record, &mut offset)?;
    let bandwidth_hz = read_u32_record(record, &mut offset)?;
    let antenna_count = *record.get(offset).ok_or(RecordRejection::Malformed)?;
    offset += 1;
    let sample_count = read_u16_record(record, &mut offset)? as usize;
    if sample_count == 0 || sample_count > limits.max_subcarriers {
        return Err(RecordRejection::Implausible);
    }
    let expected = FIXED
        .checked_add(
            sample_count
                .checked_mul(16)
                .ok_or(RecordRejection::Malformed)?,
        )
        .ok_or(RecordRejection::Malformed)?;
    if record.len() != expected {
        return Err(RecordRejection::Malformed);
    }
    if sequence == 0
        || event_time_ns == 0
        || !(2_000_000_000..=7_500_000_000).contains(&center_hz)
        || !(1_000_000..=320_000_000).contains(&bandwidth_hz)
        || antenna_count == 0
        || antenna_count > 16
    {
        return Err(RecordRejection::Implausible);
    }
    let mut subcarriers = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        let real = f64::from_bits(read_u64_record(record, &mut offset)?);
        let imaginary = f64::from_bits(read_u64_record(record, &mut offset)?);
        if !real.is_finite() || !imaginary.is_finite() {
            return Err(RecordRejection::NonFinite);
        }
        if real.abs() > 1.0e9 || imaginary.abs() > 1.0e9 {
            return Err(RecordRejection::Implausible);
        }
        subcarriers.push((real, imaginary));
    }
    Ok(ReplayFrame {
        sequence,
        event_time_ns,
        center_hz,
        bandwidth_hz,
        antenna_count,
        subcarriers,
    })
}

#[derive(Clone, Copy)]
enum Endian {
    Little,
    Big,
}

#[derive(Clone, Copy)]
enum RecordRejection {
    Malformed,
    NonFinite,
    Implausible,
}

fn check_input(bytes: &[u8], limits: ReplayLimits) -> Result<(), ReplayError> {
    if bytes.len() > limits.max_input_bytes {
        Err(ReplayError::InputTooLarge)
    } else {
        Ok(())
    }
}

fn check_record(length: usize, count: &mut usize, limits: ReplayLimits) -> Result<(), ReplayError> {
    if length > limits.max_record_bytes {
        return Err(ReplayError::RecordTooLarge);
    }
    *count = count.checked_add(1).ok_or(ReplayError::TooManyFrames)?;
    if *count > limits.max_frames {
        return Err(ReplayError::TooManyFrames);
    }
    Ok(())
}

fn read_u32(bytes: &[u8], offset: &mut usize, endian: Endian) -> Result<u32, ReplayError> {
    let end = offset.checked_add(4).ok_or(ReplayError::Truncated)?;
    let raw: [u8; 4] = bytes
        .get(*offset..end)
        .ok_or(ReplayError::Truncated)?
        .try_into()
        .expect("four-byte slice");
    *offset = end;
    Ok(match endian {
        Endian::Little => u32::from_le_bytes(raw),
        Endian::Big => u32::from_be_bytes(raw),
    })
}

fn read_u64_record(bytes: &[u8], offset: &mut usize) -> Result<u64, RecordRejection> {
    let end = offset.checked_add(8).ok_or(RecordRejection::Malformed)?;
    let raw = bytes
        .get(*offset..end)
        .ok_or(RecordRejection::Malformed)?
        .try_into()
        .map_err(|_| RecordRejection::Malformed)?;
    *offset = end;
    Ok(u64::from_le_bytes(raw))
}

fn read_u32_record(bytes: &[u8], offset: &mut usize) -> Result<u32, RecordRejection> {
    let end = offset.checked_add(4).ok_or(RecordRejection::Malformed)?;
    let raw = bytes
        .get(*offset..end)
        .ok_or(RecordRejection::Malformed)?
        .try_into()
        .map_err(|_| RecordRejection::Malformed)?;
    *offset = end;
    Ok(u32::from_le_bytes(raw))
}

fn read_u16_record(bytes: &[u8], offset: &mut usize) -> Result<u16, RecordRejection> {
    let end = offset.checked_add(2).ok_or(RecordRejection::Malformed)?;
    let raw = bytes
        .get(*offset..end)
        .ok_or(RecordRejection::Malformed)?
        .try_into()
        .map_err(|_| RecordRejection::Malformed)?;
    *offset = end;
    Ok(u16::from_le_bytes(raw))
}
