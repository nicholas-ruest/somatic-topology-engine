# Local operations UI

The ADR-057 interface lives in `ui/`. It is a Vite JavaScript application using Tailwind CSS and progressively enhanced Three.js scenes. It is a presentation client; Rust remains authoritative for every read model and command.

## Development

```bash
cd ui
npm ci
npm run dev
```

The development application binds to `127.0.0.1` and starts in conspicuous deterministic fixture mode. Fixture mode cannot perform production mutations or generate production evidence.

## Validation and production build

Run the complete UI gate from the repository root:

```bash
bash scripts/validate-ui.sh
```

The production assets are emitted to ignored `ui/dist/` without source maps. The build generates `ui/dist/asset-manifest.json` with the exact membership, size, path, and SHA-256 digest expected by `ste-ui-gateway::AssetManifest`. The release pipeline signs the enclosing release evidence and serves only files that verify against this manifest.

Do not deploy the Vite development server. Production assets must be locally packaged, offline, and exposed only through the loopback presentation host configured with the restrictive CSP and exact origin policy enforced by `ste-ui-gateway`.

## Live integration boundary

The browser uses same-origin, versioned read-model and command endpoints. A production request carries its short-lived role-bound session, device binding, CSRF value, origin, schema version, and—on mutations—an idempotency key. The gateway reauthorizes and dispatches the exact Rust application command. JavaScript route visibility is never authorization.

The stream is bounded and sequence-aware. Schema mismatches, dropped history, stale evidence, authentication failures, or disconnection render typed unavailable states rather than preserving a stale-looking value.

## Functional coverage

The role-scoped navigation covers all sixteen ADR-057 areas: live overview, topology, radio, observations, physiology, inference, personalization, device interaction, consent, validation, models, reliability, commissioning, operations, security, and release readiness.

The current release decision remains `NOT_APPROVED`. The UI cannot override capability policy or readiness evidence.

## Accessibility and graphics

Every Three.js scene has a semantic table fallback. Reduced-motion, low-power, unsupported-WebGL, and context-loss paths stop animation and preserve the functional view. Status is communicated with text and structure in addition to color. Reference-device screen-reader, color-vision, zoom, HIL, GPU-memory, thermal, and long-session reviews remain release evidence requirements rather than claims made by the development-host suite.
