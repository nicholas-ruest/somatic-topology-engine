//! Participant isolation, poisoning resistance, and candidate lifecycle tests.

use ste_personalization_memory::adaptation::*;
use ste_personalization_memory::retrieval::*;

fn anchor(id: &str, participant: &str, partition: PartitionRole, embedding: Vec<f32>) -> Anchor {
    Anchor {
        id: id.into(),
        participant_id: participant.into(),
        source_id: format!("source-{id}"),
        partition,
        embedding,
        active: true,
    }
}

#[test]
fn retrieval_is_deterministic_participant_scoped_and_training_only() {
    let memory = ParticipantVectorMemory::new(vec![
        anchor("b", "p1", PartitionRole::Training, vec![1.0, 0.0]),
        anchor("a", "p1", PartitionRole::Training, vec![1.0, 0.0]),
        anchor("foreign", "p2", PartitionRole::Training, vec![1.0, 0.0]),
        anchor(
            "held-out",
            "p1",
            PartitionRole::ProspectiveEvaluation,
            vec![1.0, 0.0],
        ),
        anchor("test", "p1", PartitionRole::Test, vec![1.0, 0.0]),
    ])
    .unwrap();
    let first = memory.retrieve("p1", &[1.0, 0.0], 10).unwrap();
    let second = memory.retrieve("p1", &[1.0, 0.0], 10).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        first
            .iter()
            .map(|hit| hit.anchor_id.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert!(
        memory
            .retrieve("unknown", &[1.0, 0.0], 10)
            .unwrap()
            .is_empty()
    );
}

fn feedback(
    id: &str,
    participant: &str,
    partition: PartitionRole,
    quality: f32,
) -> QualifiedFeedback {
    QualifiedFeedback {
        id: id.into(),
        participant_id: participant.into(),
        anchor_id: format!("anchor-{id}"),
        partition,
        quality,
    }
}

fn policy() -> AdaptationPolicy {
    AdaptationPolicy {
        minimum_evidence: 2,
        minimum_quality: 0.8,
        maximum_candidates_per_window: 1,
        minimum_improvement: 0.05,
    }
}

#[test]
fn candidate_has_exact_lineage_requires_prospective_improvement_and_can_rollback() {
    let mut factory = CandidateFactory::new(policy(), 7).unwrap();
    let mut candidate = factory
        .create(
            "candidate-1",
            "p1",
            7,
            &[
                feedback("z", "p1", PartitionRole::Training, 0.9),
                feedback("a", "p1", PartitionRole::Training, 1.0),
            ],
        )
        .unwrap();
    assert_eq!(candidate.lineage.feedback_ids, ["a", "z"]);
    assert_eq!(
        candidate.promote(),
        Err(AdaptationError::NotProspectivelyValidated)
    );
    assert_eq!(
        factory.compare_prospectively(&mut candidate, 0.8, 0.84, vec!["eval-1".into()]),
        Err(AdaptationError::NoProspectiveImprovement)
    );
    factory
        .compare_prospectively(
            &mut candidate,
            0.8,
            0.9,
            vec!["eval-1".into(), "eval-2".into()],
        )
        .unwrap();
    candidate.promote().unwrap();
    candidate.rollback("online regression").unwrap();
    assert!(matches!(candidate.state, CandidateState::RolledBack { .. }));
}

#[test]
fn poisoning_cross_user_leakage_and_rate_abuse_fail_closed() {
    let mut factory = CandidateFactory::new(policy(), 7).unwrap();
    assert_eq!(
        factory.create(
            "bad-quality",
            "p1",
            7,
            &[
                feedback("a", "p1", PartitionRole::Training, f32::NAN),
                feedback("b", "p1", PartitionRole::Training, 1.0),
            ]
        ),
        Err(AdaptationError::UnqualifiedFeedback)
    );
    assert_eq!(
        factory.create(
            "cross-user",
            "p1",
            7,
            &[
                feedback("a", "p1", PartitionRole::Training, 1.0),
                feedback("b", "p2", PartitionRole::Training, 1.0),
            ]
        ),
        Err(AdaptationError::CrossParticipantEvidence)
    );
    assert_eq!(
        factory.create(
            "leak",
            "p1",
            7,
            &[
                feedback("a", "p1", PartitionRole::Training, 1.0),
                feedback("b", "p1", PartitionRole::Test, 1.0),
            ]
        ),
        Err(AdaptationError::PartitionLeakage)
    );
    assert_eq!(
        factory.create(
            "duplicates",
            "p1",
            7,
            &[
                feedback("a", "p1", PartitionRole::Training, 1.0),
                feedback("a", "p1", PartitionRole::Training, 1.0),
            ]
        ),
        Err(AdaptationError::DuplicateEvidence)
    );
    let mut admitted = factory
        .create(
            "good",
            "p1",
            7,
            &[
                feedback("a", "p1", PartitionRole::Training, 1.0),
                feedback("b", "p1", PartitionRole::Training, 1.0),
            ],
        )
        .unwrap();
    assert_eq!(
        factory.create(
            "too-many",
            "p1",
            7,
            &[
                feedback("c", "p1", PartitionRole::Training, 1.0),
                feedback("d", "p1", PartitionRole::Training, 1.0),
            ]
        ),
        Err(AdaptationError::RateLimited)
    );
    assert_eq!(
        factory.compare_prospectively(&mut admitted, 0.5, 0.8, vec!["a".into()]),
        Err(AdaptationError::PartitionLeakage)
    );
}
