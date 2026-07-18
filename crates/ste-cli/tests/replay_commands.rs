//! Outside-in authorization and deterministic replay command tests.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::process::Command;

use ste_cli::{ReplayCommand, ReplayCommandError, ReplayFormat, ReplayInput, execute_replay};
use ste_consent_governance::domain::{
    AuthorizationRequest, DenialReason, ParticipantPseudonym, PolicyDecision, PolicyVersion,
    Purpose, SpaceId,
};
use ste_runtime::{GovernanceGate, RequestOrigin};

struct Input {
    bytes: Vec<u8>,
    reads: Cell<u32>,
}

impl ReplayInput for Input {
    fn read_bounded(&self, _: &Path, maximum: usize) -> Result<Vec<u8>, ReplayCommandError> {
        self.reads.set(self.reads.get() + 1);
        if self.bytes.len() > maximum {
            Err(ReplayCommandError::InputTooLarge)
        } else {
            Ok(self.bytes.clone())
        }
    }
}

fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("room-a").unwrap(),
        participants: [ParticipantPseudonym::new("participant-a").unwrap()]
            .into_iter()
            .collect(),
        purpose: Purpose::Research,
        policy_version: PolicyVersion::new(1).unwrap(),
        evaluated_at: 10,
    }
}

fn record(sequence: u64) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend(sequence.to_le_bytes());
    record.extend((100 + sequence).to_le_bytes());
    record.extend(5_180_000_000_u64.to_le_bytes());
    record.extend(20_000_000_u32.to_le_bytes());
    record.push(1);
    record.extend(1_u16.to_le_bytes());
    record.extend(1.0_f64.to_bits().to_le_bytes());
    record.extend((-1.0_f64).to_bits().to_le_bytes());
    record
}

fn replay_bytes() -> Vec<u8> {
    let mut bytes = b"RVCSIv1\0".to_vec();
    for record in [record(1), record(3)] {
        bytes.extend((record.len() as u32).to_le_bytes());
        bytes.extend(record);
    }
    bytes
}

#[test]
fn policy_denial_happens_before_capture_file_read() {
    let input = Input {
        bytes: replay_bytes(),
        reads: Cell::new(0),
    };
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| {
        PolicyDecision::Denied(DenialReason::NotGranted)
    });
    let command = ReplayCommand {
        input: PathBuf::from("sensitive.rvcsi"),
        format: ReplayFormat::Rvcsi,
        json: true,
    };

    assert_eq!(
        execute_replay(
            &command,
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &input
        ),
        Err(ReplayCommandError::AuthorizationRequired)
    );
    assert_eq!(input.reads.get(), 0);
}

#[test]
fn authorized_replay_preserves_accepted_and_missing_statistics() {
    let input = Input {
        bytes: replay_bytes(),
        reads: Cell::new(0),
    };
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| PolicyDecision::Authorized);
    let command = ReplayCommand::parse(["capture.rvcsi", "--format", "rvcsi", "--json"]).unwrap();

    let summary = execute_replay(
        &command,
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &input,
    )
    .unwrap();
    assert_eq!(summary.accepted, 2);
    assert_eq!(summary.missing, 1);
    assert_eq!(summary.rejected_malformed, 0);
}

#[test]
fn parser_requires_explicit_format_and_rejects_extra_arguments() {
    assert_eq!(
        ReplayCommand::parse(["capture.rvcsi"]),
        Err(ReplayCommandError::InvalidArguments)
    );
    assert_eq!(
        ReplayCommand::parse(["capture", "--format", "raw"]),
        Err(ReplayCommandError::InvalidArguments)
    );
}

#[test]
fn direct_replay_cli_fails_closed_without_authenticated_ipc() {
    let output = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["replay", "capture.rvcsi", "--format", "rvcsi", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(77));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "active authorization required"
    );
}
