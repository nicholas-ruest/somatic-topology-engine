//! Safe simulator-first hardware contracts and versioned CrowPi profiles.

use std::collections::{BTreeMap, VecDeque};

/// Versioned, policy-approved output; adapters never receive raw model scores.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HardwareProjection {
    /// Approved text label.
    pub label: String,
    /// Accessibility-safe RGB color.
    pub rgb: (u8, u8, u8),
    /// Brightness in percent.
    pub brightness_percent: u8,
    /// Whether the visible sensing indicator must be illuminated.
    pub sensing_visible: bool,
}

/// Timestamped environmental covariate with no confidence field or modifier API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentalCovariate {
    /// Event time.
    pub observed_at_ns: u64,
    /// Ambient temperature.
    pub temperature_celsius: f32,
    /// Relative humidity percentage.
    pub relative_humidity_percent: f32,
}

/// Stable touch event emitted only after debounce.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TouchEvent {
    /// Stable transition to pressed.
    Pressed,
    /// Stable transition to released.
    Released,
}

/// Display output contract.
pub trait DisplayPort {
    /// Renders one approved projection.
    fn render(&mut self, projection: &HardwareProjection) -> Result<(), HardwareError>;
}
/// RGB output contract.
pub trait RgbPort {
    /// Sets color and bounded brightness.
    fn set_rgb(&mut self, rgb: (u8, u8, u8), brightness_percent: u8) -> Result<(), HardwareError>;
}
/// Visible sensing indicator contract.
pub trait SensingIndicatorPort {
    /// Sets the visible sensing indicator.
    fn set_visible(&mut self, visible: bool) -> Result<(), HardwareError>;
}
/// Debounced touch input contract.
pub trait TouchPort {
    /// Polls for a debounced edge at monotonic time.
    fn poll_touch(&mut self, now_ns: u64) -> Result<Option<TouchEvent>, HardwareError>;
}
/// Environmental sensor contract.
pub trait EnvironmentPort {
    /// Reads a timestamped display-only environmental covariate.
    fn read_environment(&mut self, now_ns: u64) -> Result<EnvironmentalCovariate, HardwareError>;
}
/// Physical RF-off input contract.
pub trait PhysicalOffPort {
    /// Reads the independent physical RF-off control.
    fn is_physical_off(&mut self) -> Result<bool, HardwareError>;
}

/// Deterministic simulator satisfying every hardware contract.
#[derive(Clone, Debug, Default)]
pub struct HardwareSimulator {
    /// Last successfully rendered projection.
    pub display: Option<HardwareProjection>,
    /// Last RGB output.
    pub rgb: Option<((u8, u8, u8), u8)>,
    /// Last sensing indicator state.
    pub sensing_visible: bool,
    /// Current physical-off state.
    pub physical_off: bool,
    /// Fixed environment reading.
    pub environment: Option<(f32, f32)>,
    raw_touches: VecDeque<(u64, bool)>,
    faults: BTreeMap<&'static str, HardwareError>,
    touch: TouchDebouncer,
}

impl HardwareSimulator {
    /// Creates a simulator with an explicit touch debounce duration.
    #[must_use]
    pub fn with_touch_debounce(debounce_ns: u64) -> Self {
        Self {
            touch: TouchDebouncer::new(debounce_ns),
            ..Self::default()
        }
    }
    /// Queues a raw touch transition at an event time.
    pub fn queue_touch(&mut self, at_ns: u64, pressed: bool) {
        self.raw_touches.push_back((at_ns, pressed));
    }
    /// Injects a named peripheral fault without affecting other ports.
    pub fn inject_fault(&mut self, peripheral: &'static str, error: HardwareError) {
        self.faults.insert(peripheral, error);
    }
    fn fault(&self, peripheral: &'static str) -> Result<(), HardwareError> {
        self.faults.get(peripheral).copied().map_or(Ok(()), Err)
    }
}

impl DisplayPort for HardwareSimulator {
    fn render(&mut self, projection: &HardwareProjection) -> Result<(), HardwareError> {
        self.fault("display")?;
        if projection.label.trim().is_empty() || projection.brightness_percent > 100 {
            return Err(HardwareError::InvalidValue);
        }
        self.display = Some(projection.clone());
        Ok(())
    }
}
impl RgbPort for HardwareSimulator {
    fn set_rgb(&mut self, rgb: (u8, u8, u8), brightness: u8) -> Result<(), HardwareError> {
        self.fault("rgb")?;
        if brightness > 100 {
            return Err(HardwareError::InvalidValue);
        }
        self.rgb = Some((rgb, brightness));
        Ok(())
    }
}
impl SensingIndicatorPort for HardwareSimulator {
    fn set_visible(&mut self, visible: bool) -> Result<(), HardwareError> {
        self.fault("indicator")?;
        self.sensing_visible = visible;
        Ok(())
    }
}
impl TouchPort for HardwareSimulator {
    fn poll_touch(&mut self, now_ns: u64) -> Result<Option<TouchEvent>, HardwareError> {
        self.fault("touch")?;
        while self
            .raw_touches
            .front()
            .is_some_and(|(at, _)| *at <= now_ns)
        {
            let (at, pressed) = self.raw_touches.pop_front().expect("front checked");
            self.touch.observe(at, pressed)?;
        }
        self.touch.poll(now_ns)
    }
}
impl EnvironmentPort for HardwareSimulator {
    fn read_environment(&mut self, now_ns: u64) -> Result<EnvironmentalCovariate, HardwareError> {
        self.fault("dht11")?;
        let (temperature, humidity) = self.environment.ok_or(HardwareError::Unavailable)?;
        if now_ns == 0
            || !temperature.is_finite()
            || !humidity.is_finite()
            || !(0.0..=100.0).contains(&humidity)
        {
            return Err(HardwareError::InvalidValue);
        }
        Ok(EnvironmentalCovariate {
            observed_at_ns: now_ns,
            temperature_celsius: temperature,
            relative_humidity_percent: humidity,
        })
    }
}
impl PhysicalOffPort for HardwareSimulator {
    fn is_physical_off(&mut self) -> Result<bool, HardwareError> {
        self.fault("physical_off")?;
        Ok(self.physical_off)
    }
}

