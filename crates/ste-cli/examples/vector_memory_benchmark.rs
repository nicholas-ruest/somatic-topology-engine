//! Synthetic current-host reference-vector benchmark; not RuVector/RVF.
use std::time::Instant;
use ste_personalization_memory::{
    ReferenceVectorMemory,
    application::{ScopedVectorKey, ScopedVectorQuery, VectorMemory},
    domain::{EmbeddingVector, ParticipantPseudonym},
};
fn main() {
    let mut memory = ReferenceVectorMemory::default();
    let p1 = ParticipantPseudonym::new("p1").unwrap();
    let p2 = ParticipantPseudonym::new("p2").unwrap();
    let records = 10_000_u64;
    let start = Instant::now();
    for i in 0..records {
        let participant = if i % 2 == 0 { p1.clone() } else { p2.clone() };
        let vector = EmbeddingVector::new(vec![i as f32, 1., 2., 3., 4., 5., 6., 7.]).unwrap();
        memory
            .insert(
                &ScopedVectorKey {
                    participant,
                    anchor_id: format!("a{i}"),
                },
                &vector,
            )
            .unwrap();
    }
    let query = EmbeddingVector::new(vec![1., 1., 2., 3., 4., 5., 6., 7.]).unwrap();
    for _ in 0..100 {
        assert_eq!(
            memory
                .search(ScopedVectorQuery {
                    participant: &p1,
                    embedding: &query,
                    limit: 10
                })
                .unwrap()
                .len(),
            10
        );
    }
    memory.erase_participant(&p1).unwrap();
    assert!(
        memory
            .search(ScopedVectorQuery {
                participant: &p1,
                embedding: &query,
                limit: 10
            })
            .unwrap()
            .is_empty()
    );
    let elapsed = start.elapsed();
    println!("host_arch={}", std::env::consts::ARCH);
    println!("records={records}");
    println!("dimensions=8");
    println!("elapsed_ns={}", elapsed.as_nanos());
    println!(
        "records_per_second={:.3}",
        records as f64 / elapsed.as_secs_f64()
    );
    println!("adapter=deterministic_rust_reference");
    println!("ruvector_rvf_status=unverified_not_integrated");
    println!("cryptographic_erasure_rebuild=passed");
    println!("reference_pi_status=pending");
}
