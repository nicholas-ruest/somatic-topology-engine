//! Corrupt and incompatible package fixtures must remain unservable.

use ed25519_dalek::SigningKey;
use ste_model_runtime::package::{Compatibility, ModelPackage, PackageError};

#[test]
fn corrupt_fixture_is_rejected_before_registration() {
    let package: ModelPackage = serde_json::from_str(include_str!(
        "../../ste-model-runtime/tests/fixtures/corrupt-package.json"
    ))
    .unwrap();
    let key = SigningKey::from_bytes(&[5; 32]);
    assert_eq!(
        package.verify(
            &key.verifying_key(),
            &Compatibility::new("ste-0.1", "test", [1; 32]).unwrap()
        ),
        Err(PackageError::CorruptWeights)
    );
}

#[test]
fn correctly_signed_but_incompatible_fixture_is_rejected() {
    let package: ModelPackage = serde_json::from_str(include_str!(
        "../../ste-model-runtime/tests/fixtures/incompatible-package.json"
    ))
    .unwrap();
    let key = SigningKey::from_bytes(&[5; 32]);
    let signed = ModelPackage::unsigned(package.metadata().clone(), package.weights().to_vec())
        .unwrap()
        .sign(&key)
        .unwrap();
    assert_eq!(
        signed.verify(
            &key.verifying_key(),
            &Compatibility::new("ste-0.1", "test", [1; 32]).unwrap()
        ),
        Err(PackageError::Incompatible)
    );
}