/// Edge debouncer based solely on monotonic event time.
#[derive(Clone, Debug, Default)]
pub struct TouchDebouncer {
    debounce_ns: u64,
    stable: bool,
    candidate: Option<(bool, u64)>,
}
impl TouchDebouncer {
    /// Creates a debouncer.
    #[must_use]
    pub const fn new(debounce_ns: u64) -> Self {
        Self {
            debounce_ns,
            stable: false,
            candidate: None,
        }
    }
    /// Observes one monotonic raw sample.
    pub fn observe(&mut self, at_ns: u64, pressed: bool) -> Result<(), HardwareError> {
        if self.candidate.is_some_and(|(_, previous)| at_ns < previous) {
            return Err(HardwareError::NonMonotonicTime);
        }
        if pressed == self.stable {
            self.candidate = None;
        } else if self.candidate.is_none_or(|(value, _)| value != pressed) {
            self.candidate = Some((pressed, at_ns));
        }
        Ok(())
    }
    /// Emits an edge after it remains stable for the configured interval.
    pub fn poll(&mut self, now_ns: u64) -> Result<Option<TouchEvent>, HardwareError> {
        let Some((pressed, since)) = self.candidate else {
            return Ok(None);
        };
        if now_ns < since {
            return Err(HardwareError::NonMonotonicTime);
        }
        if now_ns - since < self.debounce_ns {
            return Ok(None);
        }
        self.stable = pressed;
        self.candidate = None;
        Ok(Some(if pressed {
            TouchEvent::Pressed
        } else {
            TouchEvent::Released
        }))
    }
}

/// Verified physical wiring profile; no direct memory access or unsafe code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrowPiProfile {
    /// Profile schema version.
    pub profile_version: u16,
    /// Verified board revision.
    pub board_revision: String,
    /// OLED I2C address.
    pub oled_i2c_address: u8,
    /// RGB GPIO pins.
    pub rgb_pins: [u8; 3],
    /// Touch input pin.
    pub touch_pin: u8,
    /// DHT11 input pin.
    pub dht11_pin: u8,
    /// Visible sensing indicator pin.
    pub sensing_indicator_pin: u8,
    /// Physical RF-off input pin.
    pub physical_off_pin: u8,
}
impl CrowPiProfile {
    /// Accepts only a non-zero version, named revision, legal OLED address, and unique GPIOs.
    pub fn validate(self) -> Result<Self, HardwareError> {
        let mut pins = vec![
            self.touch_pin,
            self.dht11_pin,
            self.sensing_indicator_pin,
            self.physical_off_pin,
        ];
        pins.extend(self.rgb_pins);
        pins.sort_unstable();
        if self.profile_version == 0
            || self.board_revision.trim().is_empty()
            || !(0x08..=0x77).contains(&self.oled_i2c_address)
            || pins.windows(2).any(|pair| pair[0] == pair[1])
        {
            Err(HardwareError::InvalidProfile)
        } else {
            Ok(self)
        }
    }
}

/// Result of independently attempting all outputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputSnapshot {
    /// Display attempt result.
    pub display: Result<(), HardwareError>,
    /// RGB attempt result.
    pub rgb: Result<(), HardwareError>,
    /// Indicator attempt result.
    pub indicator: Result<(), HardwareError>,
}

/// Renders all outputs independently so one peripheral cannot freeze the rest.
pub fn render_fault_isolated<D: DisplayPort, R: RgbPort, I: SensingIndicatorPort>(
    display: &mut D,
    rgb: &mut R,
    indicator: &mut I,
    projection: &HardwareProjection,
) -> OutputSnapshot {
    OutputSnapshot {
        display: display.render(projection),
        rgb: rgb.set_rgb(projection.rgb, projection.brightness_percent),
        indicator: indicator.set_visible(projection.sensing_visible),
    }
}

/// Stable hardware failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HardwareError {
    /// Board profile is unversioned, unsupported, or pin-conflicted.
    InvalidProfile,
    /// Peripheral value is malformed or out of range.
    InvalidValue,
    /// Peripheral is unavailable.
    Unavailable,
    /// Peripheral I/O operation failed.
    Io,
    /// Input event time moved backwards.
    NonMonotonicTime,
}
