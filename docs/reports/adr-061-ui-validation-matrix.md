# ADR-061 UI validation matrix

This matrix separates reproducible repository evidence from evidence requiring a real browser, people, hardware, or sustained execution. An automated pass does not satisfy an external gate.

| Area | Repository automation | Current result | External release gate |
|---|---|---|---|
| Roles and routes | jsdom renders every authorized workbench for participant, operator, support, validation, security, and release roles; forbidden navigation and deep links are checked | Pass | Task-based role review with production identity sessions |
| Eleven functional workbenches | Contract scenario traverses every role-authorized workspace, nested tabs, provenance, print metadata, and shared framework | Pass | End-to-end task completion against live Rust services and representative datasets |
| Commands and workflows | Unit/scenario coverage for start, load, confirm, resume, retry, cancel, all states, progress, terminal receipts, stale versions, conflicts, and duplicate idempotency keys | Pass | Crash/fault scenarios against the durable production journal and actual effect adapters |
| Destructive actions | Exact-scope preview remains disabled without a server challenge; dialog labels, cancel behavior, version binding, and hostile values are checked | Pass | Keyboard and assistive-technology confirmation tasks against live server challenges and step-up identity |
| Hostile content | HTML metacharacters, unsafe deep-link identifiers, stale/incompatible envelopes, and receipt boundaries are tested | Pass | Browser CSP/Trusted Types review, DAST, independent penetration test, and real reverse-proxy deployment test |
| Accessibility semantics | Roles, labels, alert/status states, progress semantics, drawers, dialogs, forms, error associations, print metadata, and keyboard-focusable native controls are checked | Pass | WCAG 2.2 AA manual review at 200%/400% zoom with keyboard, touch, screen readers, reduced motion, contrast, and color-vision variants |
| Visual compatibility | Vite production build plus deterministic visualization unit tests and benchmark | Pass | Visual regression in supported Chromium/Firefox/WebKit versions and representative display densities |
| Performance | Bundle sizes and host visualization construction benchmark are recorded | Pass on development host only | Exact supported Raspberry Pi startup, memory, GPU frame time, temperature, power, multi-day stream soak, and WebGL recovery |
| Hardware interaction | Fixture UI prevents production mutation and exposes simulator/hardware distinctions | Pass for isolation | CrowPi/Pi HIL including OLED, RGB, touch, DHT, physical-off, peripheral faults, and recovery |
| Export and printing | Every workbench emits scope, units, time basis, source, quality, freshness, status, and evidence digest metadata | Pass | Authorized-data export review and printed/PDF output inspection in supported browsers |

## Reproduction

```bash
cd ui
npm ci
npm test -- --run
npm run build
npm run benchmark:visualizations
```

External gates remain pending until their signed evidence is attached to the release record. Fixture data, jsdom, and development-host measurements cannot satisfy those gates.
