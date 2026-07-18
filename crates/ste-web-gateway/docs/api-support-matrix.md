# ADR-058 API support matrix

| Method | Route | Session | CSRF | Cache |
|---|---|---:|---:|---|
| `GET` | `/healthz` | no | no | no-store |
| `GET` | `/api/v1/session` | yes | no | no-store |
| `GET` | `/api/v1/read-models/{area}` | yes | no | no-store |
| `GET` | `/api/v1/streams/{stream}` | yes | no | no-store |
| `POST` | `/api/v1/commands/{command}` | yes | yes | no-store |
| `GET` | `/api/v1/workflows/{id}` | yes | no | no-store |
| `GET` | `/api/v1/workflows/{id}/stream` | yes | no | no-store |

Only allowlisted names are accepted by the application adapter. Unknown versions,
routes, commands, streams, areas, workflows, or safety-relevant values fail
closed. Hashed files below `/assets/` are immutable; HTML, manifests, API data,
errors, and streams are never cached.

Every route enforces exact Host. Requests carrying Origin enforce the exact
configured origin; mutations require Origin, a server-side session cookie, CSRF,
and idempotency headers. Authentication and authorization are repeated by the
authoritative adapter for every operation.
