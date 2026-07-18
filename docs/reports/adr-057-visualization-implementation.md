# ADR-057 Visualization Implementation Report

The visualization subsystem implements bounded spatial topology, evidence provenance, runtime pipeline, participant-scoped personalization neighborhood, model/capability constellation, commissioning coverage, fault containment, and release-readiness views through one policy-neutral scene model. Positions generated for fixtures are deterministic and seeded. Status is limited to operational states and never encodes inferred emotion, cardiac state, or another prohibited construct.

The WebGL renderer uses instancing and a single line-segment buffer. It caps geometry, edges, pixel ratio, and update cadence; pauses motion for reduced-motion and low-power modes; and disposes animation frames, listeners, geometries, materials, the renderer, and its context on teardown. WebGL loss immediately reveals the equivalent semantic table. Dynamic import keeps Three.js outside the critical HTML path.

Every visualization includes text metadata and a table containing status, scope, uncertainty, and provenance. The table remains the authoritative accessible representation and uses text-only DOM construction to prevent hostile labels from becoming markup.

Automated tests cover deterministic generation, resource limits, conservative defaults, equivalent table semantics, and hostile label handling. Reference-hardware frame time, thermal, memory, and soak evidence remains a release qualification item.
