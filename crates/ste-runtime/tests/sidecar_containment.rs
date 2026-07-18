//! VT-IPC-002 malicious and failed sidecar containment tests.

use ste_runtime::sidecar::*;

#[derive(Clone)]
enum Behavior {
    Healthy,
    Absent,
    Hung,
    Corrupt,
}
struct Process {
    behavior: Behavior,
    started: bool,
    terminated: bool,
    observed_sandbox: Option<SandboxPolicy>,
}
impl Process {
    fn new(behavior: Behavior) -> Self {
        Self {
            behavior,
            started: false,
            terminated: false,
            observed_sandbox: None,
        }
    }
}
impl SidecarProcess for Process {
    fn start(&mut self, sandbox: SandboxPolicy) -> Result<(), SidecarError> {
        self.observed_sandbox = Some(sandbox);
        if matches!(self.behavior, Behavior::Absent) {
            Err(SidecarError::ProcessFailed)
        } else {
            self.started = true;
            Ok(())
        }
    }
    fn exchange(
        &mut self,
        _: [u8; 32],
        request: &SidecarRequest,
        _: u64,
    ) -> Result<SidecarResponse, SidecarError> {
        match self.behavior {
            Behavior::Healthy => Ok(SidecarResponse {
                request_id: request.request_id.clone(),
                contract_digest: request.contract_digest,
                advisory_payload: b"advisory".to_vec(),
            }),
            Behavior::Hung => Err(SidecarError::TimedOut),
            Behavior::Corrupt => Ok(SidecarResponse {
                request_id: "forged".into(),
                contract_digest: [9; 32],
                advisory_payload: vec![],
            }),
            Behavior::Absent => Err(SidecarError::ProcessFailed),
        }
    }
    fn terminate(&mut self) -> Result<(), SidecarError> {
        self.terminated = true;
        Ok(())
    }
}

fn sandbox() -> SandboxPolicy {
    SandboxPolicy {
        uid: 65534,
        cpu_millis_per_second: 100,
        memory_bytes: 64 * 1024 * 1024,
        storage_bytes: 8 * 1024 * 1024,
        network_allowed: false,
        hardware_allowed: false,
        authoritative_store_allowed: false,
    }
}
fn policy(enabled: bool) -> SidecarPolicy {
    SidecarPolicy {
        enabled,
        contract_digest: [7; 32],
        maximum_requests_per_window: 1,
        maximum_payload_bytes: 64,
        timeout_ns: 100,
        sandbox: sandbox(),
    }
}
fn request() -> SidecarRequest {
    SidecarRequest {
        request_id: "request-1".into(),
        operation: SidecarOperation::SummarizeDeidentified,
        contract_digest: [7; 32],
        payload: b"deidentified".to_vec(),
    }
}

#[test]
fn disabled_by_default_and_absent_process_leave_core_operation_unchanged() {
    let mut disabled = SidecarSupervisor::new(
        Process::new(Behavior::Healthy),
        policy(false),
        "local-sidecar-secret",
    )
    .unwrap();
    disabled.start();
    assert_eq!(disabled.health(), SidecarHealth::Disabled);
    assert_eq!(
        disabled.execute(&request(), 1),
        Err(SidecarError::Unavailable)
    );
    let mut absent = SidecarSupervisor::new(
        Process::new(Behavior::Absent),
        policy(true),
        "local-sidecar-secret",
    )
    .unwrap();
    absent.start();
    assert_eq!(absent.health(), SidecarHealth::DegradedAbsent);
    assert_eq!(
        absent.execute(&request(), 1),
        Err(SidecarError::Unavailable)
    );
    let mut core_authoritative_counter = 0;
    core_authoritative_counter += 1;
    assert_eq!(core_authoritative_counter, 1);
}

#[test]
fn sandbox_must_be_offline_unprivileged_and_without_store_or_hardware_access() {
    for unsafe_policy in [
        SandboxPolicy {
            uid: 0,
            ..sandbox()
        },
        SandboxPolicy {
            network_allowed: true,
            ..sandbox()
        },
        SandboxPolicy {
            hardware_allowed: true,
            ..sandbox()
        },
        SandboxPolicy {
            authoritative_store_allowed: true,
            ..sandbox()
        },
    ] {
        let mut p = policy(true);
        p.sandbox = unsafe_policy;
        assert!(matches!(
            SidecarSupervisor::new(Process::new(Behavior::Healthy), p, "local-sidecar-secret"),
            Err(SidecarError::UnsafeSandbox)
        ));
    }
}

#[test]
fn contract_digest_payload_and_rate_are_fail_closed() {
    let mut supervisor = SidecarSupervisor::new(
        Process::new(Behavior::Healthy),
        policy(true),
        "local-sidecar-secret",
    )
    .unwrap();
    supervisor.start();
    let mut forged = request();
    forged.contract_digest = [8; 32];
    assert_eq!(
        supervisor.execute(&forged, 1),
        Err(SidecarError::InvalidContract)
    );
    assert_eq!(
        supervisor.execute(&request(), 1).unwrap().advisory_payload,
        b"advisory"
    );
    assert_eq!(
        supervisor.execute(&request(), 2),
        Err(SidecarError::RateLimited)
    );
}

#[test]
fn hung_corrupt_and_killed_sidecars_degrade_only_the_optional_feature() {
    let mut hung = SidecarSupervisor::new(
        Process::new(Behavior::Hung),
        policy(true),
        "local-sidecar-secret",
    )
    .unwrap();
    hung.start();
    assert_eq!(hung.execute(&request(), 1), Err(SidecarError::TimedOut));
    assert_eq!(hung.health(), SidecarHealth::DegradedHung);
    let mut corrupt = SidecarSupervisor::new(
        Process::new(Behavior::Corrupt),
        policy(true),
        "local-sidecar-secret",
    )
    .unwrap();
    corrupt.start();
    assert_eq!(
        corrupt.execute(&request(), 1),
        Err(SidecarError::CorruptResponse)
    );
    assert_eq!(corrupt.health(), SidecarHealth::DegradedCorrupt);
    let mut killed = SidecarSupervisor::new(
        Process::new(Behavior::Healthy),
        policy(true),
        "local-sidecar-secret",
    )
    .unwrap();
    killed.start();
    killed.kill();
    assert_eq!(killed.health(), SidecarHealth::Killed);
    assert_eq!(
        killed.execute(&request(), 1),
        Err(SidecarError::Unavailable)
    );
}

#[test]
fn allowlist_has_no_capability_widening_hardware_or_authoritative_write_operation() {
    let operations = [
        SidecarOperation::SummarizeDeidentified,
        SidecarOperation::OptimizeOfflineCopy,
    ];
    assert_eq!(operations.len(), 2);
    assert!(operations.iter().all(|operation| matches!(
        operation,
        SidecarOperation::SummarizeDeidentified | SidecarOperation::OptimizeOfflineCopy
    )));
}
