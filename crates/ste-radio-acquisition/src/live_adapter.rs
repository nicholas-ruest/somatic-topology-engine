//! Pinned, policy-gated process boundary for a privileged rvCSI helper.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::{CaptureAuthorizationPort, CaptureHealth, CaptureSession};

/// Complete environment observed by commissioning before live capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetectedRadioEnvironment {
    /// Board model reported by firmware/device tree.
    pub board_model: String,
    /// Wi-Fi chipset identifier.
    pub chipset: String,
    /// Immutable OS image digest.
    pub os_image_digest: String,
    /// Running kernel release.
    pub kernel_release: String,
    /// Patched Wi-Fi firmware digest.
    pub firmware_digest: String,
    /// Reviewed Nexmon source revision.
    pub nexmon_commit: String,
    /// rvCSI wire/tool version.
    pub rvcsi_version: String,
    /// Qualified access-point identifier.
    pub access_point: String,
    /// Radio band.
    pub band: String,
    /// Fixed channel number.
    pub channel: u16,
    /// Fixed channel width.
    pub bandwidth_mhz: u16,
    /// Qualified packet stimulus/source.
    pub packet_source: String,
    /// Physical radio geometry profile.
    pub geometry_id: String,
}

impl DetectedRadioEnvironment {
    /// Rejects incomplete probe records before compatibility evaluation.
    pub fn validate(&self) -> Result<(), LiveAdapterError> {
        for (field, value) in [
            ("board_model", self.board_model.as_str()),
            ("chipset", self.chipset.as_str()),
            ("os_image_digest", self.os_image_digest.as_str()),
            ("kernel_release", self.kernel_release.as_str()),
            ("firmware_digest", self.firmware_digest.as_str()),
            ("nexmon_commit", self.nexmon_commit.as_str()),
            ("rvcsi_version", self.rvcsi_version.as_str()),
            ("access_point", self.access_point.as_str()),
            ("band", self.band.as_str()),
            ("packet_source", self.packet_source.as_str()),
            ("geometry_id", self.geometry_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(LiveAdapterError::InvalidManifest(field));
            }
        }
        if self.channel == 0 || self.bandwidth_mhz == 0 {
            return Err(LiveAdapterError::InvalidManifest("radio_channel"));
        }
        for (field, digest) in [
            ("os_image_digest", self.os_image_digest.as_str()),
            ("firmware_digest", self.firmware_digest.as_str()),
        ] {
            let Some(hex) = digest.strip_prefix("sha256:") else {
                return Err(LiveAdapterError::InvalidManifest(field));
            };
            if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(LiveAdapterError::InvalidManifest(field));
            }
        }
        Ok(())
    }
}

/// Exact compatibility allowlist for one qualified installation profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinnedRadioManifest {
    expected: DetectedRadioEnvironment,
    interface: String,
    helper: PathBuf,
}

impl PinnedRadioManifest {
    /// Creates a production manifest from an externally verified/signed
    /// qualification record. Signature verification remains a composition
    /// concern; this constructor enforces structural completeness.
    pub fn from_qualified(
        expected: DetectedRadioEnvironment,
        interface: &str,
    ) -> Result<Self, LiveAdapterError> {
        expected.validate()?;
        Self {
            expected,
            interface: "wlan0".into(),
            helper: PathBuf::from("/usr/local/libexec/ste-rvcsi-capture"),
        }
        .with_interface(interface)
    }

    /// Deterministic Pi 4 development fixture for adapter tests. Its placeholder
    /// digests are never a production compatibility or hardware-acceptance record.
    #[must_use]
    pub fn development_pi4_fixture() -> Self {
        Self {
            expected: DetectedRadioEnvironment {
                board_model: "pi4".into(),
                chipset: "bcm43455".into(),
                os_image_digest: format!("sha256:{}", "a".repeat(64)),
                kernel_release: "6.6.31+rpt-rpi-v8".into(),
                firmware_digest: format!("sha256:{}", "b".repeat(64)),
                nexmon_commit: "ste-reviewed-nexmon-v1".into(),
                rvcsi_version: "1.0.0".into(),
                access_point: "qualified-ap".into(),
                band: "5ghz".into(),
                channel: 36,
                bandwidth_mhz: 20,
                packet_source: "ste-beacon".into(),
                geometry_id: "geometry-a".into(),
            },
            interface: "wlan0".into(),
            helper: PathBuf::from("/usr/local/libexec/ste-rvcsi-capture"),
        }
    }

    /// Returns the full expected record for comparison/probe fixtures.
    #[must_use]
    pub fn detected_reference(&self) -> DetectedRadioEnvironment {
        self.expected.clone()
    }

    /// Changes the interface only when it is a plain kernel interface token.
    pub fn with_interface(mut self, interface: &str) -> Result<Self, LiveAdapterError> {
        if interface.is_empty()
            || interface.len() > 15
            || !interface
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        {
            return Err(LiveAdapterError::InvalidManifest("interface"));
        }
        self.interface = interface.into();
        Ok(self)
    }

