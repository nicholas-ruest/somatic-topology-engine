# Phase 14 simulator benchmark

Status: current-host simulator measurement executed; CrowPi HIL pending.

The deterministic benchmark applies a bounded sequence of approved projections to simulator ports, including stale and peripheral-fault states, while recording append-only audit evidence. It reports host/toolchain, update count, elapsed time, and throughput. This measures in-memory simulator mechanics only.

Physical qualification must measure the exact CrowPi revision and signed device profile: OLED and RGB p50/p95/p99/max update latency, touch debounce behavior, DHT latency, bus recovery, indicator independence, CPU/RSS, queue delay, power, temperature, throttling, fault injection, and soak behavior. Simulator results must never be relabeled as GPIO/I2C/SPI hardware performance.

One run on 2026-07-18 used Rust 1.85.1 on x86_64 Linux and applied 100,000 in-memory text/color snapshots with 100,000 audit events in 8,047,336 ns (approximately 12.43 million updates/second). This is a single simulator observation, not peripheral timing, tail latency, CrowPi throughput, or physical qualification.
