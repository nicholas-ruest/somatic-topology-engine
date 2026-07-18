//! Current-host deterministic respiration estimator benchmark; not a Pi result.

use std::time::Instant;
use ste_physiology_estimation::estimator::{
    RespirationModelPackage, RespirationObservationEvidence, RespirationOutcome,
    estimate_respiration,
};

fn main() {
    let package = RespirationModelPackage {
        package_id: "resp-baseline-v1".into(),
        package_version: "1.0.0".into(),
        algorithm: "deterministic-periodicity-v1".into(),
        dsp_graph_version: 1,
        calibration_id: "synthetic-known-answer-v1".into(),
        minimum_bpm: 6.0,
        maximum_bpm: 30.0,
        minimum_periodicity: 0.7,
        maximum_missingness: 0.1,
        maximum_interference: 0.1,
        minimum_confidence: 0.7,
        confidence_calibration: vec![(0.0, 0.0), (1.0, 1.0)],
    };
    let evidence = RespirationObservationEvidence {
        artifact_ref: "synthetic-known-answer".into(),
        dsp_graph_version: 1,
        periodicity: 0.95,
        dominant_frequency_hz: Some(0.25),
        missingness: 0.01,
        interference: 0.01,
        contaminated: false,
    };
    let iterations = 100_000_u64;
    let start = Instant::now();
    for _ in 0..iterations {
        assert!(matches!(
            estimate_respiration(&package, &evidence).unwrap(),
            RespirationOutcome::Estimated(_)
        ));
    }
    let elapsed = start.elapsed();
    println!("host_arch={}", std::env::consts::ARCH);
    println!("host_os={}", std::env::consts::OS);
    println!("iterations={iterations}");
    println!("elapsed_ns={}", elapsed.as_nanos());
    println!(
        "throughput_per_second={:.3}",
        iterations as f64 / elapsed.as_secs_f64()
    );
    println!("reference_pi_status=pending");
    println!("human_reference_status=pending");
    println!("capability_status=disabled_not_promoted");
}
