//! Current-host deterministic baseline/projection benchmark; no trained cognitive model.
use std::time::Instant;
use ste_state_inference::{
    SafeDisplayProjection,
    domain::{
        AssessLatentState, CalibratedProbability, ConstructDefinition, EvidenceBundle,
        GateEvidence, ModelScope, Provenance, StateAssessment,
    },
};
fn main() {
    let iterations = 100_000_u64;
    let start = Instant::now();
    for index in 0..iterations {
        let evidence = EvidenceBundle::new(
            "obs",
            "phys",
            30_000,
            Provenance::new("baseline", "cal", "profile").unwrap(),
        )
        .unwrap();
        let scope = ModelScope::new("participant", "room", "task", "seated", "host").unwrap();
        let outcome = StateAssessment::assess(
            AssessLatentState::new(
                format!("a{index}"),
                ConstructDefinition::baseline(),
                evidence,
                scope,
                CalibratedProbability::new(0.99).unwrap(),
                GateEvidence::all_passed(),
            )
            .unwrap(),
        );
        let json =
            serde_json::to_string(&SafeDisplayProjection::try_from(&outcome).unwrap()).unwrap();
        assert!(!json.contains("0.99"));
        assert!(!json.contains("phys"));
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
    println!("trained_cognitive_model=absent");
    println!("unsupported_constructs=disabled");
    println!("raw_output_leakage=none_detected");
    println!("reference_pi_status=pending");
}
