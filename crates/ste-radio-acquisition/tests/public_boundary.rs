//! Public-boundary acceptance test.

#[test]
fn should_expose_application_boundary_without_infrastructure() {
    let marker = ste_radio_acquisition::application::boundary();
    assert_eq!(marker, ste_radio_acquisition::domain::DomainBoundary);
}
