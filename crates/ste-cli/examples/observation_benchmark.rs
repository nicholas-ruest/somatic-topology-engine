//! Deterministic signal-only observation replay benchmark.

use std::hint::black_box;
use std::time::Instant;

use ste_signal_observation::dsp::{DspGraphSpec, PrimitiveCsiFrame};
use ste_signal_observation::{
    AlgorithmVersion, DspVersion, ObservationReplay, ObservationWindowId, PartitionRole,
    ReplayEvidenceFrame, WindowBounds, WindowPolicy,
};

fn main() {
    const FRAMES: usize = 512;
    const ITERATIONS: usize = 100;
    let frames = frames();
    let first = replay(&frames);
    let second = replay(&frames);
    assert_eq!(first, second, "same replay must be numerically identical");
    let expected_digest = first.digest().to_owned();

    let started = Instant::now();
    for _ in 0..ITERATIONS {
        let artifact = black_box(replay(black_box(&frames)));
        assert_eq!(artifact.digest(), expected_digest);
    }
    let elapsed = started.elapsed().as_secs_f64();
    let windows_s = ITERATIONS as f64 / elapsed;
    let frames_s = (FRAMES * ITERATIONS) as f64 / elapsed;
    println!(
        "{{\"schema\":\"ste-observation-benchmark-v1\",\"frames_per_window\":{FRAMES},\"iterations\":{ITERATIONS},\"elapsed_ms\":{:.3},\"windows_s\":{windows_s:.2},\"frames_s\":{frames_s:.2},\"deterministic\":true,\"digest\":\"{expected_digest}\",\"numerical_delta\":0.0}}",
        elapsed * 1_000.0
    );
}

fn frames() -> Vec<ReplayEvidenceFrame> {
    (0..512)
        .map(|index| {
            let source_ref = format!("benchmark-radio:{index}");
            let phase = index as f64 * 0.1;
            ReplayEvidenceFrame {
                source_ref: source_ref.clone(),
                frame: PrimitiveCsiFrame {
                    source_ref,
                    event_time_ns: 1_000_000_000 + index as u64 * 10_000_000,
                    subcarriers: vec![(phase.sin(), phase.cos()), (0.5, -0.25)],
                },
            }
        })
        .collect()
}

fn replay(frames: &[ReplayEvidenceFrame]) -> ste_signal_observation::FeatureEvidenceArtifact {
    ObservationReplay::replay(
        ObservationWindowId::new("benchmark-window").unwrap(),
        WindowBounds::new(1_000_000_000, 7_000_000_000).unwrap(),
        WindowPolicy::new("benchmark-v1", 512, 512, 0.0, 1.0).unwrap(),
        AlgorithmVersion::new("features-v1").unwrap(),
        DspVersion::new("dsp-v1").unwrap(),
        "calibration-benchmark-v1".into(),
        PartitionRole::Development,
        DspGraphSpec {
            version: 1,
            sample_rate_hz: 100.0,
            window_len: 512,
            saturation_magnitude: 100.0,
            presence_threshold: 0.0,
            periodicity_min_lag: 1,
            periodicity_max_lag: 64,
        },
        frames,
    )
    .unwrap()
}
