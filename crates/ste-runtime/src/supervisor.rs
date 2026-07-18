//! Deterministic bounded queues and task supervision.
#![allow(missing_docs)]

use crate::health::{HealthState, RuntimeHealth, SafeStateReason};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Criticality {
    Critical,
    Optional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverflowPolicy {
    RejectNewest,
    DropOldestOptional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestartPolicy {
    max_restarts: u32,
}

impl RestartPolicy {
    #[must_use]
    pub const fn never() -> Self {
        Self { max_restarts: 0 }
    }

    #[must_use]
    pub const fn bounded(max_restarts: u32) -> Self {
        Self { max_restarts }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskFailure {
    Crashed(String),
    Cancelled,
    TimedOut,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelError {
    ZeroCapacity,
    ShedOptional,
    CriticalBackpressure,
}

impl fmt::Display for ChannelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ZeroCapacity => "channel capacity must be non-zero",
            Self::ShedOptional => "optional event shed under backpressure",
            Self::CriticalBackpressure => "critical event requires producer backpressure",
        })
    }
}

impl Error for ChannelError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupervisorError {
    UnknownTask(String),
    AlreadyShutdown,
}

impl fmt::Display for SupervisorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTask(name) => write!(formatter, "unknown supervised task: {name}"),
            Self::AlreadyShutdown => formatter.write_str("runtime is already shut down"),
        }
    }
}

impl Error for SupervisorError {}

#[derive(Clone, Debug)]
pub struct TaskStatus {
    pub name: String,
    pub criticality: Criticality,
    pub running: bool,
    pub restart_count: u32,
    pub circuit: CircuitState,
    restart_policy: RestartPolicy,
    pub last_failure: Option<TaskFailure>,
}

#[derive(Clone, Debug)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Debug)]
struct Envelope<T> {
    payload: T,
    criticality: Criticality,
}

/// A small deterministic supervisor core. Async adapters may wait/retry around
/// `CriticalBackpressure`; the core never silently discards a critical event.
#[derive(Debug)]
pub struct Supervisor<T> {
    capacity: usize,
    overflow: OverflowPolicy,
    queue: VecDeque<Envelope<T>>,
    tasks: HashMap<String, TaskStatus>,
    health: RuntimeHealth,
    cancelled: Arc<AtomicBool>,
    shutdown: bool,
}

impl<T> Supervisor<T> {
    #[must_use]
    pub fn new(capacity: usize, overflow: OverflowPolicy) -> Self {
        Self {
            capacity,
            overflow,
            queue: VecDeque::with_capacity(capacity),
            tasks: HashMap::new(),
            health: RuntimeHealth::default(),
            cancelled: Arc::new(AtomicBool::new(false)),
            shutdown: false,
        }
    }

    pub fn publish(&mut self, payload: T, criticality: Criticality) -> Result<(), ChannelError> {
        if self.capacity == 0 {
            return Err(ChannelError::ZeroCapacity);
        }
        if self.queue.len() < self.capacity {
            self.queue.push_back(Envelope {
                payload,
                criticality,
            });
            return Ok(());
        }

        if self.overflow == OverflowPolicy::DropOldestOptional {
            if let Some(index) = self
                .queue
                .iter()
                .position(|event| event.criticality == Criticality::Optional)
            {
                self.queue.remove(index);
                self.health.shed_optional_events += 1;
                self.queue.push_back(Envelope {
                    payload,
                    criticality,
                });
                return Ok(());
            }
        }

        match criticality {
            Criticality::Critical => Err(ChannelError::CriticalBackpressure),
            Criticality::Optional => {
                self.health.shed_optional_events += 1;
                Err(ChannelError::ShedOptional)
            }
        }
    }

    pub fn drain(&mut self) -> Vec<T> {
        self.queue.drain(..).map(|event| event.payload).collect()
    }

    pub fn register_task(
        &mut self,
        name: impl Into<String>,
        criticality: Criticality,
        restart_policy: RestartPolicy,
    ) {
        let name = name.into();
        self.tasks.insert(
            name.clone(),
            TaskStatus {
                name,
                criticality,
                running: true,
                restart_count: 0,
                circuit: CircuitState::Closed,
                restart_policy,
                last_failure: None,
            },
        );
    }

    pub fn record_failure(
        &mut self,
        name: &str,
        failure: TaskFailure,
    ) -> Result<(), SupervisorError> {
        let task = self
            .tasks
            .get_mut(name)
            .ok_or_else(|| SupervisorError::UnknownTask(name.to_owned()))?;
        task.last_failure = Some(failure);
        if task.restart_count < task.restart_policy.max_restarts {
            task.restart_count += 1;
            task.running = true;
            return Ok(());
        }
        task.running = false;
        task.circuit = CircuitState::Open;
        if task.criticality == Criticality::Critical {
            self.enter_safe_state(SafeStateReason::CriticalTaskFailed, "critical task failed");
        } else if self.health.state == HealthState::Healthy {
            self.health.state = HealthState::Degraded;
            self.health.last_issue = Some(format!("optional task {name} unavailable"));
        }
        Ok(())
    }

    #[must_use]
    pub fn task(&self, name: &str) -> Option<&TaskStatus> {
        self.tasks.get(name)
    }

    pub fn tasks(&self) -> impl Iterator<Item = &TaskStatus> {
        self.tasks.values()
    }

    #[must_use]
    pub fn health(&self) -> &RuntimeHealth {
        &self.health
    }

    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        CancellationToken(Arc::clone(&self.cancelled))
    }

    pub fn report_clock_discontinuity(&mut self, delta_millis: i64) {
        self.health.clock_discontinuities += 1;
        if self.health.state == HealthState::Healthy {
            self.health.state = HealthState::Degraded;
        }
        self.health.last_issue = Some(format!("clock discontinuity: {delta_millis}ms"));
    }

    pub fn report_low_resources(&mut self, detail: impl Into<String>) {
        self.health.low_resource_events += 1;
        self.enter_safe_state(SafeStateReason::LowResources, detail);
    }

    pub fn shutdown(&mut self) -> Result<(), SupervisorError> {
        if self.shutdown {
            return Err(SupervisorError::AlreadyShutdown);
        }
        self.shutdown = true;
        self.cancelled.store(true, Ordering::Release);
        for task in self.tasks.values_mut() {
            task.running = false;
        }
        self.enter_safe_state(
            SafeStateReason::CoordinatedShutdown,
            "coordinated shutdown complete",
        );
        Ok(())
    }

    fn enter_safe_state(&mut self, reason: SafeStateReason, detail: impl Into<String>) {
        self.health.state = HealthState::Safe;
        self.health.safe_state = Some(reason);
        self.health.last_issue = Some(detail.into());
    }
}
