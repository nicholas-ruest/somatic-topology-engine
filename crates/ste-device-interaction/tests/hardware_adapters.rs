//! Simulator, HIL profile, debounce, and peripheral isolation acceptance tests.

use ste_device_interaction::hardware::*;

fn projection() -> HardwareProjection {
    HardwareProjection {
        label: "Signal ready".into(),
        rgb: (0, 80, 255),
        brightness_percent: 40,
        sensing_visible: true,
    }
}

#[test]
fn simulator_produces_deterministic_snapshots_and_environment_is_only_a_covariate() {
    let mut first = HardwareSimulator::default();
    let mut second = HardwareSimulator::default();
    first.environment = Some((22.5, 45.0));
    second.environment = first.environment;
    first.render(&projection()).unwrap();
    second.render(&projection()).unwrap();
    first.set_rgb(projection().rgb, 40).unwrap();
    second.set_rgb(projection().rgb, 40).unwrap();
    first.set_visible(true).unwrap();
    second.set_visible(true).unwrap();
    assert_eq!(first.display, second.display);
    assert_eq!(first.rgb, second.rgb);
    assert_eq!(
        first.read_environment(100).unwrap(),
        second.read_environment(100).unwrap()
    );
    let covariate = first.read_environment(101).unwrap();
    assert_eq!(covariate.observed_at_ns, 101);
    assert_eq!(covariate.relative_humidity_percent, 45.0);
}

#[test]
fn bouncing_touch_emits_one_pressed_and_one_released_edge() {
    let mut simulator = HardwareSimulator::with_touch_debounce(10);
    simulator.queue_touch(10, true);
    simulator.queue_touch(12, false);
    simulator.queue_touch(14, true);
    assert_eq!(simulator.poll_touch(20).unwrap(), None);
    assert_eq!(simulator.poll_touch(24).unwrap(), Some(TouchEvent::Pressed));
    assert_eq!(simulator.poll_touch(25).unwrap(), None);
    simulator.queue_touch(30, false);
    simulator.queue_touch(32, true);
    simulator.queue_touch(34, false);
    assert_eq!(simulator.poll_touch(40).unwrap(), None);
    assert_eq!(
        simulator.poll_touch(44).unwrap(),
        Some(TouchEvent::Released)
    );
}

#[test]
fn hil_fixture_rejects_unverified_or_conflicting_profiles() {
    let fixture = CrowPiProfile {
        profile_version: 1,
        board_revision: "CrowPi-L verified-fixture-2026-01".into(),
        oled_i2c_address: 0x3c,
        rgb_pins: [17, 27, 22],
        touch_pin: 5,
        dht11_pin: 6,
        sensing_indicator_pin: 13,
        physical_off_pin: 19,
    };
    assert_eq!(fixture.clone().validate().unwrap(), fixture);
    let mut conflict = fixture.clone();
    conflict.touch_pin = conflict.dht11_pin;
    assert_eq!(conflict.validate(), Err(HardwareError::InvalidProfile));
    let mut unversioned = fixture;
    unversioned.profile_version = 0;
    assert_eq!(unversioned.validate(), Err(HardwareError::InvalidProfile));
}

struct Display(Result<(), HardwareError>);
impl DisplayPort for Display {
    fn render(&mut self, _: &HardwareProjection) -> Result<(), HardwareError> {
        self.0
    }
}
struct Rgb(Result<(), HardwareError>);
impl RgbPort for Rgb {
    fn set_rgb(&mut self, _: (u8, u8, u8), _: u8) -> Result<(), HardwareError> {
        self.0
    }
}
struct Indicator(Result<(), HardwareError>);
impl SensingIndicatorPort for Indicator {
    fn set_visible(&mut self, _: bool) -> Result<(), HardwareError> {
        self.0
    }
}

#[test]
fn failed_display_does_not_prevent_rgb_or_visible_indicator_attempts() {
    let snapshot = render_fault_isolated(
        &mut Display(Err(HardwareError::Io)),
        &mut Rgb(Ok(())),
        &mut Indicator(Ok(())),
        &projection(),
    );
    assert_eq!(snapshot.display, Err(HardwareError::Io));
    assert_eq!(snapshot.rgb, Ok(()));
    assert_eq!(snapshot.indicator, Ok(()));
}

#[test]
fn physical_off_and_independent_fault_injection_are_fail_observable() {
    let mut simulator = HardwareSimulator::default();
    simulator.physical_off = true;
    simulator.inject_fault("dht11", HardwareError::Unavailable);
    assert_eq!(simulator.is_physical_off(), Ok(true));
    assert_eq!(
        simulator.read_environment(10),
        Err(HardwareError::Unavailable)
    );
    assert_eq!(simulator.render(&projection()), Ok(()));
}
