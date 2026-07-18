//! Deterministic, accessible, policy-approved device interaction domain.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, error::Error, fmt};

/// Compatibility marker for bounded-context tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainBoundary;
fn required(value: impl Into<String>, label: &'static str) -> Result<String, DomainError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 256 {
        Err(DomainError::InvalidValue(label))
    } else {
        Ok(value)
    }
}

/// Supported peripheral identities.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum PeripheralId {
    /// OLED display.
    Display,
    /// RGB LED.
    Led,
    /// Touch input.
    Touch,
    /// Environmental covariate sensor.
    Environment,
}
/// Explicit health state; failures are never silently frozen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeripheralHealth {
    /// Peripheral responds normally.
    Healthy,
    /// Peripheral is degraded but isolated.
    Failed {
        /// Payload-minimized failure reason.
        reason: String,
    },
}
/// Signal quality band safe for RGB rendering.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QualityIndicator {
    /// Poor quality.
    Poor,
    /// Degraded quality.
    Degraded,
    /// Good quality.
    Good,
}
/// Arousal explicitly labeled by the participant, never inferred valence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ArousalBand {
    /// User labeled low.
    Low,
    /// User labeled medium.
    Medium,
    /// User labeled high.
    High,
}
/// Approved task-specific workload display band.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkloadBand {
    /// Lower.
    Lower,
    /// Moderate.
    Moderate,
    /// Elevated.
    Elevated,
}
/// Fixed accessible RGB palette; text always accompanies color.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RgbColor {
    /// Output disabled.
    Off,
    /// Blue.
    Blue,
    /// Amber.
    Amber,
    /// White.
    White,
    /// Cyan.
    Cyan,
    /// Magenta.
    Magenta,
    /// Green.
    Green,
    /// Red.
    Red,
}
/// Enumerated policy-approved projection vocabulary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Projection {
    /// Capture is unauthorized.
    Unauthorized,
    /// Calibration in progress.
    Calibrating,
    /// Evidence contaminated.
    Contaminated,
    /// Evidence insufficient.
    InsufficientEvidence,
    /// Evidence exceeded maximum age.
    Stale,
    /// Named peripheral fault.
    Fault(PeripheralId),
    /// Signal quality, not physiology/affect.
    SignalQuality(QualityIndicator),
    /// Participant's explicit label only.
    UserLabeledArousal(ArousalBand),
    /// Validated task-specific claim.
    TaskWorkload(WorkloadBand),
    /// Physical anchor accepted.
    AnchorConfirmed,
}
/// Rendered text-plus-color state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedProjection {
    /// Accessible text label.
    pub text: &'static str,
    /// Redundant color cue.
    pub color: RgbColor,
}
impl Projection {
    /// Maps only enumerated states to reviewed copy and palette.
    #[must_use]
    pub const fn render(&self) -> RenderedProjection {
        match self {
            Self::Unauthorized => RenderedProjection {
                text: "Sensing unauthorized",
                color: RgbColor::Red,
            },
            Self::Calibrating => RenderedProjection {
                text: "Calibrating",
                color: RgbColor::Blue,
            },
            Self::Contaminated => RenderedProjection {
                text: "Signal contaminated",
                color: RgbColor::Magenta,
            },
            Self::InsufficientEvidence => RenderedProjection {
                text: "Insufficient evidence",
                color: RgbColor::Amber,
            },
            Self::Stale => RenderedProjection {
                text: "Evidence stale",
                color: RgbColor::Amber,
            },
            Self::Fault(_) => RenderedProjection {
                text: "Peripheral fault",
                color: RgbColor::Red,
            },
            Self::SignalQuality(QualityIndicator::Poor) => RenderedProjection {
                text: "Signal quality poor",
                color: RgbColor::Red,
            },
            Self::SignalQuality(QualityIndicator::Degraded) => RenderedProjection {
                text: "Signal quality degraded",
                color: RgbColor::Amber,
            },
            Self::SignalQuality(QualityIndicator::Good) => RenderedProjection {
                text: "Signal quality good",
                color: RgbColor::Cyan,
            },
            Self::UserLabeledArousal(ArousalBand::Low) => RenderedProjection {
                text: "Your label: low arousal",
                color: RgbColor::Blue,
            },
            Self::UserLabeledArousal(ArousalBand::Medium) => RenderedProjection {
                text: "Your label: medium arousal",
                color: RgbColor::White,
            },
            Self::UserLabeledArousal(ArousalBand::High) => RenderedProjection {
                text: "Your label: high arousal",
                color: RgbColor::Magenta,
            },
            Self::TaskWorkload(WorkloadBand::Lower) => RenderedProjection {
                text: "Task workload: lower",
                color: RgbColor::Blue,
            },
            Self::TaskWorkload(WorkloadBand::Moderate) => RenderedProjection {
                text: "Task workload: moderate",
                color: RgbColor::White,
            },
            Self::TaskWorkload(WorkloadBand::Elevated) => RenderedProjection {
                text: "Task workload: elevated",
                color: RgbColor::Magenta,
            },
            Self::AnchorConfirmed => RenderedProjection {
                text: "Anchor confirmed",
                color: RgbColor::Green,
            },
        }
    }
}

