//! Encryption, key lifecycle, deletion, export, and reset acceptance tests.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use ste_storage::DataClass;
use ste_storage::crypto::{
    AssuranceLevel, CryptoError, DevelopmentKeyProvider, EnvelopeCipher, KeyProvider,
    LocalDeviceIdentity,
};
use ste_storage::lifecycle::{
    ClassLifecyclePolicy, DeviceLifecycleState, LifecycleError, LifecycleManager, LifecycleStore,
    PortableEncryptedExport, PortableExportManifest,
};

const ALL_CLASSES: [DataClass; 10] = [
    DataClass::RawCsi,
    DataClass::Observation,
    DataClass::Physiology,
    DataClass::LatentState,
    DataClass::Anchor,
    DataClass::ConsentPolicy,
    DataClass::Audit,
    DataClass::Security,
    DataClass::Diagnostics,
    DataClass::Provenance,
];

#[test]
fn envelope_rejects_wrong_key_aad_and_ciphertext_tampering() {
    let keys = DevelopmentKeyProvider::new();
    let envelope =
        EnvelopeCipher::encrypt(&keys, DataClass::RawCsi, b"sensitive", b"manifest").unwrap();
    assert_eq!(
        &*EnvelopeCipher::decrypt(&keys, &envelope, b"manifest").unwrap(),
        b"sensitive"
    );
    assert!(matches!(
        EnvelopeCipher::decrypt(&DevelopmentKeyProvider::new(), &envelope, b"manifest"),
        Err(CryptoError::KeyUnavailable)
    ));
    assert_eq!(
        EnvelopeCipher::decrypt(&keys, &envelope, b"wrong").unwrap_err(),
        CryptoError::AuthenticationFailed
    );
    let mut tampered = envelope;
    tampered.ciphertext[0] ^= 1;
    assert_eq!(
        EnvelopeCipher::decrypt(&keys, &tampered, b"manifest").unwrap_err(),
        CryptoError::AuthenticationFailed
    );
}

#[test]
fn rotation_keeps_old_envelopes_readable_until_cryptographic_erasure() {
    let keys = DevelopmentKeyProvider::new();
    let old = EnvelopeCipher::encrypt(&keys, DataClass::Anchor, b"old", b"a").unwrap();
    let old_id = old.key_id.clone();
    let new_id = keys.rotate(DataClass::Anchor).unwrap();
    assert_ne!(old_id, new_id);
    let new = EnvelopeCipher::encrypt(&keys, DataClass::Anchor, b"new", b"a").unwrap();
    assert_eq!(
        &*EnvelopeCipher::decrypt(&keys, &old, b"a").unwrap(),
        b"old"
    );
    assert_eq!(
        &*EnvelopeCipher::decrypt(&keys, &new, b"a").unwrap(),
        b"new"
    );
    keys.erase(DataClass::Anchor).unwrap();
    assert_eq!(
        EnvelopeCipher::decrypt(&keys, &old, b"a").unwrap_err(),
        CryptoError::KeyUnavailable
    );
}

#[test]
fn development_identity_cannot_claim_production_assurance() {
    let identity =
        LocalDeviceIdentity::new("device-1", AssuranceLevel::ProtectedDevelopmentFallback).unwrap();
    assert_eq!(
        identity.require_production_assurance(),
        Err(CryptoError::DevelopmentKey)
    );
    let production =
        LocalDeviceIdentity::new("device-2", AssuranceLevel::ProductionHardwareBacked).unwrap();
    assert_eq!(production.require_production_assurance(), Ok(()));
}

#[derive(Default)]
struct TestStore {
    values: Mutex<BTreeMap<(String, DataClass), u64>>,
    reset_count: Mutex<u64>,
}

impl TestStore {
    fn seed(&self, subject: &str) {
        let mut values = self.values.lock().unwrap();
        for class in ALL_CLASSES {
            values.insert((subject.to_owned(), class), 1);
        }
    }
}

impl LifecycleStore for TestStore {
    fn name(&self) -> &str {
        "test-store"
    }
    fn delete_subject(&self, subject: &str, class: DataClass) -> Result<u64, LifecycleError> {
        Ok(self
            .values
            .lock()
            .unwrap()
            .remove(&(subject.to_owned(), class))
            .unwrap_or(0))
    }
    fn reset(&self) -> Result<(), LifecycleError> {
        self.values.lock().unwrap().clear();
        *self.reset_count.lock().unwrap() += 1;
        Ok(())
    }
}

