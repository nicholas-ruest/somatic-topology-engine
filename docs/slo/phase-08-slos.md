# Phase 08 service objectives and owners

These development gates establish measurable ownership; they do not yet qualify
the Raspberry Pi deployment.

| Objective | Threshold | Class | Owner |
| --- | ---: | --- | --- |
| Capture continuity | >=95% accepted; zero hidden loss | Release | Radio acquisition owner |
| Valid observation windows | Report clean/degraded/contaminated separately | Safety | Signal observation owner |
| Synthetic queue p95 | <=50 ms | Release | Runtime owner |
| Critical event loss | 0 | Safety | Runtime owner |
| Startup and shutdown | <=2 s each | Release | Runtime owner |
| Development idle RSS | <=256 MiB | Release | Runtime owner |
| Replay numerical delta | <=1e-12 same architecture | Scientific regression | DSP owner |
| Acquisition replay | >=100,000 frames/s | Regression | Radio acquisition owner |
| Observation replay | >=100,000 frames/s and 100 windows/s | Regression | DSP owner |
| Benchmark regression | <=10% latency/RSS increase; <=10% throughput decrease | Release | Release owner |
| Diagnostic redaction | Zero canary/sensitive fields | Security | Security/privacy owner |
| Metric/record drops | Explicit counter; no silent loss | Operational | Observability owner |
| Pi temperature | Pending measured limit/profile | Release blocker | Hardware owner |
| Pi power/undervoltage | Pending measured limit/profile | Release blocker | Hardware owner |
| Storage growth/24h soak | Pending reference run | Release blocker | Reliability owner |

Zero-valued sub-microsecond queue/shutdown observations mean “below current
timer resolution,” not zero work. Idle CPU from a probe shorter than one second
is informational and not gated. Production thresholds require representative
live acquisition, journaling, projections, display, and hardware fault loads.
