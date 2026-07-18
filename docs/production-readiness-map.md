# Production Readiness Decision Map

This map shows how the ADR set covers the path from architecture to a commercially operated system. It is a completeness aid, not evidence that any decision has been implemented.

## Coverage matrix

| Readiness domain | Governing ADRs | Implementation evidence required |
|---|---|---|
| Rust architecture and boundaries | 001, 012, 013, 019 | Cargo graph checks, API/contract tests, no forbidden dependencies |
| Local deterministic runtime | 002, 014, 015, 017 | replay equality, overload/failure tests, bounded resource use |
| RF capture and calibration | 004, 029, 043, 044 | qualified Pi/Nexmon image, capture fixtures, site acceptance |
| Evidence and claim separation | 003, 005, 006, 008 | typed schemas, abstention tests, promoted capability records |
| DSP and features | 030, 031, 050 | golden vectors, numerical parity, optimized fidelity benchmarks |
| Edge inference | 032, 033, 034 | signed model packages, parity, calibration/OOD/selective-risk results |
| Personalization | 007, 035 | immutable anchors, leakage controls, prospective improvement evidence |
| Data and studies | 036, 037 | dataset cards, frozen splits, synchronized reference agreement |
| Persistence and lifecycle | 018, 028, 049, 052 | corruption recovery, restore, deletion, reset, audit tests |
| Privacy and consent | 009, 027, 028, 052 | fail-closed capture, access control, deletion and export verification |
| Security and cryptography | 023–027, 046 | threat model, SBOM, key tests, penetration tests, incident exercises |
| Hardware and peripherals | 011, 038–040 | HIL, fault injection, soak, power/thermal and accessibility results |
| UI and interaction | 010, 040 | deterministic projection tests and user studies |
| Configuration and capability control | 016, 053 | signed profiles/policy, migration, rollback, unauthorized-enable tests |
| Observability and operations | 020, 039, 046, 048, 056 | SLO dashboards, support exercises, post-market review process |
| Performance | 021, 038, 050 | reference-Pi tail latency, thermal steady state, regression gates |
| Quality and testing | 022, 038, 041, 045 | traceable automated/manual verification and release evidence |
| Delivery and updates | 026, 041, 051 | reproducible signed builds, atomic rollback, stable operator contracts |
| TypeScript integrations | 042 | sandbox, IPC contract, resource/failure isolation tests |
| Installation and field scope | 043, 044 | commissioning workflow and enforced operating envelope |
| Safety | 005, 015, 045 | hazard log, safe states, residual-risk approval, control traceability |
| Claims and regulatory | 008, 044, 047 | intended use, legal review, claim-evidence matrix, quality records |
| Commercial support | 048, 049, 054 | warranty/support policy, runbooks, lifecycle and support matrices |
| Production release | 055 | cross-functional evidence-backed readiness decision |
| Post-market operation | 046, 048, 056 | complaint/incident/corrective-action and continuous risk review |

## Production-ready definition

A release is production ready only when:

1. applicable ADRs are accepted and their implementation evidence is linked;
2. every enabled capability has passed scientific, quality, safety, security, privacy, performance, and operating-envelope gates;
3. all release artifacts are signed, reproducible, compatible, rollback-tested, and supported;
4. the reference hardware and target installation pass commissioning and soak tests;
5. intended use, claims, consent, data lifecycle, and jurisdictional requirements are approved;
6. support, update, incident, recovery, warranty, and post-market processes are staffed and exercised;
7. ADR-055 records the exact release, hardware, market, model, evidence, residual risks, and approvals.

## Decision-set evolution

These 56 ADRs cover the currently knowable production architecture. Implementation can reveal a materially new choice—such as a different chipset, regulated medical intended use, fleet cloud service, multi-person separation, or manufacturing design. Such a choice requires a new sequential ADR that amends or supersedes affected decisions; “all ADRs” means complete for the defined scope, not frozen forever.
