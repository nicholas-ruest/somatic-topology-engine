//! Deterministic synthetic corpus, penetration-style, watchdog, and evidence tests.

use ed25519_dalek::SigningKey;
use serde_json::{Value, json};
use ste_model_runtime::uncertainty::{CalibratedProbability, CalibrationArtifact};
use ste_radio_acquisition::replay::{ReplayLimits, parse_pcap, parse_rvcsi};
use ste_runtime::fault::{FaultHarness, FaultScenario};
use ste_runtime::ipc::*;
use ste_runtime::verification::*;

fn corpus(seed: u64, cases: usize, maximum_len: usize) -> Vec<Vec<u8>> {
    let mut state = seed;
    let mut output = Vec::new();
    for index in 0..cases {
        let length = index * 37 % maximum_len;
        let mut bytes = Vec::with_capacity(length);
        for _ in 0..length {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            bytes.push(state as u8);
        }
        output.push(bytes);
    }
    output
}

#[test]
fn parser_model_and_ipc_corpora_are_bounded_deterministic_and_panic_free() {
    let limits = ReplayLimits {
        max_input_bytes: 512,
        max_frames: 8,
        max_subcarriers: 16,
        max_record_bytes: 128,
    };
    let first = corpus(0x51de_cafe, 256, 600);
    let second = corpus(0x51de_cafe, 256, 600);
    assert_eq!(first, second);
    for bytes in &first {
        let _ = parse_rvcsi(bytes, limits);
        let _ = parse_pcap(bytes, limits);
    }
    for bits in 0_u64..4096 {
        let value = f64::from_bits(bits.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let result = CalibratedProbability::new(value);
        assert!(result.as_ref().is_ok_and(|p| p.get().is_finite()) || result.is_err());
    }
    let artifact = CalibrationArtifact {
        calibration_id: "c".into(),
        model_digest: "m".into(),
        training_partition_digest: "train".into(),
        calibration_partition_digest: "held-out".into(),
        knots: vec![(0.0, 0.0), (1.0, 1.0)],
        serving_threshold: 0.5,
        brier_score: 0.1,
        expected_calibration_error: 0.1,
        frozen: true,
    };
    for bytes in &first {
        let mut padded = [0_u8; 8];
        let count = bytes.len().min(8);
        padded[..count].copy_from_slice(&bytes[..count]);
        let _ = artifact.calibrate(f64::from_bits(u64::from_le_bytes(padded)));
    }
    let mut auth = IpcAuthenticator::default();
    auth.register(1000, "synthetic-test-secret", OperatorRole::Viewer)
        .unwrap();
    let mut server = IpcServer::new(auth, Echo, 512, 5, 300).unwrap();
    for (index, bytes) in first.iter().enumerate() {
        let response = server.handle_frame(PeerIdentity { uid: 1000 }, bytes, index as u64);
        assert_ne!(response.exit, TypedExit::Success);
    }
}

struct Echo;
impl IpcHandler for Echo {
    fn handle(&mut self, _: OperatorRole, _: &IpcCommand, _: &Value) -> Result<Value, IpcError> {
        Ok(json!({"ok": true}))
    }
}

#[test]
fn penetration_style_forged_admin_and_secret_fields_do_not_cross_boundary() {
    let mut auth = IpcAuthenticator::default();
    auth.register(1000, "synthetic-test-secret", OperatorRole::Viewer)
        .unwrap();
    let mut server = IpcServer::new(auth, Echo, 4096, 10, 4).unwrap();
    let request = IpcRequest {
        schema: IPC_SCHEMA_V1.into(),
        request_id: "attack".into(),
        idempotency_key: "attack-key".into(),
        nonce: "attack-nonce-0001".into(),
        issued_at_unix_seconds: 10,
        credential: "synthetic-test-secret".into(),
        command: IpcCommand::Reset,
        parameters: json!({"role":"administrator", "password":"canary"}),
    };
    let wire = IpcClient::new("/run/ste/control.sock", 4096)
        .unwrap()
        .encode(&request)
        .unwrap();
    let response = server.handle_frame(PeerIdentity { uid: 1000 }, &wire, 10);
    assert_eq!(response.exit, TypedExit::Forbidden);
    assert!(!serde_json::to_string(&response).unwrap().contains("canary"));
}

#[test]
fn low_voltage_thermal_bus_storage_ap_and_power_faults_meet_synthetic_controls() {
    for scenario in [
        FaultScenario::LowVoltage,
        FaultScenario::ThermalPressure,
        FaultScenario::PeripheralBusFailure,
        FaultScenario::DiskFull,
        FaultScenario::StorageCorruption,
        FaultScenario::AccessPointLoss,
        FaultScenario::PowerInterruption,
    ] {
        let outcome = FaultHarness::default().inject(scenario);
        assert!(outcome.synthetic_only);
        assert!(outcome.meets_expected_response());
    }
}

#[test]
fn watchdog_expires_only_missing_critical_heartbeat_deterministically() {
    let mut watchdog = Watchdog::new(100).unwrap();
    watchdog.heartbeat("capture", 10).unwrap();
    watchdog.heartbeat("storage", 20).unwrap();
    assert!(watchdog.expired(110).is_empty());
    assert_eq!(watchdog.expired(121), ["capture", "storage"]);
    watchdog.heartbeat("capture", 121).unwrap();
    watchdog.heartbeat("storage", 121).unwrap();
    assert_eq!(watchdog.expired(121), Vec::<String>::new());
    assert_eq!(
        watchdog.heartbeat("capture", 120),
        Err(VerificationError::InvalidHeartbeat)
    );
}

#[test]
fn signed_evidence_is_tamper_evident_and_cannot_claim_external_or_hil_testing() {
    let key = SigningKey::from_bytes(&[17; 32]);
    let evidence = VerificationEvidence::sign(
        "phase-17-synthetic",
        "revision-test",
        vec!["H-010".into(), "H-004".into(), "H-010".into()],
        "verification-test",
        &key,
    )
    .unwrap();
    evidence.verify(&key.verifying_key()).unwrap();
    assert!(!evidence.external_penetration_test);
    assert!(!evidence.physical_hil);
    assert_eq!(evidence.passed_controls, ["H-004", "H-010"]);
    let mut tampered = evidence;
    tampered.passed_controls.push("H-999".into());
    assert_eq!(
        tampered.verify(&key.verifying_key()),
        Err(VerificationError::InvalidSignature)
    );
}
