//! Reference vector adapter scoping, append-only history, and erasure tests.
use ste_personalization_memory::{
    AppendOnlyPatternRepository, ReferenceVectorMemory, VectorAdapterError,
    application::{PatternProfileRepository, ScopedVectorKey, ScopedVectorQuery, VectorMemory},
    domain::{
        AnchorLabel, EmbeddingVector, ParticipantPseudonym, PartitionRole, PatternProfile,
        Provenance,
    },
};
fn p(v: &str) -> ParticipantPseudonym {
    ParticipantPseudonym::new(v).unwrap()
}
#[test]
fn retrieval_is_deterministic_and_strictly_participant_scoped() {
    let mut memory = ReferenceVectorMemory::default();
    for (participant, id, v) in [
        (p("p1"), "a1", vec![1., 0.]),
        (p("p1"), "a2", vec![0., 1.]),
        (p("p2"), "secret", vec![1., 0.]),
    ] {
        memory
            .insert(
                &ScopedVectorKey {
                    participant,
                    anchor_id: id.into(),
                },
                &EmbeddingVector::new(v).unwrap(),
            )
            .unwrap();
    }
    let participant = p("p1");
    let query = EmbeddingVector::new(vec![1., 0.]).unwrap();
    let found = memory
        .search(ScopedVectorQuery {
            participant: &participant,
            embedding: &query,
            limit: 10,
        })
        .unwrap();
    assert_eq!(
        found
            .iter()
            .map(|m| m.anchor_id.as_str())
            .collect::<Vec<_>>(),
        vec!["a1", "a2"]
    );
    assert!(found.iter().all(|m| m.anchor_id != "secret"));
}
#[test]
fn erasure_destroys_key_and_rebuild_cannot_resurrect_payload() {
    let mut memory = ReferenceVectorMemory::default();
    let erased = p("p1");
    let retained = p("p2");
    let vector = EmbeddingVector::new(vec![1., 0.]).unwrap();
    for (participant, id) in [(&erased, "gone"), (&retained, "kept")] {
        memory
            .insert(
                &ScopedVectorKey {
                    participant: participant.clone(),
                    anchor_id: id.into(),
                },
                &vector,
            )
            .unwrap();
    }
    assert_eq!(memory.journal_len(), 2);
    memory.erase_participant(&erased).unwrap();
    assert!(!memory.has_key(&erased));
    memory.rebuild().unwrap();
    assert!(
        memory
            .search(ScopedVectorQuery {
                participant: &erased,
                embedding: &vector,
                limit: 10
            })
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        memory
            .search(ScopedVectorQuery {
                participant: &retained,
                embedding: &vector,
                limit: 10
            })
            .unwrap()[0]
            .anchor_id,
        "kept"
    );
    assert_eq!(
        memory.insert(
            &ScopedVectorKey {
                participant: erased,
                anchor_id: "return".into()
            },
            &vector
        ),
        Err(VectorAdapterError::Forgotten)
    );
}
#[test]
fn corrections_and_forget_are_append_only_authority() {
    let mut repository = AppendOnlyPatternRepository::default();
    let mut profile = PatternProfile::create("profile", p("p1")).unwrap();
    let anchor = profile
        .record_anchor(
            "a",
            AnchorLabel::new("focus").unwrap(),
            EmbeddingVector::new(vec![1.]).unwrap(),
            Provenance::new("assessment", "observation", "session").unwrap(),
            PartitionRole::Development,
        )
        .unwrap();
    repository.save(&profile).unwrap();
    profile
        .record_feedback("f1", &anchor, 0.1, None, PartitionRole::Development)
        .unwrap();
    profile
        .record_feedback("f2", &anchor, 0.9, Some("f1"), PartitionRole::Development)
        .unwrap();
    repository.save(&profile).unwrap();
    profile.forget("key-erasure-receipt").unwrap();
    repository.save(&profile).unwrap();
    assert_eq!(repository.history("profile").len(), 3);
    assert_eq!(repository.history("profile")[1].feedback().len(), 2);
    assert!(repository.latest("profile").unwrap().is_forgotten());
}
