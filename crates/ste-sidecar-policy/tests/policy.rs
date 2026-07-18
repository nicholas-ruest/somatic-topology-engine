//! Optional-sidecar signature, authority, and failure-isolation tests.
use ed25519_dalek::SigningKey;
use std::collections::BTreeSet;
use ste_sidecar_policy::*;

fn set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|v| (*v).into()).collect()
}
fn core() -> CoreAuthority {
    CoreAuthority {
        purposes: set(&["offline-copy"]),
        capabilities: set(&["copy-suggestion"]),
        claim_level: ClaimLevel::Experimental,
    }
}
fn manifest() -> SidecarManifest {
    SidecarManifest::new(
        "copy-tool-v1",
        "sha256:abc",
        1,
        set(&["suggest_copy"]),
        set(&["offline-copy"]),
        set(&["copy-suggestion"]),
        ClaimLevel::Experimental,
        true,
        false,
        false,
        ResourceLimits::new(100, 64 * 1024 * 1024, 16 * 1024 * 1024, 30).unwrap(),
    )
    .unwrap()
}

#[test]
fn absent_sidecar_never_reduces_core_functionality() {
    let supervisor = SidecarSupervisor::default();
    assert_eq!(supervisor.state(), SidecarState::Absent);
    assert!(supervisor.core_available());
}

#[test]
fn signed_allowlisted_manifest_verifies_and_tampering_fails() {
    let key = SigningKey::from_bytes(&[5; 32]);
    let mut signed = SignedSidecarManifest::sign(manifest(), &key).unwrap();
    assert!(signed.verify(&key.verifying_key(), &core()).is_ok());
    signed.manifest.methods.insert("capture_hardware".into());
    assert!(matches!(
        signed.verify(&key.verifying_key(), &core()),
        Err(SidecarError::InvalidSignature)
    ));
}

#[test]
fn purpose_capability_claim_or_privilege_expansion_is_rejected() {
    let mut cases = Vec::new();
    let mut purpose = manifest();
    purpose.purposes.insert("sensing".into());
    cases.push(purpose);
    let mut capability = manifest();
    capability.capabilities.insert("medical-diagnosis".into());
    cases.push(capability);
    let mut claim = manifest();
    claim.claim_level = ClaimLevel::ValidatedNonMedical;
    cases.push(claim);
    let mut hardware = manifest();
    hardware.hardware_access = true;
    cases.push(hardware);
    let mut store = manifest();
    store.authoritative_store_access = true;
    cases.push(store);
    let mut online = manifest();
    online.offline = false;
    cases.push(online);
    for manifest in cases {
        assert!(manifest.validate_authority(&core()).is_err());
    }
}

#[test]
fn hung_or_over_budget_sidecar_disables_without_affecting_core() {
    let key = SigningKey::from_bytes(&[6; 32]);
    let signed = SignedSidecarManifest::sign(manifest(), &key).unwrap();
    let mut supervisor = SidecarSupervisor::default();
    let verified = signed.verify(&key.verifying_key(), &core()).unwrap();
    supervisor.install_verified(&verified).unwrap();
    for _ in 0..3 {
        supervisor
            .observe(HealthSample {
                responsive: false,
                memory_bytes: u64::MAX,
                requests_this_minute: u32::MAX,
            })
            .unwrap();
    }
    assert_eq!(supervisor.state(), SidecarState::Disabled);
    assert!(supervisor.core_available());
}
