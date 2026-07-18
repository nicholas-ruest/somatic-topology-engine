# ADR-059 Query Plane Benchmark Plan

The query plane uses fixed-capacity rings and bounded queries. Release measurement must record p50/p95/p99 for append/resume, 10,000-point aggregation, and deterministic seek on the target Raspberry Pi, plus memory under slow-consumer and concurrent-query load.

Acceptance budgets are: no capture blocking, ring memory proportional only to configured capacity, history response at most 10,000 buckets, and replay output byte-identical across repeated seeks. Host timings are informative only; Pi thermal and long-session gates remain required.
