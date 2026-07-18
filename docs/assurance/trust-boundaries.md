# Trust Boundaries and Data-Flow Baseline

**Record:** ASSURANCE-ARCH-001
**Version:** 1.0.0
**Status:** Accepted baseline
**Owner:** Product Security Lead
**Review due:** 2026-10-18 or upon any interface, hardware, deployment, or data-class change

## Boundary diagram

```text
 Untrusted radio space                Physically accessible edge device
 AP/traffic/attackers                 +----------------------------------------+
        | TB-01 RF/Nexmon             | patched firmware + kernel driver       |
        v                              |       | TB-02 device/privilege          |
 [CSI capture hardware] --------------+-------v--------------------------------|
                                       | isolated bounded capture adapter       |
 Imported replay/media -- TB-03 ------>| parser + anti-corruption layer         |
                                       |       | ValidatedCsiFrameV1 only        |
                                       |       v                                |
                                       | Rust policy gate -> bounded runtime    |
 Participants -- TB-04 physical/UI --->|       |                                |
 Admin/support -- TB-05 local admin --->|       +--> encrypted class stores      |
                                       |       |         ^ TB-07 storage/media   |
                                       |       +--> approved UI projections     |
                                       |       |                                |
 Optional TypeScript sidecar TB-06 <-->| authenticated versioned IPC gateway    |
                                       +----------------------------------------+
                                                   | TB-08 signed update/export
 Release/support systems --------------------------+ (network optional/offline)
```

The Rust policy decision point is the mandatory chokepoint before frame publication. Crossing into the device, process, or cryptographic boundary does not imply authorization to sense.

## Boundary controls

| ID | Boundary and trust assumption | Principal risks | Mandatory controls | Verification |
|---|---|---|---|---|
| TB-01 | Untrusted RF/AP/traffic to patched Nexmon/rvCSI stack; all frames hostile | malformed CSI, firmware exploit, spoofing, flood, provenance forgery | pinned compatibility manifest; reviewed patches; least-privileged capture; bounded DMA/socket/queues; parser limits before allocation; finite/range checks; sequence/gap/provenance retention | `VT-RADIO-001` malformed/fuzz corpus; `VT-RADIO-002` flood/backpressure; firmware SBOM/signature evidence |
| TB-02 | Kernel/device interface to isolated capture adapter | privilege escalation, kernel compromise, unsafe memory, device substitution | minimal capabilities; no general shell/network authority; process/device isolation; device identity/allowlist; watchdog; reviewed unsafe boundary; restart budget and safe disable | `VT-PLAT-001` privilege audit; `VT-UNSAFE-001`; HIL fault injection |
| TB-03 | Files, PCAP/rvcsi replay, removable media to importer | parser exploit, decompression bomb, path traversal, forged metadata/model poisoning | streaming size/depth/time limits; no symlink/path escape; schema/version/signature checks; content-addressing; quarantine; fuzzing; never inherit authorization from file | `VT-IMPORT-001` adversarial corpus; parser fuzz targets; provenance tests |
| TB-04 | Participants/physical environment to buttons, sensing indicator, display | covert activation, shoulder-surfing, stuck controls, unauthorized reset | persistent visible sensing state; physical off; fail-closed boot; accessible feedback; debounce/stuck tests; physical reset cannot erase audit or enable capture without authorization | `VT-UI-001`; `VT-CONSENT-001`; physical tamper review |
| TB-05 | Local administrator/support client to privileged API/CLI | stolen session, confused deputy, CSRF/replay, overbroad support, destructive misuse | unique device identity; authenticated local channel; least-privilege roles; explicit confirmation; nonce/replay protection; audit-before-success; rate limits; no capture-purpose widening | `VT-ADMIN-001` authz matrix and replay tests; `VT-AUDIT-001` |
| TB-06 | Optional TypeScript/Node sidecar to Rust IPC gateway | dependency compromise, prototype/payload attacks, data exfiltration, purpose widening, resource exhaustion | disabled by default; separate unprivileged process; no raw device/store/key access; allowlisted versioned DTOs; mutual peer auth; message/depth/rate/timeout limits; Rust re-authorizes every command; kill/degrade independently | `VT-IPC-001` contract/fuzz suite; `VT-IPC-002` malicious-sidecar tests; dependency/SBOM gate |
| TB-07 | Rust services to local storage, swap, backup, removable media | theft, rollback, tamper, remanence, disk exhaustion | per-class envelope AEAD; managed keys; anti-rollback journal/checkpoints; checksums; quotas/reserves; no secrets in swap/logs; cryptographic erasure; boot integrity verification | `VT-STORE-001` tamper/torn-write/disk-full; `VT-CRYPTO-001`; deletion test |
| TB-08 | Release/support systems to update, model, config, export, or support bundle | supply-chain compromise, rollback, poisoned model, recipient confusion, data leakage | offline signature verification; threshold/separated release roles; hashes/SBOM/provenance; version/compatibility/rollback policy; staged atomic install and recovery; encrypted recipient-bound export; local preview | `VT-UPDATE-001`; `VT-MODEL-001`; `VT-EXPORT-001`; reproducible build evidence |

## Privilege zones

- **Z0, hostile:** radio traffic, imported files, removable media, external artifacts, and all network input.
- **Z1, constrained adapters:** firmware/driver capture and optional sidecars. Compromise must not grant policy, key, journal, or privileged-command authority.
- **Z2, trusted computing base:** Rust policy gate, versioned contract validation, runtime supervisor, cryptographic service, and authorization repositories. This zone is small, dependency-minimized, and denies on uncertainty.
- **Z3, controlled persistence:** data-class-partitioned encrypted stores. Access is by narrow ports, never filesystem path possession alone.
- **Z4, human authority:** participant consent and appropriately authenticated operational roles. Administrator status never substitutes for participant authorization.

## Secrets and identities

Each device has a unique asymmetric identity. Release roots are distinct from device, export, data-encryption, admin-session, and model-signing keys. Production private keys are non-exportable where supported; software fallback has a lower, visibly reported assurance level. Shared fleet secrets and keys derived solely from serial numbers or passwords are forbidden.

## Residual boundary risks and acceptance

| Risk | Residual rationale | Owner | Disposition | Expiry |
|---|---|---|---|---|
| Kernel/firmware compromise may cross TB-02 despite isolation | Raspberry Pi/Nexmon platform cannot provide strong hardware isolation; blast radius is reduced but not eliminated | Product Security Lead | Accepted for development and controlled pilot only; blocks general release until HIL/penetration evidence and supported-version policy exist | 2026-10-18 |
| Physical possession enables destructive denial of service | Encryption and tamper evidence protect confidentiality/integrity, not device availability | Fleet Operations Lead | Accepted with documented physical deployment controls; no safety-critical reliance | 2026-10-18 |
| A legitimate export can leave system control | Recipient copies cannot be remotely erased | Privacy and Data Governance Lead | Accepted only with explicit authorization, preview, encryption, manifest, and warning | 2026-10-18 |
| Optional sidecar expands dependency/process attack surface | Isolation contains but cannot prove absence of host-kernel escape | Product Security Lead | Accepted only when feature disabled by default and sidecar conformance/penetration gates pass | 2026-10-18 |

Any control removal, new remote ingress, cloud processing, raw-data sidecar access, shared key, or bypass around the Rust policy gate invalidates this acceptance.
