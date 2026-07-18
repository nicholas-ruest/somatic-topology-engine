# Phase 17 HIL and soak status

Status: **pending exact reference Raspberry Pi/CrowPi evidence**.

Synthetic fault tests cover packet loss, malformed frames, overload, AP loss, time jump, disk full, storage corruption, task death, low voltage, thermal pressure, power interruption, peripheral faults, optional-sidecar death, and safe capture disablement. These validate deterministic policy only.

No physical low-voltage, thermal, bus, storage-media, AP, CrowPi peripheral, watchdog, power-interruption, temperature/power instrumentation, or multi-day soak run is claimed. Production approval remains blocked until the exact supported hardware/image/profile executes the declared HIL matrix and soak duration with signed raw evidence, tail distributions, recovery records, and zero unacceptable safety violations.
