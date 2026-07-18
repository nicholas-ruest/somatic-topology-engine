//! SLO boundary and deterministic fault-response acceptance tests.

use ste_runtime::fault::{ExpectedResponse, FaultHarness, FaultScenario};
use ste_runtime::slo::{
    Celsius, Kibibytes, KibibytesPerHour, Milliseconds, Percent, Ratio, RegressionTolerance,
    SloBudgets, SloMeasurements, SloMetric, Watts, compare_regression,
};
use ste_runtime::{HealthState, SafeStateReason};

fn budgets() -> SloBudgets {
    SloBudgets {
        capture_continuity_min: Ratio::new(0.99).unwrap(),
        valid_window_coverage_min: Ratio::new(0.95).unwrap(),
        queue_delay_max: Milliseconds::new(25.0).unwrap(),
        projection_freshness_max: Milliseconds::new(250.0).unwrap(),
        startup_max: Milliseconds::new(2_000.0).unwrap(),
        recovery_max: Milliseconds::new(5_000.0).unwrap(),
        cpu_max: Percent::new(80.0).unwrap(),
        rss_max: Kibibytes::new(512_000.0).unwrap(),
        storage_growth_max: KibibytesPerHour::new(10_000.0).unwrap(),
        temperature_max: Celsius::new(80.0).unwrap(),
        power_max: Watts::new(15.0).unwrap(),
    }
}

fn measurements() -> SloMeasurements {
    SloMeasurements {
        capture_continuity: Ratio::new(0.995).unwrap(),
        valid_window_coverage: Ratio::new(0.97).unwrap(),
        queue_delay: Milliseconds::new(20.0).unwrap(),
        projection_freshness: Milliseconds::new(200.0).unwrap(),
        startup: Milliseconds::new(1_500.0).unwrap(),
        recovery: Milliseconds::new(4_000.0).unwrap(),
        cpu: Percent::new(70.0).unwrap(),
        rss: Kibibytes::new(400_000.0).unwrap(),
        storage_growth: KibibytesPerHour::new(8_000.0).unwrap(),
        temperature: Celsius::new(70.0).unwrap(),
        power: Watts::new(12.0).unwrap(),
    }
}

#[test]
fn complete_owned_slo_set_passes_or_fails_deterministically_at_inclusive_boundaries() {
    let report = budgets().evaluate(measurements());
    assert_eq!(report.results.len(), 11);
    assert!(report.passed());

    let mut failing = measurements();
    failing.capture_continuity = Ratio::new(0.98).unwrap();
    failing.queue_delay = Milliseconds::new(25.001).unwrap();
    let report = budgets().evaluate(failing);
    assert!(!report.passed());
    assert_eq!(
        report
            .results
            .iter()
            .filter(|result| !result.passed)
            .map(|result| result.metric)
            .collect::<Vec<_>>(),
        vec![SloMetric::CaptureContinuity, SloMetric::QueueDelay]
    );

    let exact = SloMeasurements {
        capture_continuity: budgets().capture_continuity_min,
        valid_window_coverage: budgets().valid_window_coverage_min,
        queue_delay: budgets().queue_delay_max,
        projection_freshness: budgets().projection_freshness_max,
        startup: budgets().startup_max,
        recovery: budgets().recovery_max,
        cpu: budgets().cpu_max,
        rss: budgets().rss_max,
        storage_growth: budgets().storage_growth_max,
        temperature: budgets().temperature_max,
        power: budgets().power_max,
    };
    assert!(budgets().evaluate(exact).passed());
}

#[test]
fn typed_measurements_reject_nonfinite_negative_and_out_of_range_values() {
    assert!(Ratio::new(1.01).is_err());
    assert!(Milliseconds::new(-1.0).is_err());
    assert!(Percent::new(f64::NAN).is_err());
    assert!(Celsius::new(f64::INFINITY).is_err());
    assert!(Watts::new(-0.1).is_err());
}

#[test]
fn regression_comparison_knows_minimum_and_maximum_metric_direction() {
    let baseline = measurements();
    let mut candidate = baseline;
    candidate.capture_continuity = Ratio::new(0.97).unwrap();
    candidate.queue_delay = Milliseconds::new(30.0).unwrap();
    candidate.cpu = Percent::new(60.0).unwrap();
    let comparison = compare_regression(
        baseline,
        candidate,
        RegressionTolerance {
            relative: Ratio::new(0.01).unwrap(),
            absolute: 0.0,
        },
    )
    .unwrap();
    assert!(
        !comparison
            .iter()
            .find(|result| result.metric == SloMetric::CaptureContinuity)
            .unwrap()
            .passed
    );
    assert!(
        !comparison
            .iter()
            .find(|result| result.metric == SloMetric::QueueDelay)
            .unwrap()
            .passed
    );
    assert!(
        comparison
            .iter()
            .find(|result| result.metric == SloMetric::Cpu)
            .unwrap()
            .passed
    );
}

#[test]
fn recoverable_input_faults_degrade_without_fabricating_hardware_evidence() {
    for scenario in [
        FaultScenario::PacketLoss,
        FaultScenario::MalformedFrame,
        FaultScenario::Overload,
        FaultScenario::OptionalTaskDeath,
    ] {
        let outcome = FaultHarness::default().inject(scenario);
        assert_eq!(outcome.expected, ExpectedResponse::ContinueDegraded);
        assert!(outcome.meets_expected_response());
        assert!(outcome.synthetic_only);
    }
}

#[test]
fn ap_loss_and_time_jump_disable_capture_while_preserving_degraded_operation() {
    for scenario in [FaultScenario::AccessPointLoss, FaultScenario::TimeJump] {
        let outcome = FaultHarness::default().inject(scenario);
        assert_eq!(outcome.expected, ExpectedResponse::DisableCaptureDegraded);
        assert!(outcome.meets_expected_response());
        assert!(!outcome.capture_enabled);
    }
}

#[test]
fn integrity_resource_and_power_faults_enter_capture_disabled_safe_state() {
    for scenario in [
        FaultScenario::DiskFull,
        FaultScenario::StorageCorruption,
        FaultScenario::CriticalTaskDeath,
        FaultScenario::LowVoltage,
        FaultScenario::ThermalPressure,
        FaultScenario::PowerInterruption,
    ] {
        let outcome = FaultHarness::default().inject(scenario);
        assert_eq!(outcome.expected, ExpectedResponse::CaptureDisabledSafe);
        assert!(outcome.meets_expected_response());
        assert_eq!(outcome.health, HealthState::Safe);
        assert!(outcome.safe_state.is_some());
    }
}

#[test]
fn overload_never_records_a_dropped_critical_event_and_supervisor_ports_are_observable() {
    let mut harness = FaultHarness::default();
    let outcome = harness.inject(FaultScenario::Overload);
    assert!(outcome.meets_expected_response());
    assert_eq!(harness.supervisor_health().shed_critical_events, 0);

    let critical = FaultHarness::default().inject(FaultScenario::CriticalTaskDeath);
    assert_eq!(
        critical.safe_state,
        Some(SafeStateReason::CriticalTaskFailed)
    );
}
