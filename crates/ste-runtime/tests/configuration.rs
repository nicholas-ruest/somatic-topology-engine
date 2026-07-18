//! Outside-in verification of versioned, layered runtime configuration.

use ed25519_dalek::{Signer, SigningKey};
use ste_runtime::config::{
    ConfigError, ConfigLayer, ConfigurationLoader, Ed25519ProfileVerifier, LayerSource,
    PartialConfig, SignedProfile,
};

const TEST_SIGNING_KEY: [u8; 32] = [7; 32];

fn signed_profile(config: &PartialConfig) -> (SignedProfile, Ed25519ProfileVerifier) {
    let signing_key = SigningKey::from_bytes(&TEST_SIGNING_KEY);
    let payload = serde_json::to_vec(config).expect("profile serializes");
    let signature = signing_key.sign(&payload).to_bytes().to_vec();
    (
        SignedProfile { payload, signature },
        Ed25519ProfileVerifier::new(signing_key.verifying_key()),
    )
}

#[test]
fn defaults_are_capture_disabled_and_resource_bounded() {
    let config = ConfigurationLoader::default()
        .load()
        .expect("safe defaults");

    assert!(!config.capture_enabled);
    assert!(!config.production_mode);
    assert_eq!(config.schema_version, 2);
    assert!((1..=4096).contains(&config.pipeline.queue_capacity));
    assert_eq!(config.secrets.provider_key, "local-development");
    assert!(!format!("{config:?}").contains("local-development"));
}

#[test]
fn explicit_layers_follow_profile_file_env_cli_precedence() {
    let profile_values = PartialConfig {
        display_name: Some("profile".into()),
        queue_capacity: Some(32),
        ..PartialConfig::default()
    };
    let (profile, verifier) = signed_profile(&profile_values);

    let config = ConfigurationLoader::default()
        .with_signed_profile(profile, &verifier)
        .expect("valid signature")
        .with_layer(ConfigLayer::new(
            LayerSource::DeviceFile,
            PartialConfig {
                display_name: Some("file".into()),
                queue_capacity: Some(64),
                ..PartialConfig::default()
            },
        ))
        .with_layer(ConfigLayer::new(
            LayerSource::Environment,
            PartialConfig {
                display_name: Some("env".into()),
                queue_capacity: Some(128),
                ..PartialConfig::default()
            },
        ))
        .with_layer(ConfigLayer::new(
            LayerSource::CommandLine,
            PartialConfig {
                display_name: Some("cli".into()),
                queue_capacity: Some(256),
                ..PartialConfig::default()
            },
        ))
        .load()
        .expect("valid layered config");

    assert_eq!(config.display_name, "cli");
    assert_eq!(config.pipeline.queue_capacity, 256);
}

#[test]
fn ambient_environment_is_never_read_and_sensitive_overrides_fail_closed() {
    let explicit = ConfigLayer::new(
        LayerSource::Environment,
        PartialConfig {
            capture_enabled: Some(true),
            ..PartialConfig::default()
        },
    );
    let error = ConfigurationLoader::default()
        .with_layer(explicit)
        .load()
        .expect_err("environment cannot enable capture");

    assert_eq!(
        error,
        ConfigError::ForbiddenOverride {
            source: LayerSource::Environment,
            field: "capture_enabled",
        }
    );
}

#[test]
fn validation_rejects_unbounded_and_inconsistent_configuration() {
    let error = ConfigurationLoader::default()
        .with_layer(ConfigLayer::new(
            LayerSource::DeviceFile,
            PartialConfig {
                queue_capacity: Some(0),
                ..PartialConfig::default()
            },
        ))
        .load()
        .expect_err("zero capacity rejected");
    assert!(matches!(error, ConfigError::Validation(_)));

    let error = ConfigurationLoader::default()
        .with_layer(ConfigLayer::new(
            LayerSource::CommandLine,
            PartialConfig {
                production_mode: Some(true),
                developer_relaxations: Some(true),
                ..PartialConfig::default()
            },
        ))
        .load()
        .expect_err("production cannot relax controls");
    assert!(matches!(error, ConfigError::Validation(_)));
}

#[test]
fn version_one_documents_migrate_explicitly_and_unknown_versions_are_rejected() {
    let v1 = r#"{"schema_version":1,"name":"legacy","channel_capacity":77}"#;
    let migrated = ConfigLayer::from_json(LayerSource::DeviceFile, v1).expect("v1 migrates");
    let config = ConfigurationLoader::default()
        .with_layer(migrated)
        .load()
        .expect("migrated configuration validates");
    assert_eq!(config.display_name, "legacy");
    assert_eq!(config.pipeline.queue_capacity, 77);

    let unsupported = ConfigLayer::from_json(
        LayerSource::DeviceFile,
        r#"{"schema_version":99,"display_name":"future"}"#,
    )
    .expect_err("unsupported major version rejected");
    assert_eq!(unsupported, ConfigError::UnsupportedSchemaVersion(99));
}

#[test]
fn signed_profiles_verify_before_parsing_and_tampering_fails_closed() {
    let values = PartialConfig {
        display_name: Some("signed".into()),
        ..PartialConfig::default()
    };
    let (mut profile, verifier) = signed_profile(&values);
    profile.payload.push(b' ');

    let error = ConfigurationLoader::default()
        .with_signed_profile(profile, &verifier)
        .expect_err("tampered profile must fail");
    assert_eq!(error, ConfigError::InvalidProfileSignature);
}

#[test]
fn secret_material_is_isolated_redacted_and_excluded_from_digest() {
    let base = ConfigurationLoader::default()
        .with_secret_provider_key("first-secret")
        .load()
        .expect("valid config");
    let rotated = ConfigurationLoader::default()
        .with_secret_provider_key("rotated-secret")
        .load()
        .expect("valid config");

    assert_eq!(base.non_secret_digest(), rotated.non_secret_digest());
    assert!(!base.to_string().contains("first-secret"));
    assert!(!format!("{base:?}").contains("first-secret"));
    let serialized = serde_json::to_string(&base).expect("non-secret config serializes");
    assert!(!serialized.contains("secret"));
}

#[test]
fn non_secret_digest_is_deterministic_and_changes_with_effective_values() {
    let first = ConfigurationLoader::default().load().expect("defaults");
    let second = ConfigurationLoader::default().load().expect("defaults");
    assert_eq!(first.non_secret_digest(), second.non_secret_digest());

    let changed = ConfigurationLoader::default()
        .with_layer(ConfigLayer::new(
            LayerSource::DeviceFile,
            PartialConfig {
                display_name: Some("other".into()),
                ..PartialConfig::default()
            },
        ))
        .load()
        .expect("changed config");
    assert_ne!(first.non_secret_digest(), changed.non_secret_digest());
}
