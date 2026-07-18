# Phase 07 observation replay benchmark

Run `bash scripts/benchmark-observation.sh run`. The release benchmark executes
the pinned signal-only DSP and artifact closure over 512 deterministic complex
radio frames for 100 iterations. It requires identical SHA-256 artifacts,
numerical delta <=1e-12, at least 100 windows/second, and at least 100,000
frames/second. Generated JSON is retained under ignored `target/benchmarks`.

Development-host result on 2026-07-18 (x86_64 release profile): 100 windows and
51,200 source frames in 43.432 ms; 2,302.44 windows/second and 1,178,851.34
frames/second; exact repeated digest
`9697803f6d421d8c6f388f554f5a06486ec1239431f9a0594a48b1fd900f59d1`;
reported numerical delta 0.0.

This is an in-memory synthetic numerical/performance baseline, not reference-Pi,
live-radio, scientific-validity, or physiological evidence. Cross-architecture
replay must additionally satisfy the declared DSP tolerance profile and retain
the complete compiler/target/environment record. Reference Pi CPU, RSS,
temperature, power, end-to-end queue latency, and 24-hour stability remain
pending hardware execution.