    fn verify(&self, detected: &DetectedRadioEnvironment) -> Result<(), LiveAdapterError> {
        detected.validate()?;
        macro_rules! exact {
            ($field:ident) => {
                if self.expected.$field != detected.$field {
                    return Err(LiveAdapterError::IncompatibleEnvironment(stringify!(
                        $field
                    )));
                }
            };
        }
        exact!(board_model);
        exact!(chipset);
        exact!(os_image_digest);
        exact!(kernel_release);
        exact!(firmware_digest);
        exact!(nexmon_commit);
        exact!(rvcsi_version);
        exact!(access_point);
        exact!(band);
        exact!(channel);
        exact!(bandwidth_mhz);
        exact!(packet_source);
        exact!(geometry_id);
        Ok(())
    }
}

/// Injection-safe process spawning boundary.
pub trait ProcessRunner {
    /// Owned live process handle.
    type Handle;
    /// Spawns a fixed program with distinct arguments, never a shell command.
    fn spawn(&self, program: &Path, args: &[OsString]) -> Result<Self::Handle, LiveAdapterError>;
}

/// Production runner; it invokes no shell and never concatenates arguments.
pub struct StdProcessRunner;

impl ProcessRunner for StdProcessRunner {
    type Handle = Child;

    fn spawn(&self, program: &Path, args: &[OsString]) -> Result<Self::Handle, LiveAdapterError> {
        Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| LiveAdapterError::ProcessLaunch)
    }
}

/// Live process adapter parameterized over an auditable runner.
pub struct RvcsiLiveAdapter<'a, R> {
    runner: &'a R,
    manifest: PinnedRadioManifest,
    detected: DetectedRadioEnvironment,
}

impl<'a, R: ProcessRunner> RvcsiLiveAdapter<'a, R> {
    /// Creates an adapter from pinned expectations and freshly probed facts.
    #[must_use]
    pub const fn new(
        runner: &'a R,
        manifest: PinnedRadioManifest,
        detected: DetectedRadioEnvironment,
    ) -> Self {
        Self {
            runner,
            manifest,
            detected,
        }
    }

    /// Verifies current policy and every compatibility field immediately before launch.
    pub fn start(
        &self,
        session: &CaptureSession,
        policy: &dyn CaptureAuthorizationPort,
    ) -> Result<R::Handle, LiveAdapterError> {
        if !policy.authorize_capture(session) {
            return Err(LiveAdapterError::Unauthorized);
        }
        self.manifest.verify(&self.detected)?;
        let args = vec![
            "--interface".into(),
            self.manifest.interface.clone().into(),
            "--channel".into(),
            self.manifest.expected.channel.to_string().into(),
            "--bandwidth-mhz".into(),
            self.manifest.expected.bandwidth_mhz.to_string().into(),
            "--output-format".into(),
            "rvcsi-v1".into(),
        ];
        self.runner.spawn(&self.manifest.helper, &args)
    }
}

/// Qualification disposition preserving degraded and rejected operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QualificationDisposition {
    /// Meets the initial acquisition threshold.
    Accepted,
    /// Usable only as explicitly degraded evidence.
    Degraded,
    /// Does not qualify for capture publication.
    Rejected,
}

/// Explicit acquisition counts and integer acceptance ratio.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QualificationStatistics {
    /// Valid accepted frames.
    pub accepted: u64,
    /// Structurally or physically rejected frames.
    pub rejected: u64,
    /// Sequence-derived missing frames.
    pub missing: u64,
    /// Frames explicitly affected by backpressure.
    pub backpressured: u64,
    /// Sum of all explicit dispositions.
    pub total_expected: u64,
    /// Accepted fraction where 10,000 equals 100 percent.
    pub accepted_percent_basis_points: u64,
    /// Threshold-derived qualification disposition.
    pub disposition: QualificationDisposition,
}

impl QualificationStatistics {
    /// Converts acquisition health without dropping any failure category.
    #[must_use]
    pub fn from_health(health: CaptureHealth) -> Self {
        let total_expected = health
            .accepted
            .saturating_add(health.rejected)
            .saturating_add(health.missing)
            .saturating_add(health.backpressured);
        let basis_points = if total_expected == 0 {
            0
        } else {
            health.accepted.saturating_mul(10_000) / total_expected
        };
        let disposition = if basis_points >= 9_500 && health.backpressured == 0 {
            QualificationDisposition::Accepted
        } else if basis_points >= 8_000 {
            QualificationDisposition::Degraded
        } else {
            QualificationDisposition::Rejected
        };
        Self {
            accepted: health.accepted,
            rejected: health.rejected,
            missing: health.missing,
            backpressured: health.backpressured,
            total_expected,
            accepted_percent_basis_points: basis_points,
            disposition,
        }
    }
}

/// Stable, payload-free live adapter failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveAdapterError {
    /// Current capture policy denied launch.
    Unauthorized,
    /// Required manifest field is invalid or incomplete.
    InvalidManifest(&'static str),
    /// Probed field differs from the pinned allowlist.
    IncompatibleEnvironment(&'static str),
    /// Fixed helper process could not launch.
    ProcessLaunch,
}
