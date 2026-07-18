# Raspberry Pi 4 radio compatibility contract

Live CSI is disabled unless a freshly probed environment exactly matches a
reviewed, signed compatibility manifest. The Rust adapter compares every field
before launching the fixed `/usr/local/libexec/ste-rvcsi-capture` helper and
rechecks capture policy first. It invokes the helper directly with structured
arguments; no shell, command string, PATH lookup, or unvalidated interface token
is used.

The production manifest must pin:

| Field | Qualification requirement |
| --- | --- |
| Board/chipset | Raspberry Pi 4 revision and BCM43455 identifier |
| OS | Exact image SHA-256 and 64-bit Pi OS release |
| Kernel | Exact running release used to build/test firmware integration |
| Firmware | Patched firmware SHA-256 and reviewed Nexmon commit |
| rvCSI | Exact wire/tool version |
| Network | AP identity, 5 GHz band, channel, bandwidth, packet source |
| Installation | Antenna/device geometry and calibration profile |

`development_pi4_fixture()` exists only for deterministic unit tests. Its
placeholder digests are not a known-good image and must never be promoted. No
production digest is checked in until the reference image is built, scanned,
reproduced, and accepted on hardware.

Qualification records report accepted, rejected, missing, and backpressured
counts separately. Initial software classification is accepted at >=95% valid
with no backpressure, degraded at >=80%, and rejected below 80%. Scientific and
site-specific gates may be stricter and cannot be weakened by this classification.

**Physical Pi 4/Nexmon/rvCSI compatibility: pending reference-hardware run.**
