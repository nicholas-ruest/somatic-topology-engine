# Phase 06 acquisition replay baseline

Run `bash scripts/benchmark-acquisition.sh run`. The release benchmark generates
10,000 deterministic finite rvCSI frames in memory, proves two identical parses
are equal, then parses 20 iterations (200,000 frames). The executable gate
requires all generated frames accepted, zero rejection, determinism, and at
least 100,000 frames/second on the development host. Generated JSON is stored
under ignored `target/benchmarks`.

Development-host result on 2026-07-18 (x86_64 release profile): 200,000/200,000
frames accepted, zero rejected, 510,008-byte fixture, 11.516 ms elapsed,
17,366,773 frames/second, and identical repeated replay. This is a point-in-time
container baseline; rerun JSON is authoritative for regression comparison.

This parser microbenchmark excludes file I/O, privileged helper IPC, radio
capture, journaling, and publication. It is not Pi performance or live packet
continuity evidence.

Reference Pi procedure: run the same locked revision/profile for at least 30
minutes at the qualified packet rate; record accepted/rejected/missing/
backpressured distributions, p50/p95/p99 IPC latency, CPU, RSS, storage, voltage,
temperature, throttling, and replay equality. Repeat AP loss, interference,
queue saturation, and revocation. Preserve raw reports and signed environment
manifest.

**Live Pi 4 acquisition benchmark and hardware acceptance: pending.**