/// Independent display cadence and evidence staleness policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RefreshPolicy {
    cadence_ms: u64,
    maximum_evidence_age_ms: u64,
}
impl RefreshPolicy {
    /// Creates positive policy with age not shorter than cadence.
    pub fn new(cadence_ms: u64, maximum_age_ms: u64) -> Result<Self, DomainError> {
        if cadence_ms == 0 || maximum_age_ms < cadence_ms {
            Err(DomainError::InvalidRefreshPolicy)
        } else {
            Ok(Self {
                cadence_ms,
                maximum_evidence_age_ms: maximum_age_ms,
            })
        }
    }
}
/// Debounced physical touch evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TouchGesture {
    occurred_at: u64,
    duration_ms: u64,
}
impl TouchGesture {
    /// Creates a timestamped physical gesture.
    pub fn new(occurred_at: u64, duration_ms: u64) -> Result<Self, DomainError> {
        if duration_ms == 0 {
            Err(DomainError::InvalidValue("gesture duration"))
        } else {
            Ok(Self {
                occurred_at,
                duration_ms,
            })
        }
    }
}
/// Authorized anchor request emitted from a physical gesture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchorRequest {
    /// Session identity.
    pub session_id: String,
    /// Gesture timestamp.
    pub requested_at: u64,
}

