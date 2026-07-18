//! Versioned DSP golden and adversarial numerical tests.

use serde::Deserialize;
use ste_signal_observation::dsp::{
    DspError, DspGraphSpec, PrimitiveCsiFrame, ToleranceProfile, execute_dsp,
    execute_dsp_with_source_refs,
};

fn spec(window_len: usize) -> DspGraphSpec {
    DspGraphSpec {
        version: 1,
        sample_rate_hz: 10.0,
        window_len,
        saturation_magnitude: 4.0,
        presence_threshold: 0.0,
        periodicity_min_lag: 1,
        periodicity_max_lag: window_len / 2,
    }
}

fn frames(values: &[(f64, f64)]) -> Vec<PrimitiveCsiFrame> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| PrimitiveCsiFrame {
            source_ref: format!("fixture-{index}"),
            event_time_ns: (index as u64 + 1) * 100_000_000,
            subcarriers: vec![*value],
        })
        .collect()
}

#[test]
fn constant_signal_has_zero_motion_periodicity_drift_and_interference() {
    let result = execute_dsp(spec(8), &frames(&[(1.0, 0.0); 8])).unwrap();
    assert_eq!(result.motion_energy, 0.0);
    assert_eq!(result.presence_score, 0.5);
    assert_eq!(result.periodicity, 0.0);
    assert!(result.drift_per_second.abs() < 1.0e-12);
    assert_eq!(result.interference_ratio, 0.0);
    assert_eq!(result.missingness, 0.0);
}

#[test]
fn impulse_motion_and_saturation_are_explicit_and_finite() {
    let mut values = [(0.0, 0.0); 8];
    values[3] = (8.0, 0.0);
    let result = execute_dsp(spec(8), &frames(&values)).unwrap();
    assert!((result.motion_energy - 128.0 / 7.0).abs() < 1.0e-12);
    assert_eq!(result.saturation_fraction, 0.125);
    assert!(result.interference_ratio > 0.0);
    assert!(result.motion_energy.is_finite());
}

#[test]
fn sinusoid_exposes_periodicity_while_seeded_noise_exposes_interference() {
    let sinusoid = (0..32)
        .map(|index| {
            (
                1.0 + 0.5 * (std::f64::consts::TAU * index as f64 / 8.0).sin(),
                0.0,
            )
        })
        .collect::<Vec<_>>();
    let mut sine_spec = spec(32);
    sine_spec.periodicity_max_lag = 12;
    let sine = execute_dsp(sine_spec, &frames(&sinusoid)).unwrap();
    let mut state = 0x1234_5678_u64;
    let noise = (0..32)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            (0.5 + ((state >> 32) as f64 / u32::MAX as f64), 0.0)
        })
        .collect::<Vec<_>>();
    let random = execute_dsp(sine_spec, &frames(&noise)).unwrap();
    assert!(sine.periodicity > random.periodicity);
    assert!(random.interference_ratio > sine.interference_ratio);
}

#[test]
fn gaps_drift_phase_wrap_and_motion_are_preserved_separately() {
    let angles = [3.0_f64, -3.0, -2.8, -2.6, -2.4, -2.2, -2.0, -1.8];
    let mut input = angles
        .iter()
        .enumerate()
        .map(|(index, angle)| PrimitiveCsiFrame {
            source_ref: format!("phase-{index}"),
            event_time_ns: (index as u64 + 1) * 100_000_000,
            subcarriers: vec![(
                (1.0 + index as f64 * 0.1) * angle.cos(),
                (1.0 + index as f64 * 0.1) * angle.sin(),
            )],
        })
        .collect::<Vec<_>>();
    input[4].event_time_ns += 200_000_000;
    for frame in input.iter_mut().skip(5) {
        frame.event_time_ns += 200_000_000;
    }
    let result = execute_dsp(spec(8), &input).unwrap();
    assert_eq!(result.phase_wraps, 1);
    assert_eq!(result.missing_frames, 2);
    assert_eq!(result.missingness, 0.2);
    assert!(result.drift_per_second > 0.0);
    assert!(result.motion_energy > 0.0);
}

#[test]
fn nonfinite_shape_time_and_graph_fail_closed_without_panics() {
    let mut input = frames(&[(1.0, 0.0); 8]);
    input[2].subcarriers[0].0 = f64::NAN;
    assert_eq!(execute_dsp(spec(8), &input), Err(DspError::NonFiniteInput));
    let input = frames(&[(1.0, 0.0); 7]);
    assert_eq!(execute_dsp(spec(8), &input), Err(DspError::WindowShape));
    let mut input = frames(&[(1.0, 0.0); 8]);
    input[3].event_time_ns = input[2].event_time_ns;
    assert_eq!(
        execute_dsp(spec(8), &input),
        Err(DspError::NonMonotonicTime)
    );
    let mut invalid = spec(8);
    invalid.version = 99;
    assert_eq!(
        execute_dsp(invalid, &frames(&[(1.0, 0.0); 8])),
        Err(DspError::InvalidSpec)
    );
}

#[derive(Deserialize)]
struct GoldenFixture {
    graph: DspGraphSpec,
    frames: Vec<GoldenFrame>,
    expected: GoldenExpected,
}

#[derive(Deserialize)]
struct GoldenFrame {
    event_time_ns: u64,
    subcarriers: Vec<(f64, f64)>,
}

#[derive(Deserialize)]
struct GoldenExpected {
    observed_frames: u64,
    missing_frames: u64,
    phase_wraps: u64,
}

#[test]
fn recorded_csi_fixture_replays_exactly_and_serialized_graph_is_versioned() {
    let fixture: GoldenFixture =
        serde_json::from_str(include_str!("fixtures/signal/recorded-csi-golden.json")).unwrap();
    let frames = fixture
        .frames
        .into_iter()
        .map(|frame| PrimitiveCsiFrame {
            source_ref: format!("recorded-{}", frame.event_time_ns),
            event_time_ns: frame.event_time_ns,
            subcarriers: frame.subcarriers,
        })
        .collect::<Vec<_>>();
    let source_refs = frames
        .iter()
        .map(|frame| frame.source_ref.clone())
        .collect::<Vec<_>>();
    let first = execute_dsp_with_source_refs(fixture.graph, &frames, &source_refs).unwrap();
    let second = execute_dsp_with_source_refs(fixture.graph, &frames, &source_refs).unwrap();
    assert!(ToleranceProfile::STRICT_REPLAY.observations_match(&first, &second));
    assert!(ToleranceProfile::CROSS_ARCHITECTURE.observations_match(&first, &second));
    assert_eq!(first.observed_frames, fixture.expected.observed_frames);
    assert_eq!(first.missing_frames, fixture.expected.missing_frames);
    assert_eq!(first.phase_wraps, fixture.expected.phase_wraps);
    assert_eq!(first.source_refs, source_refs);
    let graph_json = serde_json::to_string(&fixture.graph).unwrap();
    assert!(graph_json.contains("\"version\":1"));
}
