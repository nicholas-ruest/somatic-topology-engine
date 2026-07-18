//! Honest exact-scope blocked readiness decision for missing production evidence.
use ed25519_dalek::SigningKey;
use std::collections::{BTreeMap, BTreeSet};
use ste_commercial_readiness::{ReadinessEngine, ReadinessReview, ReleaseScope, ResidualRisk};
fn main() {
    let scope = ReleaseScope::new(
        "crowpi-unqualified",
        "workspace-candidate",
        BTreeSet::from(["no-production-model".into()]),
        BTreeSet::from(["no-approved-market".into()]),
        "unqualified-envelope",
        "draft-support",
    )
    .unwrap();
    let review = ReadinessReview {
        scope: scope.clone(),
        scope_binding: "candidate-unapproved".into(),
        evidence: BTreeMap::new(),
        residual_risks: vec![
            ResidualRisk::new("physical-hil-missing", 5, false, "phase17-pending").unwrap(),
        ],
        exceptions: vec![],
    };
    let key = SigningKey::from_bytes(&[33; 32]);
    let decision =
        ReadinessEngine::decide(review, 1, 2, "automated-blocking-review", &key).unwrap();
    decision
        .verify(&key.verifying_key(), &scope, "candidate-unapproved", 1)
        .unwrap();
    assert!(!decision.authorizes_release(1));
    println!("{}", serde_json::to_string(&decision).unwrap());
}
