# Phase 13 vector-memory benchmark

Status: deterministic Rust reference adapter current-host measurement executed; RuVector/RVF and reference Pi pending.

The benchmark inserts participant-scoped synthetic vectors, performs deterministic similarity retrieval, deletes one participant, rebuilds from the authoritative append-only journal, and proves deleted records are not retrievable while another participant remains intact. It records host/toolchain, dimensions, record/query counts, elapsed time, and throughput.

Results apply only to the bounded reference adapter and synthetic fixtures. They are not RuVector, RVF, Raspberry Pi, steady-state, memory, thermal, power, or commercial-capacity claims. A candidate RuVector/RVF adapter must be pinned, license-reviewed, verified against this port, and benchmarked on the declared Pi with p50/p95/p99/max latency, RSS, storage growth, rebuild duration, deletion behavior, corruption recovery, and soak evidence.

One run on 2026-07-18 used Rust 1.85.1 on x86_64 Linux. It encrypted and indexed 10,000 synthetic eight-dimensional records, executed 100 bounded participant-scoped queries, erased one participant, and rebuilt the retained index in 95,586,486 ns overall (approximately 104,617 inserted records/second when dividing records by total scenario time). This mixed-scenario number is a single engineering observation, not an isolated insertion/query distribution.
