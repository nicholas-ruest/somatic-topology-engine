//! Outside-in tests for the pinned, policy-gated live rvCSI process boundary.

use std::cell::RefCell;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use ste_radio_acquisition::live_adapter::{
    DetectedRadioEnvironment, LiveAdapterError, PinnedRadioManifest, ProcessRunner,
    QualificationDisposition, QualificationStatistics, RvcsiLiveAdapter,
};
use ste_radio_acquisition::{
    CalibrationMetadata, CaptureAuthorizationPort, CaptureHealth, CaptureLink, CaptureProfile,
    CaptureSession, HardwareProvenance,
};

struct Policy(bool);

impl CaptureAuthorizationPort for Policy {
    fn authorize_capture(&self, _: &CaptureSession) -> bool {
        self.0
    }
}

#[derive(Default)]
struct RecordingRunner {
    calls: RefCell<Vec<(PathBuf, Vec<OsString>)>>,
}

impl ProcessRunner for RecordingRunner {
    type Handle = u32;

    fn spawn(&self, program: &Path, args: &[OsString]) -> Result<Self::Handle, LiveAdapterError> {
        self.calls
            .borrow_mut()
            .push((program.to_owned(), args.to_vec()));
        Ok(42)
    }
}

fn session() -> CaptureSession {
    CaptureSession::start(
        CaptureProfile::new(5_180_000_000, 20_000_000, 1, 2).unwrap(),
        CaptureLink::new("wlan0", "link-a").unwrap(),
        HardwareProvenance::new("pi4", "bcm43455", "nexmon-1", "linux-6", "ap-a").unwrap(),
        CalibrationMetadata::new("cal-v1", "geometry-a").unwrap(),
        1,
    )
    .unwrap()
}

#[test]
fn policy_denial_prevents_process_launch() {
    let runner = RecordingRunner::default();
    let manifest = PinnedRadioManifest::development_pi4_fixture();
    let adapter = RvcsiLiveAdapter::new(&runner, manifest.clone(), manifest.detected_reference());

    assert_eq!(
        adapter.start(&session(), &Policy(false)),
        Err(LiveAdapterError::Unauthorized)
    );
    assert!(runner.calls.borrow().is_empty());
}

#[test]
fn every_compatibility_field_must_match_before_launch() {
    let runner = RecordingRunner::default();
    let manifest = PinnedRadioManifest::development_pi4_fixture();
    let mut detected = manifest.detected_reference();
    detected.firmware_digest = format!("sha256:{}", "c".repeat(64));
    let adapter = RvcsiLiveAdapter::new(&runner, manifest, detected);

    assert_eq!(
        adapter.start(&session(), &Policy(true)),
        Err(LiveAdapterError::IncompatibleEnvironment("firmware_digest"))
    );
    assert!(runner.calls.borrow().is_empty());
}

#[test]
fn launch_uses_fixed_executable_and_structured_arguments_without_a_shell() {
    let runner = RecordingRunner::default();
    let manifest = PinnedRadioManifest::development_pi4_fixture();
    let adapter = RvcsiLiveAdapter::new(&runner, manifest.clone(), manifest.detected_reference());

    assert_eq!(adapter.start(&session(), &Policy(true)).unwrap(), 42);
    let calls = runner.calls.borrow();
    let (program, args) = &calls[0];
    assert_eq!(program, Path::new("/usr/local/libexec/ste-rvcsi-capture"));
    assert!(
        !args
            .iter()
            .any(|value| value.to_string_lossy().contains(';'))
    );
    assert_eq!(args[0], "--interface");
    assert_eq!(args[1], "wlan0");
}

#[test]
fn manifest_rejects_interface_shell_metacharacters() {
    assert_eq!(
        PinnedRadioManifest::development_pi4_fixture().with_interface("wlan0; reboot"),
        Err(LiveAdapterError::InvalidManifest("interface"))
    );
}

#[test]
fn qualification_statistics_never_hide_rejection_missing_or_backpressure() {
    let statistics = QualificationStatistics::from_health(CaptureHealth {
        accepted: 900,
        rejected: 20,
        missing: 70,
        backpressured: 10,
    });
    assert_eq!(statistics.total_expected, 1_000);
    assert_eq!(statistics.accepted_percent_basis_points, 9_000);
    assert_eq!(statistics.disposition, QualificationDisposition::Degraded);
}

#[test]
fn detected_environment_is_an_explicit_complete_record() {
    let detected = DetectedRadioEnvironment {
        board_model: "pi4".into(),
        chipset: "bcm43455".into(),
        os_image_digest: format!("sha256:{}", "a".repeat(64)),
        kernel_release: "6.6.31+rpt-rpi-v8".into(),
        firmware_digest: format!("sha256:{}", "b".repeat(64)),
        nexmon_commit: "commit".into(),
        rvcsi_version: "1.0.0".into(),
        access_point: "qualified-ap".into(),
        band: "5ghz".into(),
        channel: 36,
        bandwidth_mhz: 20,
        packet_source: "ste-beacon".into(),
        geometry_id: "geometry-a".into(),
    };
    assert!(detected.validate().is_ok());
}
