//! Deterministic synthetic replay acceptance tests.

use ste_runtime::{DeterministicClock, SyntheticEvent, SyntheticPipeline};

#[test]
fn identical_inputs_replay_to_identical_outputs() {
    let inputs = vec![3_i64, 1, 4, 1, 5];
    let first = SyntheticPipeline::new(DeterministicClock::new(1_000, 10)).replay(&inputs);
    let second = SyntheticPipeline::new(DeterministicClock::new(1_000, 10)).replay(&inputs);
    assert_eq!(first, second);
    assert_eq!(first.last().unwrap().event_time_millis, 1_040);
}

#[test]
fn replay_preserves_order_and_uses_monotonic_event_time() {
    let result = SyntheticPipeline::new(DeterministicClock::new(0, 5)).replay(&[7, 8, 9]);
    assert_eq!(
        result,
        vec![
            SyntheticEvent::new(0, 7),
            SyntheticEvent::new(5, 8),
            SyntheticEvent::new(10, 9),
        ]
    );
    assert!(
        result
            .windows(2)
            .all(|window| window[0].event_time_millis < window[1].event_time_millis)
    );
}

#[test]
fn test_clock_can_advance_without_wall_clock_or_sleep() {
    let mut clock = DeterministicClock::new(50, 2);
    assert_eq!(clock.now_millis(), 50);
    clock.advance_steps(4).unwrap();
    assert_eq!(clock.now_millis(), 58);
}

#[test]
fn clock_overflow_is_typed_instead_of_wrapping() {
    let mut clock = DeterministicClock::new(u64::MAX - 1, 2);
    assert_eq!(
        clock.tick().unwrap_err().to_string(),
        "deterministic clock overflow"
    );
}
