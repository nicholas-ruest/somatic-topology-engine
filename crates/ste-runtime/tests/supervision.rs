//! Outside-in acceptance tests for bounded supervision and safe shutdown.

use ste_runtime::{
    ChannelError, CircuitState, Criticality, HealthState, OverflowPolicy, RestartPolicy,
    SafeStateReason, Supervisor, TaskFailure,
};

#[test]
fn bounded_queue_rejects_optional_work_but_never_sheds_accepted_critical_work() {
    let mut supervisor = Supervisor::<u8>::new(2, OverflowPolicy::RejectNewest);
    assert_eq!(supervisor.publish(1, Criticality::Optional), Ok(()));
    assert_eq!(supervisor.publish(2, Criticality::Optional), Ok(()));
    assert_eq!(
        supervisor.publish(3, Criticality::Optional),
        Err(ChannelError::ShedOptional)
    );
    assert_eq!(
        supervisor.publish(9, Criticality::Critical),
        Err(ChannelError::CriticalBackpressure)
    );
    assert_eq!(supervisor.drain(), vec![1, 2]);
    assert_eq!(supervisor.health().shed_optional_events, 1);
    assert_eq!(supervisor.health().shed_critical_events, 0);
}

#[test]
fn drop_oldest_policy_only_evicts_optional_work() {
    let mut supervisor = Supervisor::new(2, OverflowPolicy::DropOldestOptional);
    supervisor
        .publish("critical", Criticality::Critical)
        .unwrap();
    supervisor.publish("old", Criticality::Optional).unwrap();
    supervisor.publish("new", Criticality::Optional).unwrap();
    assert_eq!(supervisor.drain(), vec!["critical", "new"]);
}

#[test]
fn crashed_optional_task_degrades_without_stopping_critical_tasks() {
    let mut supervisor = Supervisor::<()>::new(1, OverflowPolicy::RejectNewest);
    supervisor.register_task("capture", Criticality::Critical, RestartPolicy::never());
    supervisor.register_task("sidecar", Criticality::Optional, RestartPolicy::never());
    supervisor
        .record_failure("sidecar", TaskFailure::Crashed("exit 1".into()))
        .unwrap();
    assert_eq!(supervisor.health().state, HealthState::Degraded);
    assert!(supervisor.task("capture").unwrap().running);
    assert!(!supervisor.task("sidecar").unwrap().running);
}

#[test]
fn restart_budget_opens_circuit_after_bounded_retries() {
    let mut supervisor = Supervisor::<()>::new(1, OverflowPolicy::RejectNewest);
    supervisor.register_task("adapter", Criticality::Optional, RestartPolicy::bounded(2));
    for _ in 0..3 {
        supervisor
            .record_failure("adapter", TaskFailure::Crashed("boom".into()))
            .unwrap();
    }
    let task = supervisor.task("adapter").unwrap();
    assert_eq!(task.restart_count, 2);
    assert_eq!(task.circuit, CircuitState::Open);
    assert!(!task.running);
}

#[test]
fn cancellation_and_shutdown_are_coordinated_and_verifiably_safe() {
    let mut supervisor = Supervisor::<()>::new(1, OverflowPolicy::RejectNewest);
    supervisor.register_task("capture", Criticality::Critical, RestartPolicy::never());
    supervisor.register_task("ui", Criticality::Optional, RestartPolicy::never());
    let token = supervisor.cancellation_token();
    supervisor.shutdown().unwrap();
    assert!(token.is_cancelled());
    assert!(supervisor.tasks().all(|task| !task.running));
    assert_eq!(
        supervisor.health().safe_state,
        Some(SafeStateReason::CoordinatedShutdown)
    );
}

#[test]
fn clock_discontinuity_and_low_resources_force_safe_degradation() {
    let mut supervisor = Supervisor::<()>::new(1, OverflowPolicy::RejectNewest);
    supervisor.report_clock_discontinuity(20);
    supervisor.report_low_resources("disk reserve exhausted");
    assert_eq!(supervisor.health().state, HealthState::Safe);
    assert_eq!(
        supervisor.health().safe_state,
        Some(SafeStateReason::LowResources)
    );
    assert_eq!(supervisor.health().clock_discontinuities, 1);
}
