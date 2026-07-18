//! Hostile-input and authenticated local IPC acceptance tests.

use serde_json::{Value, json};
use ste_runtime::ipc::*;

#[derive(Default)]
struct Handler {
    calls: usize,
}
impl IpcHandler for Handler {
    fn handle(
        &mut self,
        _: OperatorRole,
        _: &IpcCommand,
        parameters: &Value,
    ) -> Result<Value, IpcError> {
        self.calls += 1;
        Ok(json!({"accepted": true, "nested_token": parameters.get("token").cloned()}))
    }
}

fn request(command: IpcCommand) -> IpcRequest {
    IpcRequest {
        schema: IPC_SCHEMA_V1.into(),
        request_id: "request-1".into(),
        idempotency_key: "idem-key-0001".into(),
        nonce: "unique-nonce-0001".into(),
        issued_at_unix_seconds: 1_000,
        credential: "correct horse battery staple".into(),
        command,
        parameters: json!({"token": "must-not-leak", "value": 7}),
    }
}
fn server() -> IpcServer<Handler> {
    let mut auth = IpcAuthenticator::default();
    auth.register(1000, "correct horse battery staple", OperatorRole::Operator)
        .unwrap();
    IpcServer::new(auth, Handler::default(), 4096, 30, 16).unwrap()
}

#[test]
fn authenticated_authorized_request_has_stable_redacted_json_and_typed_exit() {
    let client = IpcClient::new("/run/ste/control.sock", 4096).unwrap();
    let wire = client.encode(&request(IpcCommand::CaptureTest)).unwrap();
    let response = server().handle_frame(PeerIdentity { uid: 1000 }, &wire, 1_010);
    assert_eq!(response.exit, TypedExit::Success);
    assert_eq!(response.exit.code(), 0);
    let json = serde_json::to_string(&response).unwrap();
    assert!(!json.contains("must-not-leak"));
    assert!(json.contains("[REDACTED]"));
    assert!(!format!("{:?}", request(IpcCommand::Status)).contains("correct horse"));
}

#[test]
fn wrong_peer_secret_and_insufficient_role_are_distinct_typed_denials() {
    let client = IpcClient::new("/run/ste/control.sock", 4096).unwrap();
    let mut wrong = request(IpcCommand::Status);
    wrong.credential = "incorrect credential value".into();
    assert_eq!(
        server()
            .handle_frame(
                PeerIdentity { uid: 1000 },
                &client.encode(&wrong).unwrap(),
                1_000
            )
            .exit,
        TypedExit::Unauthorized
    );
    let admin = request(IpcCommand::Reset);
    assert_eq!(
        server()
            .handle_frame(
                PeerIdentity { uid: 1000 },
                &client.encode(&admin).unwrap(),
                1_000
            )
            .exit,
        TypedExit::Forbidden
    );
    assert_eq!(
        server()
            .handle_frame(
                PeerIdentity { uid: 9999 },
                &client.encode(&request(IpcCommand::Status)).unwrap(),
                1_000
            )
            .exit,
        TypedExit::Unauthorized
    );
}

#[test]
fn idempotent_retry_returns_same_response_but_nonce_replay_and_key_collision_fail() {
    let client = IpcClient::new("/run/ste/control.sock", 4096).unwrap();
    let mut server = server();
    let first_request = request(IpcCommand::Status);
    let frame = client.encode(&first_request).unwrap();
    let first = server.handle_frame(PeerIdentity { uid: 1000 }, &frame, 1_000);
    assert_eq!(
        server.handle_frame(PeerIdentity { uid: 1000 }, &frame, 1_001),
        first
    );
    let mut replay = first_request.clone();
    replay.idempotency_key = "another-idem-key".into();
    assert_eq!(
        server
            .handle_frame(
                PeerIdentity { uid: 1000 },
                &client.encode(&replay).unwrap(),
                1_001
            )
            .exit,
        TypedExit::Conflict
    );
    let mut collision = first_request;
    collision.nonce = "different-nonce-01".into();
    collision.command = IpcCommand::Doctor;
    assert_eq!(
        server
            .handle_frame(
                PeerIdentity { uid: 1000 },
                &client.encode(&collision).unwrap(),
                1_001
            )
            .exit,
        TypedExit::Conflict
    );
}

#[test]
fn oversized_malformed_unknown_field_deep_and_expired_inputs_fail_before_handler() {
    let peer = PeerIdentity { uid: 1000 };
    let mut server = server();
    assert_eq!(
        server.handle_frame(peer, &[b'x'; 4097], 1_000).exit,
        TypedExit::InvalidInput
    );
    assert_eq!(
        server.handle_frame(peer, b"{not-json", 1_000).exit,
        TypedExit::InvalidInput
    );
    let unknown = br#"{"schema":"ste-ipc-v1","request_id":"x","idempotency_key":"12345678","nonce":"1234567890123456","issued_at_unix_seconds":1000,"credential":"correct horse battery staple","command":"status","parameters":{},"unexpected":true}"#;
    assert_eq!(
        server.handle_frame(peer, unknown, 1_000).exit,
        TypedExit::InvalidInput
    );
    let client = IpcClient::new("/run/ste/control.sock", 4096).unwrap();
    assert_eq!(
        server
            .handle_frame(
                peer,
                &client.encode(&request(IpcCommand::Status)).unwrap(),
                2_000
            )
            .exit,
        TypedExit::InvalidInput
    );
    let mut deep = request(IpcCommand::Status);
    deep.parameters = json!([[[[[[[[[[1]]]]]]]]]]);
    assert_eq!(client.encode(&deep), Err(IpcError::MalformedRequest));
}

#[test]
fn client_rejects_relative_socket_and_stable_schema_rejects_unknown_versions() {
    assert!(matches!(
        IpcClient::new("relative.sock", 4096),
        Err(IpcError::InvalidConfiguration)
    ));
    let client = IpcClient::new("/run/ste/control.sock", 4096).unwrap();
    let mut future = request(IpcCommand::Status);
    future.schema = "ste-ipc-v2".into();
    assert_eq!(client.encode(&future), Err(IpcError::MalformedRequest));
}