/// Interaction aggregate supervising projections and isolated peripherals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionSession {
    id: String,
    projection: Projection,
    policy: RefreshPolicy,
    evidence_at: Option<u64>,
    last_touch_at: Option<u64>,
    health: BTreeMap<PeripheralId, PeripheralHealth>,
    events: Vec<InteractionEvent>,
    active: bool,
}
impl InteractionSession {
    /// Starts in explicit insufficient-evidence state.
    pub fn start(id: impl Into<String>, policy: RefreshPolicy) -> Result<Self, DomainError> {
        let id = required(id, "interaction session identifier")?;
        Ok(Self {
            events: vec![InteractionEvent::Started {
                session_id: id.clone(),
            }],
            id,
            projection: Projection::InsufficientEvidence,
            policy,
            evidence_at: None,
            last_touch_at: None,
            health: [
                PeripheralId::Display,
                PeripheralId::Led,
                PeripheralId::Touch,
                PeripheralId::Environment,
            ]
            .into_iter()
            .map(|p| (p, PeripheralHealth::Healthy))
            .collect(),
            active: true,
        })
    }
    /// Renders a policy-approved projection and records its evidence time.
    pub fn render(
        &mut self,
        projection: Projection,
        evidence_at: u64,
        now: u64,
    ) -> Result<(), DomainError> {
        self.ensure_active()?;
        if evidence_at > now {
            return Err(DomainError::FutureEvidence);
        }
        self.projection = if now - evidence_at > self.policy.maximum_evidence_age_ms {
            Projection::Stale
        } else {
            projection
        };
        self.evidence_at = Some(evidence_at);
        self.events.push(InteractionEvent::ProjectionRendered {
            projection: self.projection.clone(),
            rendered_at: now,
        });
        Ok(())
    }
    /// Applies staleness independently of the underlying evidence horizon.
    pub fn refresh(&mut self, now: u64) -> Result<(), DomainError> {
        self.ensure_active()?;
        if self
            .evidence_at
            .is_none_or(|at| now.saturating_sub(at) > self.policy.maximum_evidence_age_ms)
        {
            self.projection = Projection::Stale;
            self.events.push(InteractionEvent::ProjectionRendered {
                projection: Projection::Stale,
                rendered_at: now,
            });
        }
        Ok(())
    }
    /// Validates physical debounce and authorization before requesting an anchor.
    pub fn handle_touch(
        &mut self,
        gesture: TouchGesture,
        authorized: bool,
    ) -> Result<AnchorRequest, DomainError> {
        self.ensure_active()?;
        if !authorized {
            return Err(DomainError::UnauthorizedAnchor);
        }
        if gesture.duration_ms < 50
            || self
                .last_touch_at
                .is_some_and(|last| gesture.occurred_at.saturating_sub(last) < 200)
        {
            return Err(DomainError::GestureNotDebounced);
        }
        self.last_touch_at = Some(gesture.occurred_at);
        self.projection = Projection::AnchorConfirmed;
        let request = AnchorRequest {
            session_id: self.id.clone(),
            requested_at: gesture.occurred_at,
        };
        self.events
            .push(InteractionEvent::AnchorRequested(request.clone()));
        Ok(request)
    }
    /// Isolates a peripheral failure while keeping core session supervision active.
    pub fn record_peripheral_failure(
        &mut self,
        peripheral: PeripheralId,
        reason: impl Into<String>,
    ) -> Result<(), DomainError> {
        self.ensure_active()?;
        let reason = required(reason, "peripheral failure")?;
        self.health.insert(
            peripheral,
            PeripheralHealth::Failed {
                reason: reason.clone(),
            },
        );
        self.projection = Projection::Fault(peripheral);
        self.events
            .push(InteractionEvent::PeripheralFailed { peripheral, reason });
        Ok(())
    }
    /// Stops interaction without affecting capture supervision owned elsewhere.
    pub fn stop(&mut self) -> Result<(), DomainError> {
        self.ensure_active()?;
        self.active = false;
        self.events.push(InteractionEvent::Stopped);
        Ok(())
    }
    fn ensure_active(&self) -> Result<(), DomainError> {
        if self.active {
            Ok(())
        } else {
            Err(DomainError::SessionStopped)
        }
    }
    /// Stable identity.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Current projection.
    #[must_use]
    pub const fn projection(&self) -> &Projection {
        &self.projection
    }
    /// Session liveness.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }
    /// Immutable event history.
    #[must_use]
    pub fn events(&self) -> &[InteractionEvent] {
        &self.events
    }
}

/// Aggregate events for audit and recovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteractionEvent {
    /// Session began.
    Started {
        /// Session identifier.
        session_id: String,
    },
    /// Projection rendered.
    ProjectionRendered {
        /// Approved projection.
        projection: Projection,
        /// Render time.
        rendered_at: u64,
    },
    /// Anchor requested.
    AnchorRequested(AnchorRequest),
    /// Peripheral failed.
    PeripheralFailed {
        /// Peripheral identity.
        peripheral: PeripheralId,
        /// Payload-minimized reason.
        reason: String,
    },
    /// Session stopped.
    Stopped,
}
/// Stable invariant failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    /// Required value invalid.
    InvalidValue(&'static str),
    /// Refresh policy invalid.
    InvalidRefreshPolicy,
    /// Evidence timestamp lies in future.
    FutureEvidence,
    /// Touch lacked current authorization.
    UnauthorizedAnchor,
    /// Gesture failed duration/refractory debounce.
    GestureNotDebounced,
    /// Session is terminally stopped.
    SessionStopped,
}
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "device interaction invariant failed: {self:?}")
    }
}
impl Error for DomainError {}
