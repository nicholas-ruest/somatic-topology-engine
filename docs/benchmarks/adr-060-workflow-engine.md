# ADR-060 workflow-engine benchmark plan

The workflow engine has no unbounded queues and performs work proportional to one workflow's event stream. Release benchmarking must measure create, optimistic append, full projection rebuild, lock conflict, authorization preemption, and receipt serialization at 50, 500, and 5,000 events. The acceptance targets are p99 below 10 ms for command decisions and below 50 ms for recovery of a 5,000-event instance on the supported Raspberry Pi. Storage implementations must separately fault-test atomic append/fsync behavior; in-memory numbers are not durable-storage evidence.

The benchmark corpus must include destructive confirmations, concurrent activation conflicts, cancellation, compensation, expiry, and revocation. Results record hardware, build profile, compiler, event count, sample count, p50/p95/p99, allocation high-water mark, and journal medium. No synthetic host result may satisfy Pi qualification.
