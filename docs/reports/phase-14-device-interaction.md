# Phase 14 device-interaction evidence

Status: deterministic simulator profile; physical CrowPi qualification pending.

## Supported interaction claims

The live surface renders only fixed Rust projection variants: unauthorized, calibrating, contaminated, insufficient evidence, stale, unavailable, signal quality, and explicit anchor confirmation. Text always accompanies color. RGB never represents inferred valence; environmental readings remain timestamped covariates and cannot change model confidence.

Simulator and physical adapters implement the same display, RGB, touch, sensing-indicator, environmental-sensor, and peripheral-health ports. The physical profile is configuration metadata only until the exact CrowPi revision, pin mapping, electrical limits, display controller, and HIL fixtures are verified. No hardware qualification is claimed by simulator success.

## Safety and accessibility controls

- sensing authorization drives a dedicated visible indicator independently of optional display/RGB health;
- display and RGB snapshots are deterministic and use approved text plus redundant icon/pattern/color semantics;
- brightness is bounded and transitions are stable;
- evidence age produces an explicit stale projection;
- touch anchors require debounce and confirmation; bounce cannot append multiple anchors;
- DHT data is labeled environmental and stale/fault-aware;
- one peripheral failure is recorded in the append-only audit journal and cannot freeze healthy peripherals or core supervision;
- unauthorized and fault states fail safe.

## Evidence status

Run `scripts/validate-device-interaction.sh` for repository, audit, simulator snapshot, accessibility, debounce, staleness, and peripheral-fault evidence. Physical CrowPi HIL, power/thermal, long-duration soak, exact GPIO qualification, and accessibility user review remain pending.

The executable suite passes fourteen device-interaction tests, three governed simulator CLI tests, and strict Clippy. It covers simulator/physical-off port parity, touch edges and debounce, accessible snapshots, independent peripheral fault injection, visible-indicator attempts, explicit stale/fault states, audit persistence, and rejection of unverified/conflicting HIL profiles.
