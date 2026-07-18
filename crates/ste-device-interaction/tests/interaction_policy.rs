//! Approved projection, stale-state, touch, and fault policy acceptance tests.
use ste_device_interaction::domain::{
    ArousalBand, DomainError, InteractionSession, PeripheralId, Projection, QualityIndicator,
    RefreshPolicy, RgbColor, TouchGesture, WorkloadBand,
};

fn session() -> InteractionSession {
    InteractionSession::start("s1", RefreshPolicy::new(500, 2_000).unwrap()).unwrap()
}

#[test]
fn every_projection_has_text_and_accessible_color_without_valence_vocabulary() {
    let projections = [
        Projection::Unauthorized,
        Projection::Calibrating,
        Projection::Contaminated,
        Projection::InsufficientEvidence,
        Projection::Stale,
        Projection::Fault(PeripheralId::Display),
        Projection::SignalQuality(QualityIndicator::Good),
        Projection::UserLabeledArousal(ArousalBand::High),
        Projection::TaskWorkload(WorkloadBand::Moderate),
        Projection::AnchorConfirmed,
    ];
    for projection in projections {
        let rendered = projection.render();
        assert!(!rendered.text.is_empty());
        assert_ne!(rendered.color, RgbColor::Off);
    }
}

#[test]
fn stale_evidence_overrides_a_previously_healthy_projection() {
    let mut session = session();
    session
        .render(
            Projection::SignalQuality(QualityIndicator::Good),
            1_000,
            1_100,
        )
        .unwrap();
    session.refresh(3_001).unwrap();
    assert_eq!(session.projection(), &Projection::Stale);
}

#[test]
fn touch_anchor_requires_authorization_and_a_debounced_physical_gesture() {
    let mut session = session();
    let too_short = TouchGesture::new(1_000, 20).unwrap();
    assert_eq!(
        session.handle_touch(too_short, true),
        Err(DomainError::GestureNotDebounced)
    );
    let gesture = TouchGesture::new(2_000, 80).unwrap();
    assert_eq!(
        session.handle_touch(gesture.clone(), false),
        Err(DomainError::UnauthorizedAnchor)
    );
    assert!(session.handle_touch(gesture, true).is_ok());
    assert_eq!(session.projection(), &Projection::AnchorConfirmed);
}

#[test]
fn peripheral_fault_is_explicit_and_does_not_stop_session_supervision() {
    let mut session = session();
    session
        .record_peripheral_failure(PeripheralId::Led, "i2c timeout")
        .unwrap();
    assert!(session.is_active());
    assert_eq!(session.projection(), &Projection::Fault(PeripheralId::Led));
}
