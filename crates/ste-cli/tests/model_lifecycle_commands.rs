//! Governed model lifecycle and known-answer rollback integration tests.

use ed25519_dalek::SigningKey;
use std::{cell::RefCell, collections::BTreeSet, process::Command, rc::Rc};
use ste_cli::{
    KnownAnswerGate, LocalModelLifecycle, ModelLifecycleCommand, ModelLifecycleCommandError,
    ModelLifecycleOperations, execute_model_lifecycle_command,
};
use ste_consent_governance::domain::{
    AuthorizationRequest, DenialReason, ParticipantPseudonym, PolicyDecision, PolicyVersion,
    Purpose, SpaceId,
};
use ste_model_runtime::{
    package::{Compatibility, ModelMetadata, ModelPackage, VerifiedPackage},
    registry::{ModelRegistry, RegistryState},
};
use ste_runtime::{GovernanceGate, RequestOrigin};

fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        space: SpaceId::new("device").unwrap(),
        participants: [ParticipantPseudonym::new("operator").unwrap()]
            .into_iter()
            .collect(),
        purpose: Purpose::Wellness,
        policy_version: PolicyVersion::new(1).unwrap(),
        evaluated_at: 1,
    }
}
fn package(id: &str, key: &SigningKey) -> VerifiedPackage {
    let metadata = ModelMetadata::new(
        id,
        "deterministic-v1",
        [1; 32],
        [2; 32],
        [3; 32],
        "single-participant-still-v1",
        "study-immutable",
        "host-profile",
        "MIT",
        "ste-0.1",
        "x86_64-test",
        [4; 32],
    )
    .unwrap();
    ModelPackage::unsigned(metadata, id.as_bytes().to_vec())
        .unwrap()
        .sign(key)
        .unwrap()
        .verify(
            &key.verifying_key(),
            &Compatibility::new("ste-0.1", "x86_64-test", [1; 32]).unwrap(),
        )
        .unwrap()
}
fn promoted_registry() -> (ModelRegistry, SigningKey) {
    let key = SigningKey::from_bytes(&[7; 32]);
    let mut registry = ModelRegistry::default();
    for id in ["model-old", "model-new"] {
        registry.register(package(id, &key)).unwrap();
        registry.evaluate(id, [1; 32], "qa").unwrap();
        registry.promote(id, [2; 32], "science").unwrap();
    }
    registry.activate("model-old", [3; 32], "release").unwrap();
    (registry, key)
}
struct Kat(Rc<RefCell<BTreeSet<String>>>);
impl KnownAnswerGate for Kat {
    fn passes(&self, package: &VerifiedPackage) -> bool {
        !self
            .0
            .borrow()
            .contains(&package.package().metadata().model_id)
    }
}

#[test]
fn failed_candidate_kat_cannot_replace_active_model() {
    let (registry, _) = promoted_registry();
    let lifecycle = LocalModelLifecycle::new(
        registry,
        Kat(Rc::new(RefCell::new(BTreeSet::from(["model-new".into()])))),
        [8; 32],
        "operator",
    )
    .unwrap();
    assert_eq!(
        lifecycle.activate("model-new"),
        Err(ModelLifecycleCommandError::LifecycleGateFailed)
    );
    lifecycle.inspect(|registry| {
        assert_eq!(
            registry.active().unwrap().package().metadata().model_id,
            "model-old"
        )
    });
}

#[test]
fn post_activation_health_failure_suspends_candidate_and_restores_prior_kat() {
    let (registry, _) = promoted_registry();
    let failures = Rc::new(RefCell::new(BTreeSet::new()));
    let lifecycle =
        LocalModelLifecycle::new(registry, Kat(Rc::clone(&failures)), [8; 32], "operator").unwrap();
    lifecycle.activate("model-new").unwrap();
    failures.borrow_mut().insert("model-new".into());
    assert_eq!(
        lifecycle.health(),
        Err(ModelLifecycleCommandError::LifecycleGateFailed)
    );
    lifecycle.inspect(|registry| {
        assert_eq!(
            registry.active().unwrap().package().metadata().model_id,
            "model-old"
        );
        assert_eq!(
            registry.state("model-new").unwrap(),
            RegistryState::Suspended
        );
    });
}

struct Ops;
impl ModelLifecycleOperations for Ops {
    fn status(&self, _: &str) -> Result<String, ModelLifecycleCommandError> {
        Ok("Quarantined".into())
    }
    fn activate(&self, _: &str) -> Result<String, ModelLifecycleCommandError> {
        Ok("active".into())
    }
    fn health(&self) -> Result<String, ModelLifecycleCommandError> {
        Ok("healthy".into())
    }
    fn rollback(&self) -> Result<String, ModelLifecycleCommandError> {
        Ok("rollback".into())
    }
}

#[test]
fn governance_denial_and_direct_process_are_fail_closed() {
    let gate = GovernanceGate::new(|_| PolicyDecision::Denied(DenialReason::NotGranted));
    assert_eq!(
        execute_model_lifecycle_command(
            &ModelLifecycleCommand::Activate {
                model_id: "model-new".into()
            },
            &request(),
            RequestOrigin::LocalOperator,
            &gate,
            &Ops
        ),
        Err(ModelLifecycleCommandError::AuthorizationRequired)
    );
    let status = Command::new(env!("CARGO_BIN_EXE_ste"))
        .args(["model", "activate", "model-new"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(77));
}
