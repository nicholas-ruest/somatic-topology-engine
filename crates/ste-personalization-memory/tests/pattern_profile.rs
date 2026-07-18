//! Aggregate invariants for participant-scoped append-only memory.
use ste_personalization_memory::domain::{
    AnchorLabel, DomainError, EmbeddingVector, ParticipantPseudonym, PartitionRole, PatternProfile,
    Provenance,
};

fn participant(value: &str) -> ParticipantPseudonym {
    ParticipantPseudonym::new(value).unwrap()
}
fn provenance() -> Provenance {
    Provenance::new("assessment-1", "observation-1", "session-1").unwrap()
}

#[test]
fn anchors_and_feedback_corrections_append_without_rewriting_history() {
    let mut profile = PatternProfile::create("profile-1", participant("p1")).unwrap();
    let anchor = profile
        .record_anchor(
            "a1",
            AnchorLabel::new("focused").unwrap(),
            EmbeddingVector::new(vec![0.1, 0.2]).unwrap(),
            provenance(),
            PartitionRole::Development,
        )
        .unwrap();
    profile
        .record_feedback("f1", &anchor, 0.2, None, PartitionRole::Development)
        .unwrap();
    profile
        .record_feedback("f2", &anchor, 0.9, Some("f1"), PartitionRole::Development)
        .unwrap();
    assert_eq!(profile.feedback().len(), 2);
    assert_eq!(profile.feedback()[0].reward(), 0.2);
}

#[test]
fn evaluation_and_test_partitions_can_never_mutate_memory() {
    for role in [PartitionRole::Evaluation, PartitionRole::Test] {
        let mut profile = PatternProfile::create("profile", participant("p1")).unwrap();
        assert_eq!(
            profile.record_anchor(
                "a",
                AnchorLabel::new("x").unwrap(),
                EmbeddingVector::new(vec![1.0]).unwrap(),
                provenance(),
                role
            ),
            Err(DomainError::ReadOnlyPartition)
        );
    }
}

#[test]
fn adaptation_requires_parent_exact_feedback_and_prospective_evidence_for_improvement() {
    let mut profile = PatternProfile::create("profile", participant("p1")).unwrap();
    let anchor = profile
        .record_anchor(
            "a",
            AnchorLabel::new("x").unwrap(),
            EmbeddingVector::new(vec![1.0]).unwrap(),
            provenance(),
            PartitionRole::Development,
        )
        .unwrap();
    profile
        .record_feedback("f", &anchor, 1.0, None, PartitionRole::Development)
        .unwrap();
    profile
        .build_adaptation("v1", None, vec!["f".into()], false)
        .unwrap();
    assert_eq!(
        profile.build_adaptation("v2", Some("v1"), vec!["f".into()], true),
        Err(DomainError::ProspectiveEvidenceRequired)
    );
    profile
        .record_prospective_evaluation("eval-1", "v1")
        .unwrap();
    profile
        .build_adaptation("v2", Some("v1"), vec!["f".into()], true)
        .unwrap();
    assert_eq!(profile.adaptations().len(), 2);
}

#[test]
fn participant_forget_tombstones_and_removes_all_retrievable_payloads() {
    let mut profile = PatternProfile::create("profile", participant("p1")).unwrap();
    profile
        .record_anchor(
            "a",
            AnchorLabel::new("x").unwrap(),
            EmbeddingVector::new(vec![1.0]).unwrap(),
            provenance(),
            PartitionRole::Development,
        )
        .unwrap();
    profile.forget("erasure-proof-1").unwrap();
    assert!(profile.is_forgotten());
    assert!(profile.anchors().is_empty());
    assert_eq!(
        profile.record_prospective_evaluation("x", "v"),
        Err(DomainError::ProfileForgotten)
    );
}
