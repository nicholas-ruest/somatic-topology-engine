#![no_main]

use libfuzzer_sys::fuzz_target;
use ste_signal_observation::dsp::{DspGraphSpec, PrimitiveCsiFrame, execute_dsp};

fuzz_target!(|data: &[u8]| {
    const WINDOW: usize = 16;
    let mut frames = Vec::with_capacity(WINDOW);
    for (index, chunk) in data.chunks(16).take(WINDOW).enumerate() {
        let mut real = [0_u8; 8];
        let mut imaginary = [0_u8; 8];
        let split = chunk.len().min(8);
        real[..split].copy_from_slice(&chunk[..split]);
        if chunk.len() > 8 {
            imaginary[..chunk.len() - 8].copy_from_slice(&chunk[8..]);
        }
        frames.push(PrimitiveCsiFrame {
            source_ref: format!("fuzz-{index}"),
            event_time_ns: (index as u64 + 1) * 100_000_000,
            subcarriers: vec![(
                f64::from_bits(u64::from_le_bytes(real)),
                f64::from_bits(u64::from_le_bytes(imaginary)),
            )],
        });
    }
    while frames.len() < WINDOW {
        frames.push(PrimitiveCsiFrame {
            source_ref: format!("padding-{}", frames.len()),
            event_time_ns: (frames.len() as u64 + 1) * 100_000_000,
            subcarriers: vec![(0.0, 0.0)],
        });
    }
    let _ = execute_dsp(
        DspGraphSpec {
            version: 1,
            sample_rate_hz: 10.0,
            window_len: WINDOW,
            saturation_magnitude: 1.0e6,
            presence_threshold: 0.0,
            periodicity_min_lag: 1,
            periodicity_max_lag: 8,
        },
        &frames,
    );
});
