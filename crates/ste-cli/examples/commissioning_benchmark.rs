//! Synthetic commissioning workflow benchmark; not physical site qualification.
use ed25519_dalek::SigningKey;
use std::time::Instant;
use ste_commissioning::{CapabilityCoverage, CheckKind, CheckOutcome, CommissioningSession};
fn main() {
    let key = SigningKey::from_bytes(&[22; 32]);
    let iterations = 1_000_u64;
    let start = Instant::now();
    for i in 0..iterations {
        let mut session = CommissioningSession::start(
            format!("c{i}"),
            "synthetic-site",
            "simulator-profile",
            ["presence-v1".into(), "respiration-v1".into()],
        )
        .unwrap();
        for check in CheckKind::ALL {
            session
                .record_check(
                    check,
                    CheckOutcome::passed(format!("synthetic-{check:?}")).unwrap(),
                )
                .unwrap();
        }
        session
            .record_coverage(
                CapabilityCoverage::new("presence-v1", true, "synthetic-coverage").unwrap(),
            )
            .unwrap();
        session
            .record_coverage(
                CapabilityCoverage::new("respiration-v1", false, "synthetic-below-threshold")
                    .unwrap(),
            )
            .unwrap();
        let signed = session.qualify(i, "test-key", &key).unwrap();
        signed.verify(&key.verifying_key()).unwrap();
        assert!(signed.enables("presence-v1"));
        assert!(!signed.enables("respiration-v1"));
    }
    let elapsed = start.elapsed();
    println!("host_arch={}", std::env::consts::ARCH);
    println!("synthetic_sessions={iterations}");
    println!("elapsed_ns={}", elapsed.as_nanos());
    println!(
        "sessions_per_second={:.3}",
        iterations as f64 / elapsed.as_secs_f64()
    );
    println!("threshold_weakening=prohibited");
    println!("physical_site_qualification=not_performed");
    println!("reference_pi_status=pending");
}
