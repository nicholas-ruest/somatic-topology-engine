# Phase 15 commissioning benchmark

Status: synthetic current-host workflow executed; reference device pending.

The benchmark constructs complete synthetic sessions, records all mandatory checks and capability coverage, signs/verifies acceptance, and asserts failed coverage remains blocked. It measures Rust workflow mechanics, not probes or physical qualification. Reference-device measurement must include each real check duration, total technician time, recovery/requalification, signature verification, report storage, and failure paths on the exact supported hardware/site profile.

One run on 2026-07-18 used Rust 1.85.1 on x86_64 Linux and completed 1,000 synthetic 13-check sessions with signed/verified acceptance and explicit enabled/blocked capability coverage in 78,612,822 ns (approximately 12,721 workflows/second). This is a single software observation, not technician, probe, reference-Pi, or site-qualification performance.
