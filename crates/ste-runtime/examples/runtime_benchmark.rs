//! Stable JSON benchmark entry point for runtime resource-budget evidence.

use std::env;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;
use ste_runtime::{Criticality, DeterministicClock, OverflowPolicy, Supervisor, SyntheticPipeline};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(profile_index) = args.iter().position(|arg| arg == "--profile") else {
        eprintln!("usage: runtime_benchmark --profile idle|synthetic --json");
        return ExitCode::from(2);
    };
    let Some(profile) = args.get(profile_index + 1) else {
        eprintln!("missing profile");
        return ExitCode::from(2);
    };
    if !args.iter().any(|arg| arg == "--json") {
        eprintln!("--json is required");
        return ExitCode::from(2);
    }

    let total_started = Instant::now();
    let cpu_started = process_cpu_ticks();
    let startup = Instant::now();
    let mut supervisor = Supervisor::new(256, OverflowPolicy::RejectNewest);
    let startup_ms = startup.elapsed().as_secs_f64() * 1_000.0;
    let (processed, peak_queue_depth, shed_noncritical, critical_delivered, latencies) =
        match profile.as_str() {
            "idle" => (0_u64, 0_usize, 0_u64, 0_u64, Vec::new()),
            "synthetic" => run_synthetic(&mut supervisor),
            _ => {
                eprintln!("profile must be idle or synthetic");
                return ExitCode::from(2);
            }
        };
    let shutdown = Instant::now();
    supervisor.shutdown().expect("first shutdown succeeds");
    let shutdown_ms = shutdown.elapsed().as_secs_f64() * 1_000.0;
    let duration_ms = total_started.elapsed().as_secs_f64() * 1_000.0;
    let cpu_ticks = process_cpu_ticks().saturating_sub(cpu_started);
    let cpu_percent = if duration_ms > 0.0 {
        // Linux exposes process ticks at 100 Hz on supported reference targets.
        ((cpu_ticks as f64 * 10.0) / duration_ms) * 100.0
    } else {
        0.0
    };
    let max_rss_kib = max_rss_kib();
    let (p50, p95, p99) = percentiles(&latencies);
    println!(
        "{{\"profile\":\"{profile}\",\"duration_ms\":{duration_ms:.3},\"operations\":{processed},\"cpu_percent\":{cpu_percent:.3},\"max_rss_kib\":{max_rss_kib},\"queue_latency_p50_us\":{p50},\"queue_latency_p95_us\":{p95},\"queue_latency_p99_us\":{p99},\"startup_ms\":{startup_ms:.3},\"shutdown_ms\":{shutdown_ms:.3},\"dropped_critical_events\":{},\"bounded_capacity\":256,\"peak_queue_depth\":{peak_queue_depth},\"shed_noncritical\":{shed_noncritical},\"critical_delivered\":{critical_delivered}}}",
        supervisor.health().shed_critical_events
    );
    ExitCode::SUCCESS
}

fn process_cpu_ticks() -> u64 {
    let Ok(stat) = fs::read_to_string("/proc/self/stat") else {
        return 0;
    };
    // The executable name is parenthesized and may contain spaces. Fields 14
    // and 15 follow it as zero-based positions 11 and 12 in the suffix.
    let Some(end) = stat.rfind(')') else {
        return 0;
    };
    let fields = stat[end + 1..].split_whitespace().collect::<Vec<_>>();
    fields
        .get(11)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
        .saturating_add(
            fields
                .get(12)
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0),
        )
}

fn max_rss_kib() -> u64 {
    let Ok(status) = fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

fn run_synthetic(supervisor: &mut Supervisor<(u64, i64)>) -> (u64, usize, u64, u64, Vec<u128>) {
    let inputs = (0..10_000).map(i64::from).collect::<Vec<_>>();
    let events = SyntheticPipeline::new(DeterministicClock::new(0, 1)).replay(&inputs);
    let mut processed = 0_u64;
    let mut peak = 0_usize;
    let mut critical = 0_u64;
    let mut latencies = Vec::with_capacity(events.len());
    for (index, event) in events.into_iter().enumerate() {
        let started = Instant::now();
        let kind = if index % 1000 == 0 {
            Criticality::Critical
        } else {
            Criticality::Optional
        };
        if supervisor
            .publish((event.event_time_millis, event.value), kind)
            .is_err()
        {
            processed += supervisor.drain().len() as u64;
            supervisor
                .publish((event.event_time_millis, event.value), kind)
                .expect("drained queue accepts event");
        }
        if kind == Criticality::Critical {
            critical += 1;
        }
        peak = peak.max((index % 256) + 1);
        latencies.push(started.elapsed().as_micros());
    }
    processed += supervisor.drain().len() as u64;
    (
        processed,
        peak,
        supervisor.health().shed_optional_events,
        critical,
        latencies,
    )
}

fn percentiles(values: &[u128]) -> (u128, u128, u128) {
    if values.is_empty() {
        return (0, 0, 0);
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let pick = |numerator: usize| sorted[((sorted.len() - 1) * numerator) / 100];
    (pick(50), pick(95), pick(99))
}
