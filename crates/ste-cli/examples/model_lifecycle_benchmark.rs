//! Current-host package verification and rollback benchmark.

use ed25519_dalek::SigningKey;
use std::time::Instant;
use ste_model_runtime::{
    package::{Compatibility, ModelMetadata, ModelPackage},
    registry::ModelRegistry,
};

fn signed(id: &str, key: &SigningKey) -> ModelPackage {
    let metadata = ModelMetadata::new(
        id,
        "deterministic-v1",
        [1; 32],
        [2; 32],
        [3; 32],
        "test-scope",
        "immutable-lineage",
        "host-profile",
        "MIT",
        "ste-0.1",
        std::env::consts::ARCH,
        [4; 32],
    )
    .unwrap();
    ModelPackage::unsigned(metadata, vec![42; 4096])
        .unwrap()
        .sign(key)
        .unwrap()
}

fn main() {
    let key = SigningKey::from_bytes(&[11; 32]);
    let compatibility = Compatibility::new("ste-0.1", std::env::consts::ARCH, [1; 32]).unwrap();
    let candidate = signed("candidate-v1", &key);
    let iterations = 10_000_u64;
    let start = Instant::now();
    for _ in 0..iterations {
        candidate
            .verify(&key.verifying_key(), &compatibility)
            .unwrap();
    }
    let elapsed = start.elapsed();

    let mut registry = ModelRegistry::default();
    for id in ["model-old", "model-new"] {
        registry
            .register(
                signed(id, &key)
                    .verify(&key.verifying_key(), &compatibility)
                    .unwrap(),
            )
            .unwrap();
        registry.evaluate(id, [5; 32], "qa").unwrap();
        registry.promote(id, [6; 32], "science").unwrap();
    }
    registry.activate("model-old", [7; 32], "release").unwrap();
    registry.activate("model-new", [8; 32], "release").unwrap();
    registry.suspend("model-new", [9; 32], "health").unwrap();
    registry.rollback([10; 32], "recovery").unwrap();
    assert_eq!(
        registry.active().unwrap().package().metadata().model_id,
        "model-old"
    );

    println!("host_arch={}", std::env::consts::ARCH);
    println!("host_os={}", std::env::consts::OS);
    println!("payload_bytes=4096");
    println!("verify_iterations={iterations}");
    println!("verify_elapsed_ns={}", elapsed.as_nanos());
    println!(
        "verify_per_second={:.3}",
        iterations as f64 / elapsed.as_secs_f64()
    );
    println!("rollback_kat=passed");
    println!("reference_pi_status=pending");
}
