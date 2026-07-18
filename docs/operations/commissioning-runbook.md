# Offline commissioning and site qualification runbook

Use an authenticated local operator session. Start with `operator doctor`, `hardware-probe`, and `capture-test`, each with a unique idempotency key and `--json` for retained evidence. Direct process invocation is intentionally denied; production composition supplies authenticated Unix-domain IPC identity and current authorization.

Commissioning must record all thirteen checks: hardware, signed firmware, peripherals/visible indicator, power, thermal, AP/link, packet rate, geometry, consent/purpose, encrypted storage, clocks, calibration, and interference. Record exact frozen coverage for every requested capability. Missing/failed mandatory checks reject the attempt. Failed capability coverage blocks that capability without changing its threshold.

Sign and verify the acceptance record offline, retain its immutable evidence references, and confirm enabled/blocked capability sets. Requalification links the previous acceptance record. Site, hardware, firmware, geometry, AP/link, calibration, or interference changes require requalification.

For failure, use `--dry-run` data/recovery operations first. Recovery mode never manufactures acceptance. Reset requires `--confirm`, current authorization, and a unique idempotency key. Preserve the prior receipt and support bundle before destructive action when policy allows.

The current repository contains simulator commissioning evidence only. Follow the hardware/HIL procedures before treating a physical CrowPi installation as qualified.