#[test]
fn deletion_visits_every_registered_store_and_data_class_then_erases_keys() {
    let first = Arc::new(TestStore::default());
    let second = Arc::new(TestStore::default());
    first.seed("subject-ref");
    second.seed("subject-ref");
    let keys = Arc::new(DevelopmentKeyProvider::new());
    for class in ALL_CLASSES {
        EnvelopeCipher::encrypt(keys.as_ref(), class, b"x", b"a").unwrap();
    }
    let manager = LifecycleManager::new(vec![first, second], keys.clone());
    let receipt = manager
        .delete_everywhere("subject-ref", &ALL_CLASSES, 100)
        .unwrap();
    assert_eq!(receipt.steps.len(), 20);
    assert!(receipt.steps.iter().all(|step| step.deleted_records == 1));
    assert!(receipt.cryptographic_erasure);
    for class in ALL_CLASSES {
        assert!(matches!(
            keys.key_by_id(class, "dev-key-1"),
            Err(CryptoError::KeyUnavailable)
        ));
    }
}

fn manifest() -> PortableExportManifest {
    PortableExportManifest {
        version: 1,
        export_id: "export-1".into(),
        device_id: "device-public-id".into(),
        data_class: DataClass::Observation,
        purpose: "participant-portability".into(),
        authorization_reference: "authorization-hash".into(),
        created_at: 10,
        expires_at: 100,
        schema_version: 1,
        recipient_key_id: "recipient-public-key-1".into(),
    }
}

#[test]
fn encrypted_export_manifest_has_no_secret_and_restore_is_policy_and_expiry_gated() {
    let keys = DevelopmentKeyProvider::new();
    let policy = ClassLifecyclePolicy {
        data_class: DataClass::Observation,
        retention_seconds: 60,
        export_allowed: true,
        backup_allowed: true,
    };
    let portable =
        PortableEncryptedExport::create(&keys, policy, manifest(), b"portable data").unwrap();
    let manifest_json = serde_json::to_string(&portable.manifest).unwrap();
    assert!(!manifest_json.contains("portable data"));
    assert!(!manifest_json.contains("secret"));
    assert_eq!(
        &*portable.restore(&keys, policy, 20).unwrap(),
        b"portable data"
    );
    assert_eq!(
        portable.restore(&keys, policy, 100).unwrap_err(),
        LifecycleError::InvalidManifest
    );
    let forbidden = ClassLifecyclePolicy {
        backup_allowed: false,
        ..policy
    };
    assert_eq!(
        portable.restore(&keys, forbidden, 20).unwrap_err(),
        LifecycleError::RestoreForbidden
    );
}

#[test]
fn reset_and_decommission_clear_stores_and_keys_and_never_reenable_capture() {
    let store = Arc::new(TestStore::default());
    store.seed("subject");
    let keys = Arc::new(DevelopmentKeyProvider::new());
    let envelope =
        EnvelopeCipher::encrypt(keys.as_ref(), DataClass::ConsentPolicy, b"grant", b"a").unwrap();
    let manager = LifecycleManager::new(vec![store.clone()], keys.clone());
    assert_eq!(
        manager.factory_reset().unwrap(),
        DeviceLifecycleState::CaptureDisabled
    );
    assert_eq!(manager.state(), DeviceLifecycleState::CaptureDisabled);
    assert_eq!(
        EnvelopeCipher::decrypt(keys.as_ref(), &envelope, b"a").unwrap_err(),
        CryptoError::KeyUnavailable
    );
    assert_eq!(*store.reset_count.lock().unwrap(), 1);
    assert_eq!(
        manager.decommission().unwrap(),
        DeviceLifecycleState::Decommissioned
    );
    assert_eq!(manager.state(), DeviceLifecycleState::Decommissioned);
}

#[test]
fn failures_are_redacted_and_never_echo_keys_or_payloads() {
    let diagnostic = CryptoError::AuthenticationFailed.to_string();
    assert_eq!(diagnostic, "authentication failed");
    assert!(!diagnostic.contains("sensitive"));
    assert!(
        !LifecycleError::Crypto(CryptoError::KeyUnavailable)
            .to_string()
            .contains("key id")
    );
}
