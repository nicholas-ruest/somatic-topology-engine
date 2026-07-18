//! CLI-level deterministic radio-to-observation replay tests.

use std::cell::Cell;
use std::path::Path;

use ste_cli::{
    ObservationReplayCommand, ReplayCommandError, ReplayInput, execute_observation_replay,
};
use ste_consent_governance::domain::{
    AuthorizationRequest, DenialReason, ParticipantPseudonym, PolicyDecision, PolicyVersion,
    Purpose, SpaceId,
};
use ste_runtime::{GovernanceGate, RequestOrigin};
use ste_signal_observation::ContentAddressedEvidenceRepository;

struct Input {
    bytes: Vec<u8>,
    reads: Cell<u32>,
}
impl ReplayInput for Input {
    fn read_bounded(&self, _: &Path, _: usize) -> Result<Vec<u8>, ReplayCommandError> {
        self.reads.set(self.reads.get() + 1);
        Ok(self.bytes.clone())
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

fn fixture() -> Vec<u8> {
    let mut bytes = b"RVCSIv1\0".to_vec();
    for sequence in 1..=8_u64 {
        let mut record = Vec::new();
        record.extend(sequence.to_le_bytes());
        record.extend((1_000_000_000 + sequence * 100_000_000).to_le_bytes());
        record.extend(5_180_000_000_u64.to_le_bytes());
        record.extend(20_000_000_u32.to_le_bytes());
        record.push(1);
        record.extend(1_u16.to_le_bytes());
        record.extend((sequence as f64 / 10.0).to_bits().to_le_bytes());
        record.extend(0.25_f64.to_bits().to_le_bytes());
        bytes.extend((record.len() as u32).to_le_bytes());
        bytes.extend(record);
    }
    bytes
}

#[test]
fn identical_radio_replay_is_idempotent_and_has_stable_digest() {
    let command = ObservationReplayCommand::parse([
        "capture.rvcsi",
        "--format",
        "rvcsi",
        "--partition",
        "development",
    ])
    .unwrap();
    let input = Input {
        bytes: fixture(),
        reads: Cell::new(0),
    };
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| PolicyDecision::Authorized);
    let repository = ContentAddressedEvidenceRepository::default();
    let first = execute_observation_replay(
        &command,
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &input,
        &repository,
    )
    .unwrap();
    let second = execute_observation_replay(
        &command,
        &request(),
        RequestOrigin::LocalOperator,
        &gate,
        &input,
        &repository,
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.source_frames, 8);
    assert_eq!(repository.len(), Ok(1));
}

#[test]
fn denied_observation_replay_does_not_read_capture() {
    let command = ObservationReplayCommand::parse([
        "capture.rvcsi",
        "--format",
        "rvcsi",
        "--partition",
        "validation",
    ])
    .unwrap();
    let input = Input {
        bytes: fixture(),
        reads: Cell::new(0),
    };
    let gate = GovernanceGate::new(|_: &AuthorizationRequest| {
        PolicyDecision::Denied(DenialReason::NotGranted)
    });
    assert_eq!(
        execute_observation_replay(
            &command,
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &input,
            &ContentAddressedEvidenceRepository::default(),
        ),
        Err(ReplayCommandError::AuthorizationRequired)
    );
    assert_eq!(input.reads.get(), 0);
}

#[test]
fn production_partition_is_not_selectable_from_offline_cli() {
    assert_eq!(
        ObservationReplayCommand::parse([
            "capture.rvcsi",
            "--format",
            "rvcsi",
            "--partition",
            "production",
        ]),
        Err(ReplayCommandError::InvalidArguments)
    );
}
