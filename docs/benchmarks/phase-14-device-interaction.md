# Phase 14 simulator benchmark

Status: current-host simulator measurement pending; CrowPi HIL pending.

The deterministic benchmark applies a bounded sequence of approved projections to simulator ports, including stale and peripheral-fault states, while recording append-only audit evidence. It reports host/toolchain, update count, elapsed time, and throughput. This measures in-memory simulator mechanics only.

Physical qualification must measure the exact CrowPi revision and signed device profile: OLED and RGB p50/p95/p99/max update latency, touch debounce behavior, DHT latency, bus recovery, indicator independence, CPU/RSS, queue delay, power, temperature, throttling, fault injection, and soak behavior. Simulator results must never be relabeled as GPIO/I2C/SPI hardware performance.
