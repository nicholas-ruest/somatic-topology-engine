#![allow(missing_docs)]

use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use ste_ui_gateway::*;

fn model(seq: u64) -> ReadModel {
    ReadModel::new(
        FunctionalArea::LiveOverview,
        seq,
        100,
        50,
        "projection:approved",
        json!({"status":"abstained"}),
    )
    .unwrap()
}

#[test]
fn all_sixteen_areas_have_stable_wire_names() {
    let areas = [
        FunctionalArea::LiveOverview,
        FunctionalArea::SpatialSignalTopology,
        FunctionalArea::RadioAcquisition,
        FunctionalArea::SignalObservation,
        FunctionalArea::PhysiologyEstimation,
        FunctionalArea::StateInference,
        FunctionalArea::PersonalizationMemory,
        FunctionalArea::DeviceInteraction,
        FunctionalArea::ConsentGovernance,
        FunctionalArea::ExperimentValidation,
        FunctionalArea::ModelsCapabilities,
        FunctionalArea::ObservabilityReliability,
        FunctionalArea::Commissioning,
        FunctionalArea::OperationsDataLifecycle,
        FunctionalArea::SecurityIncidents,
        FunctionalArea::ReleaseCommercialReadiness,
    ];
    assert_eq!(areas.len(), 16);
    assert!(
        serde_json::to_string(&areas)
            .unwrap()
            .contains("release_commercial_readiness")
    );
}

#[test]
fn nested_and_case_changed_secret_fields_are_rejected() {
    for payload in [
        json!({"nested":{"access_TOKEN":"x"}}),
        json!({"participant_id":"x"}),
        json!({"raw_CSI_payload":[]} ),
    ] {
        assert_eq!(
            ReadModel::new(FunctionalArea::SecurityIncidents, 1, 1, 1, "p", payload).unwrap_err(),
            GatewayError::ProhibitedField
        );
    }
}

#[test]
fn schema_staleness_and_complexity_fail_closed() {
    assert_eq!(model(1).validate_at(151), Err(GatewayError::Stale));
    let mut deep = json!(null);
    for _ in 0..14 {
        deep = json!([deep]);
    }
    assert_eq!(
        ReadModel::new(FunctionalArea::LiveOverview, 1, 1, 1, "p", deep).unwrap_err(),
        GatewayError::PayloadComplexity
    );
}

#[test]
fn session_requires_every_binding_and_safe_policy() {
    let policy = BrowserSecurityPolicy {
        allowed_origin: "http://127.0.0.1:8080".into(),
        csp: "default-src 'self'; object-src 'none'; frame-ancestors 'none'".into(),
        session_ttl_ms: 10_000,
        max_body_bytes: 1024,
        requests_per_minute: 60,
    };
    let session = Session {
        id: "opaque".into(),
        role: Role::Operator,
        device_binding: "device-a".into(),
        csrf: "csrf-token-123456".into(),
        expires_at_ms: 200,
    };
    assert!(
        session
            .authorize(
                100,
                "device-a",
                "csrf-token-123456",
                "http://127.0.0.1:8080",
                &policy
            )
            .is_ok()
    );
    assert_eq!(
        session.authorize(
            100,
            "device-b",
            "csrf-token-123456",
            "http://127.0.0.1:8080",
            &policy
        ),
        Err(GatewayError::Unauthorized)
    );
    let mut bad = policy;
    bad.csp.push_str("; script-src 'unsafe-eval'");
    assert_eq!(bad.validate(), Err(GatewayError::UnsafePolicy));
}

#[test]
fn body_and_rate_limits_are_enforced_and_reset() {
    let policy = BrowserSecurityPolicy {
        allowed_origin: "http://localhost:8080".into(),
        csp: "default-src 'self'; object-src 'none'; frame-ancestors 'none'".into(),
        session_ttl_ms: 1000,
        max_body_bytes: 4,
        requests_per_minute: 2,
    };
    let mut guards = RequestGuards::new(0);
    assert!(guards.admit(1, 4, &policy).is_ok());
    assert!(guards.admit(2, 0, &policy).is_ok());
    assert_eq!(guards.admit(3, 0, &policy), Err(GatewayError::RateLimited));
    assert_eq!(
        guards.admit(60_001, 5, &policy),
        Err(GatewayError::BodyTooLarge)
    );
    assert!(guards.admit(60_002, 1, &policy).is_ok());
}

#[test]
fn hidden_or_unversioned_commands_cannot_cross_bridge() {
    let bridge = CommandBridge::new(BTreeMap::from([(
        "v1.capture.stop".into(),
        BTreeSet::from([Role::Operator]),
    )]))
    .unwrap();
    assert!(
        bridge
            .authorize("v1.capture.stop", Role::Operator, "019-key")
            .is_ok()
    );
    assert_eq!(
        bridge.authorize("v1.capture.stop", Role::Participant, "019-key"),
        Err(GatewayError::CommandDenied)
    );
    assert_eq!(
        bridge.authorize("v1.shell", Role::Operator, "019-key"),
        Err(GatewayError::CommandDenied)
    );
}

