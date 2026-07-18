//! Adversarial update, recovery, journal, key, and reset tests.
use ed25519_dalek::SigningKey;
use ste_release_hardening::*;

#[test]
fn signed_ab_activation_rolls_back_and_unsigned_downgrade_is_blocked() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let bundle = UpdateBundle::sign(2, vec![1, 2, 3], "pi5", 1, false, &key).unwrap();
    let verified = bundle.verify(&key.verifying_key(), "pi5", 1).unwrap();
    let mut manager = AbUpdateManager::new(1);
    assert_eq!(manager.stage(verified), Ok(Slot::B));
    manager.activate_pending().unwrap();
    assert_eq!(manager.active(), (Slot::B, 2));
    manager.rollback("health failed").unwrap();
    assert_eq!(manager.active(), (Slot::A, 1));
    let downgrade = UpdateBundle::sign(1, vec![4], "pi5", 1, false, &key).unwrap();
    assert_eq!(
        manager.stage(downgrade.verify(&key.verifying_key(), "pi5", 1).unwrap()),
        Err(HardeningError::Downgrade)
    );
}

#[test]
fn backup_authentication_restore_and_reset_erasure_are_fail_closed() {
    let key = [8; 32];
    let backup = EncryptedBackup::create(1, b"journal", &key, [9; 24]).unwrap();
    assert_eq!(backup.restore(1, &key).unwrap(), b"journal");
    assert_eq!(
        backup.restore(1, &[0; 32]),
        Err(HardeningError::BackupAuthentication)
    );
    let mut secrets = vec![vec![1, 2, 3], vec![4, 5]];
    let evidence = factory_reset(&mut secrets, "decommission").unwrap();
    assert_eq!(evidence.erased_key_count, 2);
    assert!(secrets.iter().all(Vec::is_empty));
}

#[test]
fn journal_corruption_and_unsupported_migration_are_detected() {
    let first = JournalEntry::append(1, 1, [0; 32], vec![1]).unwrap();
    let second = JournalEntry::append(2, 1, first.digest, vec![2]).unwrap();
    let journal = vec![first, second];
    assert_eq!(validate_journal(&journal), Ok(()));
    assert_eq!(migrate_journal(&journal, 1, 2).unwrap()[0].schema, 2);
    let mut corrupt = journal.clone();
    corrupt[1].payload[0] = 9;
    assert_eq!(
        validate_journal(&corrupt),
        Err(HardeningError::JournalCorrupt)
    );
    assert_eq!(
        migrate_journal(&journal, 1, 3),
        Err(HardeningError::UnsupportedMigration)
    );
}

#[test]
fn compromised_key_is_rejected_after_rotation() {
    let old = SigningKey::from_bytes(&[10; 32]);
    let new = SigningKey::from_bytes(&[11; 32]);
    let mut ring = VerificationKeyRing::default();
    ring.rotate("old", old.verifying_key()).unwrap();
    ring.compromise("old").unwrap();
    assert!(ring.trusted("old").is_none());
    ring.rotate("new", new.verifying_key()).unwrap();
    assert!(ring.trusted("new").is_some());
}
