//! Adversarial package verification and lifecycle acceptance tests.

use ed25519_dalek::SigningKey;
use ste_model_runtime::{
    package::{Compatibility, ModelMetadata, ModelPackage, PackageError, VerifiedPackage},
    registry::{ModelRegistry, RegistryError, RegistryState},
};

fn metadata(id: &str) -> ModelMetadata {
    ModelMetadata::new(
        id,
        "ste-linear-v1",
        [1; 32],
        [2; 32],
        [3; 32],
        "single-still-adult",
        "study-9",
        "pi5-budget-v1",
        "MIT",
        "ste-0.1",
        "aarch64-pi5",
        [4; 32],
    )
    .unwrap()
}
fn compatibility() -> Compatibility {
    Compatibility::new("ste-0.1", "aarch64-pi5", [1; 32]).unwrap()
}
fn signed(id: &str, key: &SigningKey) -> ModelPackage {
    ModelPackage::unsigned(metadata(id), vec![1, 2, 3])
        .unwrap()
        .sign(key)
        .unwrap()
}
fn verified(id: &str, key: &SigningKey) -> VerifiedPackage {
    signed(id, key)
        .verify(&key.verifying_key(), &compatibility())
        .unwrap()
}

#[test]
fn unsigned_corrupt_and_incompatible_packages_never_verify() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let unsigned = ModelPackage::unsigned(metadata("m1"), vec![1]).unwrap();
    assert_eq!(
        unsigned.verify(&key.verifying_key(), &compatibility()),
        Err(PackageError::Unsigned)
    );

    let mut json = serde_json::to_value(signed("m2", &key)).unwrap();
    json["weights"][0] = serde_json::json!(9);
    let corrupt: ModelPackage = serde_json::from_value(json).unwrap();
    assert_eq!(
        corrupt.verify(&key.verifying_key(), &compatibility()),
        Err(PackageError::CorruptWeights)
    );

    let incompatible = Compatibility::new("ste-9", "aarch64-pi5", [1; 32]).unwrap();
    assert_eq!(
        signed("m3", &key).verify(&key.verifying_key(), &incompatible),
        Err(PackageError::Incompatible)
    );
}

#[test]
fn only_promoted_active_non_revoked_models_can_serve() {
    let key = SigningKey::from_bytes(&[8; 32]);
    let mut registry = ModelRegistry::default();
    registry.register(verified("m1", &key)).unwrap();
    assert!(registry.active().is_none());
    registry.evaluate("m1", [1; 32], "qa").unwrap();
    registry.promote("m1", [2; 32], "science").unwrap();
    registry.activate("m1", [3; 32], "release").unwrap();
    assert!(registry.active().is_some());
    registry.revoke("m1", [4; 32], "security").unwrap();
    assert!(registry.active().is_none());
    assert_eq!(registry.state("m1"), Ok(RegistryState::Revoked));
    assert_eq!(
        registry.activate("m1", [5; 32], "operator"),
        Err(RegistryError::InvalidTransition)
    );
}

#[test]
fn activation_and_rollback_are_atomic_and_decisions_are_preserved() {
    let key = SigningKey::from_bytes(&[9; 32]);
    let mut registry = ModelRegistry::default();
    for id in ["old", "new"] {
        registry.register(verified(id, &key)).unwrap();
        registry.evaluate(id, [1; 32], "qa").unwrap();
        registry.promote(id, [2; 32], "science").unwrap();
    }
    registry.activate("old", [3; 32], "release").unwrap();
    registry.activate("new", [4; 32], "release").unwrap();
    assert_eq!(
        registry.active().unwrap().package().metadata().model_id,
        "new"
    );
    registry.rollback([5; 32], "incident").unwrap();
    assert_eq!(
        registry.active().unwrap().package().metadata().model_id,
        "old"
    );
    assert_eq!(registry.decisions().len(), 9);
}

#[test]
fn failed_activation_or_rollback_is_atomic() {
    let key = SigningKey::from_bytes(&[10; 32]);
    let mut registry = ModelRegistry::default();
    for id in ["old", "new"] {
        registry.register(verified(id, &key)).unwrap();
        registry.evaluate(id, [1; 32], "qa").unwrap();
        registry.promote(id, [2; 32], "science").unwrap();
    }
    registry.activate("old", [3; 32], "release").unwrap();
    let decisions = registry.decisions().len();
    assert_eq!(
        registry.activate("new", [4; 32], ""),
        Err(RegistryError::MissingApproval)
    );
    assert_eq!(
        registry.active().unwrap().package().metadata().model_id,
        "old"
    );
    assert_eq!(registry.decisions().len(), decisions);
    registry.activate("new", [4; 32], "release").unwrap();
    let decisions = registry.decisions().len();
    assert_eq!(
        registry.rollback([5; 32], ""),
        Err(RegistryError::MissingApproval)
    );
    assert_eq!(
        registry.active().unwrap().package().metadata().model_id,
        "new"
    );
    assert_eq!(registry.decisions().len(), decisions);
}
