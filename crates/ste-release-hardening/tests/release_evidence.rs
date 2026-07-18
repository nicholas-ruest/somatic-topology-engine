//! Reproducibility, completeness, tamper, and identical-channel release tests.

use ed25519_dalek::SigningKey;
use sha2::{Digest, Sha256};
use ste_release_hardening::release_evidence::*;

fn artifacts() -> Vec<ReleaseArtifact> {
    use ReleaseArtifactKind::*;
    [
        SourceTree,
        DependencyLock,
        Toolchain,
        Software,
        Firmware,
        SoftwareSbom,
        FirmwareSbom,
        ModelSbom,
        DatasetSbom,
        ModelCards,
        DatasetCards,
        CompatibilityMatrix,
        Migrations,
        UpdateDocumentation,
        RollbackPlan,
        SupportMatrix,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, kind)| {
        let name = format!("artifact-{index}");
        ReleaseArtifact {
            kind,
            sha256: Sha256::digest(name.as_bytes()).into(),
            bytes: index as u64 + 1,
            name,
        }
    })
    .collect()
}
fn build(input: Vec<ReleaseArtifact>) -> SignedReleaseManifest {
    ReleaseManifestBuilder::build_and_sign(
        "ste-1.0.0",
        "aarch64-unknown-linux-gnu",
        "crowpi-verified-r1",
        input,
        "release-2026",
        &SigningKey::from_bytes(&[31; 32]),
    )
    .unwrap()
}

#[test]
fn unordered_identical_inputs_build_byte_identical_signed_manifests() {
    let first = build(artifacts());
    let mut reversed = artifacts();
    reversed.reverse();
    let second = build(reversed);
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    first
        .verify(&SigningKey::from_bytes(&[31; 32]).verifying_key())
        .unwrap();
}

#[test]
fn every_required_evidence_class_including_rollback_and_support_is_mandatory() {
    for index in 0..artifacts().len() {
        let mut incomplete = artifacts();
        incomplete.remove(index);
        assert_eq!(
            ReleaseManifestBuilder::build_and_sign(
                "ste-1",
                "aarch64",
                "hardware",
                incomplete,
                "key",
                &SigningKey::from_bytes(&[31; 32])
            ),
            Err(ReleaseEvidenceError::MissingRequiredArtifact)
        );
    }
}

#[test]
fn artifact_manifest_and_signature_tampering_are_detected() {
    let key = SigningKey::from_bytes(&[31; 32]);
    let mut artifact_tamper = build(artifacts());
    artifact_tamper.manifest.artifacts[0].sha256[0] ^= 1;
    assert_eq!(
        artifact_tamper.verify(&key.verifying_key()),
        Err(ReleaseEvidenceError::DigestMismatch)
    );
    let mut digest_tamper = build(artifacts());
    digest_tamper.manifest_digest[0] ^= 1;
    assert_eq!(
        digest_tamper.verify(&key.verifying_key()),
        Err(ReleaseEvidenceError::DigestMismatch)
    );
    let mut signature_tamper = build(artifacts());
    signature_tamper.signature[0] ^= 1;
    assert_eq!(
        signature_tamper.verify(&key.verifying_key()),
        Err(ReleaseEvidenceError::InvalidSignature)
    );
}

#[test]
fn candidate_pilot_and_production_must_reference_the_identical_manifest_digest() {
    let digest = build(artifacts()).manifest_digest;
    let other = [9; 32];
    let mut ledger = ChannelPromotionLedger::default();
    assert_eq!(
        ledger.promote(ReleaseChannel::Pilot, digest),
        Err(ReleaseEvidenceError::NonIdenticalOrInvalidPromotion)
    );
    ledger.promote(ReleaseChannel::Candidate, digest).unwrap();
    assert_eq!(
        ledger.promote(ReleaseChannel::Pilot, other),
        Err(ReleaseEvidenceError::NonIdenticalOrInvalidPromotion)
    );
    ledger.promote(ReleaseChannel::Pilot, digest).unwrap();
    ledger.promote(ReleaseChannel::Production, digest).unwrap();
    assert_eq!(ledger.digest(ReleaseChannel::Production), Some(digest));
    assert_eq!(
        ledger.promote(ReleaseChannel::Production, digest),
        Err(ReleaseEvidenceError::NonIdenticalOrInvalidPromotion)
    );
}

#[test]
fn duplicate_evidence_cannot_hide_a_missing_category() {
    let mut duplicate = artifacts();
    duplicate[1].kind = duplicate[0].kind;
    assert_eq!(
        ReleaseManifestBuilder::build_and_sign(
            "ste-1",
            "aarch64",
            "hardware",
            duplicate,
            "key",
            &SigningKey::from_bytes(&[31; 32])
        ),
        Err(ReleaseEvidenceError::DuplicateOrNonCanonical)
    );
}
