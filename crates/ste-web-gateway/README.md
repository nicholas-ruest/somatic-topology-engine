# STE Web Gateway

The production, loopback-only HTTP/SSE adapter specified by ADR-058. It exposes
only the versioned support matrix in `supported_routes`, serves assets after
SHA-256 manifest verification, and delegates every read, subscription, command,
and workflow query to an authoritative Rust `ApplicationServices` adapter.

The gateway is deliberately restartable and contains no sensing, persistence,
inference, safety, or recovery authority. Deploy it under an independent
supervisor. Production composition must supply secure server-side sessions and
must regenerate the verified UI asset manifest during the signed release build.

Resource ceilings are explicit in `WebConfig`. SSE clients receive sequence,
loss, cursor, and heartbeat data; they must reconnect with `Last-Event-ID`.

## Start the production adapter

Build the UI first so its SHA-256 manifest exists, then provide locally
provisioned session and policy bindings. These values are examples only; use
random secret material and an authorization reference issued by deployment
policy:

```bash
cd ui && npm ci && npm run build && cd ..
STE_SESSION_COOKIE='<at-least-32-random-characters>' \
STE_CSRF_TOKEN='<at-least-16-random-characters>' \
STE_AUTHORIZATION_REF='local-policy:v1' \
STE_PURPOSE_REF='operations' \
cargo run -p ste-web-gateway
```

The default listener is `127.0.0.1:4173` and the verified distribution is
`ui/dist`. Override them with `STE_WEB_BIND`, `STE_WEB_ORIGIN`, and
`STE_UI_DIST`. Wildcard/LAN binds, mismatched origins, missing secrets, and
tampered or unlisted assets fail before serving. The embedded journal is for a
single supervised local process; deployments requiring restart persistence must
compose `ProductionServices` with their durable `Journal` adapter.
