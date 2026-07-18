use schemars::schema_for;
use serde_json::{json, Value};
use ste_contracts::{
    CaptureHealthV1, Compatibility, ContractEnvelopeV1, ContractVersion, DisplayMode,
    DisplayProjectionV1, FiniteF64, ObservationWindowClosedV1, PhysiologyEvidenceUpdatedV1,
    QualityDisposition, SchemaSupport, ValidatedCsiFrameV1,
};
use uuid::Uuid;

fn envelope<P>(payload: P) -> ContractEnvelopeV1<P> {
    ContractEnvelopeV1 {
        schema_version: ContractVersion::new(1, 0),
        event_id: Uuid::now_v7(),
        aggregate_id: Uuid::now_v7(),
        source_time_ns: 10,
        emitted_at_unix_ns: 20,
        producer_version: "ste-radio-acquisition/0.1.0".into(),
        correlation_id: Some(Uuid::now_v7()),
        causation_id: None,
        provenance_ref: "sha256:abc".into(),
        idempotency_key: "capture/frame/42".into(),
        payload,
    }
}

#[test]
fn should_round_trip_a_complete_versioned_envelope() {
    let value = envelope(CaptureHealthV1 {
        source_id: "wlan0".into(),
        frames_received: 42,
        frames_rejected: 2,
        dropped_frames: 1,
        disposition: QualityDisposition::Usable,
    });
    let encoded = serde_json::to_value(&value).unwrap();
    let decoded: ContractEnvelopeV1<CaptureHealthV1> = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn should_reject_an_envelope_when_a_required_safety_field_is_missing() {
    let mut value = serde_json::to_value(envelope(CaptureHealthV1 {
        source_id: "wlan0".into(),
        frames_received: 1,
        frames_rejected: 0,
        dropped_frames: 0,
        disposition: QualityDisposition::Usable,
    }))
    .unwrap();
    value.as_object_mut().unwrap().remove("provenance_ref");
    assert!(serde_json::from_value::<ContractEnvelopeV1<CaptureHealthV1>>(value).is_err());
}

#[test]
fn should_reject_empty_required_envelope_strings() {
    let mut value = serde_json::to_value(envelope(CaptureHealthV1 {
        source_id: "wlan0".into(),
        frames_received: 1,
        frames_rejected: 0,
        dropped_frames: 0,
        disposition: QualityDisposition::Usable,
    }))
    .unwrap();
    value["idempotency_key"] = json!("");
    assert!(serde_json::from_value::<ContractEnvelopeV1<CaptureHealthV1>>(value).is_err());
}

#[test]
fn should_never_serialize_or_deserialize_non_finite_contract_numbers() {
    assert!(FiniteF64::new(f64::NAN).is_err());
    assert!(FiniteF64::new(f64::INFINITY).is_err());
    assert!(serde_json::from_value::<FiniteF64>(json!("NaN")).is_err());
    assert_eq!(
        serde_json::to_value(FiniteF64::new(0.25).unwrap()).unwrap(),
        json!(0.25)
    );
}

#[test]
fn should_apply_explicit_major_and_minor_compatibility_rules() {
    let support = SchemaSupport::new(1, 2);
    assert_eq!(
        support.compatibility(ContractVersion::new(1, 0)),
        Compatibility::Compatible
    );
    assert_eq!(
        support.compatibility(ContractVersion::new(1, 3)),
        Compatibility::UnsupportedMinor
    );
    assert_eq!(
        support.compatibility(ContractVersion::new(2, 0)),
        Compatibility::UnsupportedMajor
    );
}

#[test]
fn should_reject_zero_major_schema_versions() {
    assert!(serde_json::from_value::<ContractVersion>(json!({"major": 0, "minor": 1})).is_err());
}

#[test]
fn should_support_representative_pipeline_contract_shapes() {
    let contracts: Vec<Value> = vec![
        serde_json::to_value(ValidatedCsiFrameV1 {
            capture_session_id: Uuid::now_v7(),
            sequence: 7,
            monotonic_time_ns: 1_000,
            center_frequency_hz: 5_180_000_000,
            bandwidth_hz: 80_000_000,
            antenna_count: 2,
            subcarrier_count: 256,
            payload_ref: "chunk:7".into(),
        })
        .unwrap(),
        serde_json::to_value(ObservationWindowClosedV1 {
            window_id: Uuid::now_v7(),
            started_at_ns: 1_000,
            ended_at_ns: 2_000,
            accepted_frames: 99,
            rejected_frames: 1,
            feature_artifact_ref: "features:1".into(),
            disposition: QualityDisposition::Usable,
        })
        .unwrap(),
        serde_json::to_value(PhysiologyEvidenceUpdatedV1 {
            assessment_id: Uuid::now_v7(),
            window_id: Uuid::now_v7(),
            respiration_hz: Some(FiniteF64::new(0.25).unwrap()),
            confidence: FiniteF64::new(0.9).unwrap(),
            disposition: QualityDisposition::Usable,
        })
        .unwrap(),
        serde_json::to_value(DisplayProjectionV1 {
            projection_id: Uuid::now_v7(),
            revision: 3,
            mode: DisplayMode::Respiration,
            headline: "Respiration signal available".into(),
            confidence: Some(FiniteF64::new(0.9).unwrap()),
            stale: false,
        })
        .unwrap(),
    ];
    assert!(contracts.iter().all(Value::is_object));
}

#[test]
fn should_reject_structurally_invalid_pipeline_contracts() {
    let invalid_frame = json!({
        "capture_session_id": Uuid::now_v7(), "sequence": 1, "monotonic_time_ns": 1,
        "center_frequency_hz": 5_180_000_000_u64, "bandwidth_hz": 80_000_000,
        "antenna_count": 0, "subcarrier_count": 256, "payload_ref": "chunk:1"
    });
    assert!(serde_json::from_value::<ValidatedCsiFrameV1>(invalid_frame).is_err());

    let invalid_health = json!({
        "source_id": " ", "frames_received": 1, "frames_rejected": 0,
        "dropped_frames": 0, "disposition": "usable"
    });
    assert!(serde_json::from_value::<CaptureHealthV1>(invalid_health).is_err());
}

#[test]
fn should_generate_a_json_schema_for_the_public_envelope() {
    let schema = schema_for!(ContractEnvelopeV1<CaptureHealthV1>);
    let value = serde_json::to_value(schema).unwrap();
    assert_eq!(value["type"], "object");
    assert!(value["required"]
        .as_array()
        .unwrap()
        .contains(&json!("schema_version")));
}
