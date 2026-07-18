//! Current-host simulator benchmark; not CrowPi hardware evidence.
use std::time::Instant;
use ste_device_interaction::{
    AppendOnlyInteractionAudit, SimulatorDisplay, SimulatorLed,
    application::{DisplayPort, InteractionAuditJournal, LedPort},
    domain::{InteractionEvent, Projection, QualityIndicator},
};
fn main() {
    let mut display = SimulatorDisplay::default();
    let mut led = SimulatorLed::default();
    let mut audit = AppendOnlyInteractionAudit::default();
    let iterations = 100_000_u64;
    let start = Instant::now();
    for i in 0..iterations {
        let p = if i % 3 == 0 {
            Projection::Calibrating
        } else if i % 3 == 1 {
            Projection::SignalQuality(QualityIndicator::Good)
        } else {
            Projection::Stale
        };
        let r = p.render();
        display.display(&r).unwrap();
        led.set_color(r.color).unwrap();
        audit
            .append(&InteractionEvent::ProjectionRendered {
                projection: p,
                rendered_at: i,
            })
            .unwrap();
    }
    let elapsed = start.elapsed();
    println!("host_arch={}", std::env::consts::ARCH);
    println!("updates={iterations}");
    println!("elapsed_ns={}", elapsed.as_nanos());
    println!(
        "updates_per_second={:.3}",
        iterations as f64 / elapsed.as_secs_f64()
    );
    println!("snapshots={}", display.snapshots().len());
    println!("audit_events={}", audit.events().len());
    println!("physical_crowpi_status=unqualified_pending_revision_hil");
}
