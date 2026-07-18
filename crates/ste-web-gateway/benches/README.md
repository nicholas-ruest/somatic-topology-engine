# Gateway benchmark protocol

Benchmark the release binary on the target Raspberry Pi with 1, 8, and the
configured maximum concurrent clients. Measure p50/p95/p99 response latency,
resident memory, rejected overload, SSE reconnect latency, and slow-consumer
eviction. The acceptance run must include verified production assets and the
real application-service adapter; host-only synthetic measurements are not
target-hardware evidence.
