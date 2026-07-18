//! Content-addressed evidence persistence and deterministic replay adapters.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::application::FeatureArtifactStore;
use crate::domain::{
    AlgorithmVersion, BaselineDrift, DspVersion, FeatureEvidenceArtifact, FrameEvidence,
    MotionEnergy, ObservationError, ObservationWindow, ObservationWindowId, PartitionRole,
    PresenceScore, WindowBounds, WindowPolicy,
};
use crate::dsp::{DspError, DspGraphSpec, PrimitiveCsiFrame, execute_dsp};

/// Generic collision-verifying immutable content-addressed store.
pub struct ContentAddressedStore<T> {
    entries: Mutex<BTreeMap<String, T>>,
}

impl<T> Default for ContentAddressedStore<T> {
    fn default() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
        }
    }
}

impl<T: Clone + PartialEq> ContentAddressedStore<T> {
    /// Stores idempotently. An occupied digest with different content is a hard
    /// collision and never overwrites the original value.
    pub fn put_verified(&self, digest: &str, value: &T) -> Result<PutOutcome, RepositoryError> {
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(RepositoryError::InvalidDigest);
        }
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| RepositoryError::Unavailable)?;
        match entries.get(digest) {
            Some(existing) if existing == value => Ok(PutOutcome::AlreadyPresent),
            Some(_) => Err(RepositoryError::DigestCollision),
            None => {
                entries.insert(digest.to_owned(), value.clone());
                Ok(PutOutcome::Inserted)
            }
        }
    }

    /// Returns an immutable clone by exact digest.
    pub fn get(&self, digest: &str) -> Result<Option<T>, RepositoryError> {
        self.entries
            .lock()
            .map(|entries| entries.get(digest).cloned())
            .map_err(|_| RepositoryError::Unavailable)
    }

    /// Number of distinct immutable artifacts.
    pub fn len(&self) -> Result<usize, RepositoryError> {
        self.entries
            .lock()
            .map(|entries| entries.len())
            .map_err(|_| RepositoryError::Unavailable)
    }

    /// Whether no artifacts have been stored.
    pub fn is_empty(&self) -> Result<bool, RepositoryError> {
        self.len().map(|length| length == 0)
    }
}

/// Evidence-specific repository enforcing the artifact's own digest.
#[derive(Default)]
pub struct ContentAddressedEvidenceRepository {
    inner: ContentAddressedStore<FeatureEvidenceArtifact>,
}

impl ContentAddressedEvidenceRepository {
    /// Stores and reports whether bytes were newly inserted or already present.
    pub fn put(&self, artifact: &FeatureEvidenceArtifact) -> Result<PutOutcome, RepositoryError> {
        self.inner.put_verified(artifact.digest(), artifact)
    }

    /// Retrieves an immutable artifact clone.
    pub fn get(&self, digest: &str) -> Result<Option<FeatureEvidenceArtifact>, RepositoryError> {
        self.inner.get(digest)
    }

    /// Number of distinct artifacts.
    pub fn len(&self) -> Result<usize, RepositoryError> {
        self.inner.len()
    }

    /// Whether no evidence exists.
    pub fn is_empty(&self) -> Result<bool, RepositoryError> {
        self.inner.is_empty()
    }
}

impl FeatureArtifactStore for ContentAddressedEvidenceRepository {
    fn put(&self, artifact: &FeatureEvidenceArtifact) -> Result<(), ObservationError> {
        Self::put(self, artifact)
            .map(|_| ())
            .map_err(|_| ObservationError::ArtifactFailure)
    }
}

/// Result of an immutable put.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PutOutcome {
    /// New digest/content inserted.
    Inserted,
    /// Exact digest/content already existed.
    AlreadyPresent,
}

/// Payload-free repository failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepositoryError {
    /// Digest is not canonical lowercase/uppercase SHA-256 hex shape.
    InvalidDigest,
    /// Existing content differs at the requested digest.
    DigestCollision,
    /// Repository synchronization failed.
    Unavailable,
}

/// Acquisition-independent radio evidence input for observation replay.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayEvidenceFrame {
    /// Immutable radio frame reference.
    pub source_ref: String,
    /// Primitive CSI values only; no upstream implementation type crosses.
    pub frame: PrimitiveCsiFrame,
}

/// Deterministic radio-to-observation anti-corruption adapter.
pub struct ObservationReplay;

impl ObservationReplay {
    /// Executes one pinned DSP graph and closes an immutable observation artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn replay(
        window_id: ObservationWindowId,
        bounds: WindowBounds,
        policy: WindowPolicy,
        algorithm_version: AlgorithmVersion,
        dsp_version: DspVersion,
        calibration_version: String,
        partition: PartitionRole,
        spec: DspGraphSpec,
        frames: &[ReplayEvidenceFrame],
    ) -> Result<FeatureEvidenceArtifact, ReplayRepositoryError> {
        if frames.iter().any(|frame| {
            frame.source_ref.trim().is_empty() || frame.source_ref != frame.frame.source_ref
        }) {
            return Err(ReplayRepositoryError::InvalidEvidence);
        }
        let primitives = frames
            .iter()
            .map(|frame| frame.frame.clone())
            .collect::<Vec<_>>();
        let observation = execute_dsp(spec, &primitives)?;
        let mut window = ObservationWindow::open(
            window_id,
            bounds,
            policy,
            algorithm_version,
            dsp_version,
            calibration_version,
        );
        for (index, source) in frames.iter().enumerate() {
            window.append(FrameEvidence {
                source_ref: source.source_ref.clone(),
                event_time_ns: source.frame.event_time_ns,
                motion_energy: MotionEnergy::new(observation.motion_energy)?,
                presence_score: PresenceScore::new(observation.presence_score)?,
                periodicity: None,
                baseline_drift: BaselineDrift::new(observation.drift_per_second.abs())?,
                missing_before: if index + 1 == frames.len() {
                    observation.missing_frames
                } else {
                    0
                },
                interference: observation.interference_ratio > 0.5
                    || observation.saturation_fraction > 0.0,
            })?;
        }
        window.close(partition).map_err(Into::into)
    }
}

/// Replay adapter failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayRepositoryError {
    /// DSP graph/input validation failed.
    Dsp(DspError),
    /// Observation aggregate rejected derived evidence.
    Observation(ObservationError),
    /// Source provenance was missing.
    InvalidEvidence,
}

impl From<DspError> for ReplayRepositoryError {
    fn from(value: DspError) -> Self {
        Self::Dsp(value)
    }
}

impl From<ObservationError> for ReplayRepositoryError {
    fn from(value: ObservationError) -> Self {
        Self::Observation(value)
    }
}