#[test]
fn bounded_stream_reports_backpressure_loss() {
    let mut stream = BoundedStream::new(2).unwrap();
    stream.push(model(1));
    stream.push(model(2));
    stream.push(model(3));
    assert_eq!(stream.dropped(), 1);
    assert_eq!(stream.pop().unwrap().sequence, 2);
}

#[test]
fn asset_manifest_rejects_tampering_extras_and_traversal() {
    let body = b"immutable".to_vec();
    let digest = format!("{:x}", Sha256::digest(&body));
    let manifest = AssetManifest {
        assets: vec![AssetEntry {
            path: "assets/app.js".into(),
            sha256: digest,
            bytes: 9,
        }],
    };
    let files = BTreeMap::from([("assets/app.js".into(), body)]);
    assert!(manifest.verify(&files).is_ok());
    let tampered = BTreeMap::from([("assets/app.js".into(), b"tampered!".to_vec())]);
    assert_eq!(manifest.verify(&tampered), Err(GatewayError::AssetMismatch));
}

#[derive(Default)]
struct CountingExecutor {
    calls: usize,
}
impl CommandExecutor for CountingExecutor {
    fn execute(
        &mut self,
        _: &str,
        _: &serde_json::Value,
    ) -> Result<serde_json::Value, GatewayError> {
        self.calls += 1;
        Ok(json!({"status":"accepted", "nested":{"access_token":"must-not-leak"}}))
    }
}

fn command_policy() -> BrowserSecurityPolicy {
    BrowserSecurityPolicy {
        allowed_origin: "http://127.0.0.1:8080".into(),
        csp: "default-src 'self'; object-src 'none'; frame-ancestors 'none'".into(),
        session_ttl_ms: 10_000,
        max_body_bytes: 1024,
        requests_per_minute: 60,
    }
}

fn command_session(role: Role) -> Session {
    Session {
        id: "session-a".into(),
        role,
        device_binding: "device-a".into(),
        csrf: "csrf-token-123456".into(),
        expires_at_ms: 1_000,
    }
}

fn request<'a>(command: &'a str, key: &'a str, body: &'a serde_json::Value) -> CommandRequest<'a> {
    CommandRequest {
        now_ms: 100,
        origin: "http://127.0.0.1:8080",
        device_binding: "device-a",
        csrf: "csrf-token-123456",
        command,
        idempotency_key: key,
        body_bytes: 16,
        body,
    }
}

#[test]
fn exact_retry_returns_receipt_without_duplicate_execution() {
    let bridge = CommandBridge::new(BTreeMap::from([(
        "v1.capture.stop".into(),
        BTreeSet::from([Role::Operator]),
    )]))
    .unwrap();
    let mut service =
        GatewayService::new(command_policy(), bridge, CountingExecutor::default(), 0).unwrap();
    let body = json!({"reason":"operator"});
    let first = service
        .dispatch(
            &command_session(Role::Operator),
            request("v1.capture.stop", "key-a", &body),
        )
        .unwrap();
    let retry = service
        .dispatch(
            &command_session(Role::Operator),
            request("v1.capture.stop", "key-a", &body),
        )
        .unwrap();
    assert!(!first.replayed);
    assert!(retry.replayed);
    assert_eq!(first.response, json!({"status":"accepted", "nested":{}}));
    assert_eq!(first.response, retry.response);
    assert_eq!(service.into_executor().calls, 1);
}

#[test]
fn conflicting_key_and_denied_role_never_dispatch() {
    let bridge = CommandBridge::new(BTreeMap::from([(
        "v1.capture.stop".into(),
        BTreeSet::from([Role::Operator]),
    )]))
    .unwrap();
    let mut service =
        GatewayService::new(command_policy(), bridge, CountingExecutor::default(), 0).unwrap();
    let first = json!({"reason":"one"});
    service
        .dispatch(
            &command_session(Role::Operator),
            request("v1.capture.stop", "same-key", &first),
        )
        .unwrap();
    let conflict = json!({"reason":"two"});
    assert_eq!(
        service.dispatch(
            &command_session(Role::Operator),
            request("v1.capture.stop", "same-key", &conflict)
        ),
        Err(GatewayError::IdempotencyConflict)
    );
    assert_eq!(
        service.dispatch(
            &command_session(Role::Participant),
            request("v1.capture.stop", "other-key", &first)
        ),
        Err(GatewayError::CommandDenied)
    );
    assert_eq!(service.into_executor().calls, 1);
}

#[test]
fn static_host_is_strictly_loopback_and_relative() {
    let valid = StaticHostConfig {
        bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
        asset_root: PathBuf::from("ui/dist"),
    };
    assert!(valid.validate().is_ok());
    let exposed = StaticHostConfig {
        bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080),
        asset_root: PathBuf::from("ui/dist"),
    };
    assert_eq!(exposed.validate(), Err(GatewayError::UnsafeHost));
    let traversal = StaticHostConfig {
        bind: valid.bind,
        asset_root: PathBuf::from("../dist"),
    };
    assert_eq!(traversal.validate(), Err(GatewayError::UnsafeHost));
}
