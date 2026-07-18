<div align="center">

# Somatic Topology Engine

**Local-first, wearable-free ambient sensing for the edge.**
Consent-gated Wi‑Fi sensing, on-device inference, and a local operations console — no cloud, no wearables, no compromise on privacy.

[![License](https://img.shields.io/badge/license-MIT-3fd67a)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-4ea1ff)](rust-toolchain.toml)
[![Platform](https://img.shields.io/badge/platform-Raspberry%20Pi%204-c51a4a)](#reference-hardware)
[![Status](https://img.shields.io/badge/release-NOT__APPROVED-orange)](#release-status)
[![Data](https://img.shields.io/badge/data-local--only-3fd67a)](#privacy-by-design)

<br>

<img src="docs/assets/dashboard-overview.png" alt="Somatic Topology Engine local operations dashboard" width="850">

</div>

## What it is

Somatic Topology Engine (STE) turns a Wi‑Fi radio into an ambient sensor. It
observes how radio signals move through a room to estimate occupancy,
motion, and — under tightly controlled conditions — respiration candidates,
without a camera, microphone, or anything worn on the body.

Everything runs **locally, on the device, in Rust**. There is no cloud
round-trip and no raw signal ever leaves the box. Every capability is
consent-gated, purpose-bound, and time-limited, and every estimate the system
is not confident about is **abstained** rather than guessed — the UI says
`no claim` instead of making one up.

It ships with a local, role-aware web console for operating, validating, and
auditing the system in real time — no external dashboard, telemetry
pipeline, or third-party service required.

## What it does

- **Sensing** — captures and windows Wi‑Fi Channel State Information (CSI) to
  observe occupancy, motion, and signal quality in a room.
- **Physiology estimation** — derives conservative, uncertainty-aware
  respiration candidates, fails closed, and refuses to output a number it
  can't stand behind.
- **State inference** — projects a policy-approved, evidence-bounded read
  model instead of an opaque black-box score.
- **Personalization** — builds a per-participant, cryptographically scoped
  memory that never mixes across participants.
- **Device interaction** — drives the on-device display, status LEDs, and
  touch controls so a room's occupant always has a physical, human-readable
  signal of what the system is doing.
- **Consent & governance** — every session is purpose-bound, scoped, and
  revocable in one action; nothing is captured outside an active
  authorization.
- **Local operations console** — a role-based web UI (participant, operator,
  support, validation, security, release) for live monitoring, replay,
  commissioning, and guarded operator workflows.

## Reference hardware

STE targets a small, self-contained edge device — no server rack, no GPU
cluster, no data center:

| Component | Reference spec |
| --- | --- |
| Compute | Raspberry Pi 4 |
| Wi‑Fi chipset | Broadcom BCM43455 (CSI-capable via patched firmware) |
| Enclosure / peripherals | CrowPi kit — OLED display, RGB status LEDs, capacitive touch controls, DHT11 temperature/humidity sensor |
| Network | Dedicated 5 GHz access point on a fixed channel and bandwidth |
| Footprint | Single board, single room, no external services |

The same codebase also runs entirely in a deterministic **fixture / simulator
mode** on a laptop or CI runner — with no physical hardware attached — for
development, demos, and testing.

<details>
<summary><strong>📸 Product tour — more screenshots</strong></summary>
<br>

The console covers sixteen role-scoped areas: live overview, spatial
topology, radio, observations, physiology, inference, personalization,
device interaction, consent, validation, models, reliability, commissioning,
operations, security, and release readiness.

<img src="docs/assets/dashboard-topology.png" alt="Spatial signal topology view with live 3D visualization" width="850">

*Spatial signal topology — an illustrative, non-localization view of the
sensed environment, rendered live in the browser.*

</details>

<details>
<summary><strong>🚀 Getting started</strong></summary>
<br>

**Requirements:** the Rust toolchain pinned in `rust-toolchain.toml`, and
Node.js for the local console.

Build and verify the Rust workspace:

```sh
cargo build --workspace --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
bash scripts/verify.sh
```

Run the local operations console in fixture/simulator mode (no hardware
required):

```sh
cd ui
npm ci
npm run dev
```

Build and gate the production console bundle:

```sh
bash scripts/validate-ui.sh
```

See [`docs/operations/local-ui.md`](docs/operations/local-ui.md) for local
development details.

</details>

<details id="privacy-by-design">
<summary><strong>🔒 Privacy, safety & research scope</strong></summary>
<br>

- **Not a medical device.** STE is research infrastructure. Its outputs must
  never be interpreted as a diagnosis or clinical advice.
- **Consent-gated by design.** Sensing only runs inside an active,
  purpose-bound, time-limited authorization, and can be revoked in a single
  action that stops capture immediately.
- **Local-only data.** Raw signal data and derived evidence stay on-device.
  Nothing is uploaded, and no vendor telemetry is collected.
- **Fails closed, not open.** When the system isn't confident in an
  estimate, the interface reports an explicit `no claim` / `abstained`
  state rather than a plausible-looking guess.
- **Participant-scoped memory.** Personalization data is cryptographically
  scoped per participant; cross-participant queries are prohibited outright.

</details>

<details id="release-status">
<summary><strong>🧭 Release status</strong></summary>
<br>

STE is under active development. The current production release decision is
**`NOT_APPROVED`** — software controls alone cannot substitute for
hardware-in-the-loop testing, human validation, legal/jurisdictional review,
a commercial pilot, penetration testing, and an accessibility review. The
console always displays this status and cannot override it.

</details>

<details>
<summary><strong>📄 License</strong></summary>
<br>

Licensed under the [MIT License](LICENSE).

</details>
