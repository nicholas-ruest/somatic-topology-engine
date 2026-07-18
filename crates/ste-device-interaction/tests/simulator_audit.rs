//! Simulator snapshots, audit, accessibility, and isolated fault tests.
use ste_device_interaction::{
    AppendOnlyInteractionAudit, CrowPiPhysicalProfile, SimulatorDisplay, SimulatorLed,
    SnapshotSessionRepository,
    application::{DisplayPort, InteractionAuditJournal, InteractionSessionRepository, LedPort},
    domain::{
        InteractionSession, PeripheralId, Projection, QualityIndicator, RefreshPolicy, RgbColor,
    },
};
#[test]
fn simulator_snapshot_is_accessible_and_deterministic() {
    let rendered = Projection::SignalQuality(QualityIndicator::Good).render();
    let mut display = SimulatorDisplay::default();
    let mut led = SimulatorLed::default();
    display.display(&rendered).unwrap();
    led.set_color(rendered.color).unwrap();
    assert_eq!(
        display.snapshots(),
        &[("Signal quality good".into(), RgbColor::Cyan)]
    );
    assert_eq!(led.colors(), &[RgbColor::Cyan]);
    assert!(!display.snapshots()[0].0.contains("valence"));
}
#[test]
fn one_peripheral_fault_does_not_freeze_healthy_port_or_supervision() {
    let mut display = SimulatorDisplay::default();
    let mut led = SimulatorLed::default();
    display.fail_next();
    let rendered = Projection::Calibrating.render();
    assert!(display.display(&rendered).is_err());
    led.set_color(rendered.color).unwrap();
    assert_eq!(led.colors(), &[RgbColor::Blue]);
    let mut session =
        InteractionSession::start("s", RefreshPolicy::new(500, 2000).unwrap()).unwrap();
    session
        .record_peripheral_failure(PeripheralId::Display, "simulated fault")
        .unwrap();
    assert!(session.is_active());
}
#[test]
fn snapshots_and_events_are_persisted_without_overwriting_audit() {
    let mut session =
        InteractionSession::start("s", RefreshPolicy::new(500, 2000).unwrap()).unwrap();
    session.render(Projection::Calibrating, 1, 1).unwrap();
    let mut repository = SnapshotSessionRepository::default();
    repository.save(&session).unwrap();
    repository.save(&session).unwrap();
    let mut audit = AppendOnlyInteractionAudit::default();
    for event in session.events() {
        audit.append(event).unwrap();
    }
    assert_eq!(repository.get("s"), Some(&session));
    assert_eq!(audit.events(), session.events());
}
#[test]
fn physical_profile_is_explicitly_unqualified_without_revision_and_hil() {
    assert!(!CrowPiPhysicalProfile::default().is_qualified());
}
