//! Development-host deterministic acquisition replay benchmark.

use std::hint::black_box;
use std::time::Instant;

use ste_radio_acquisition::replay::{ReplayLimits, parse_rvcsi};

fn main() {
    const FRAMES: u64 = 10_000;
    const ITERATIONS: u64 = 20;
    let bytes = fixture(FRAMES);
    let limits = ReplayLimits::default();
    let first = parse_rvcsi(&bytes, limits).expect("generated fixture parses");
    let second = parse_rvcsi(&bytes, limits).expect("generated fixture replays");
    assert_eq!(first, second, "identical replay must be deterministic");

    let started = Instant::now();
    let mut accepted = 0_u64;
    let mut rejected = 0_u64;
    for _ in 0..ITERATIONS {
        let report = black_box(parse_rvcsi(black_box(&bytes), limits).expect("fixture parses"));
        accepted += report.frames.len() as u64;
        rejected +=
            report.rejected_malformed + report.rejected_implausible + report.rejected_non_finite;
    }
    let elapsed = started.elapsed().as_secs_f64();
    let throughput = accepted as f64 / elapsed;
    println!(
        "{{\"schema\":\"ste-acquisition-benchmark-v1\",\"fixture_frames\":{FRAMES},\"iterations\":{ITERATIONS},\"accepted\":{accepted},\"rejected\":{rejected},\"input_bytes\":{},\"elapsed_ms\":{:.3},\"throughput_frames_s\":{throughput:.2},\"deterministic\":true}}",
        bytes.len(),
        elapsed * 1_000.0
    );
}

fn fixture(frames: u64) -> Vec<u8> {
    let mut bytes = b"RVCSIv1\0".to_vec();
    for sequence in 1..=frames {
        let mut record = Vec::with_capacity(43);
        record.extend(sequence.to_le_bytes());
        record.extend((1_000 + sequence).to_le_bytes());
        record.extend(5_180_000_000_u64.to_le_bytes());
        record.extend(20_000_000_u32.to_le_bytes());
        record.push(1);
        record.extend(1_u16.to_le_bytes());
        record.extend(1.0_f64.to_bits().to_le_bytes());
        record.extend((-1.0_f64).to_bits().to_le_bytes());
        bytes.extend((record.len() as u32).to_le_bytes());
        bytes.extend(record);
    }
    bytes
}
