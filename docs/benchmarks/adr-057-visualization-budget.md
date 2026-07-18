# ADR-057 Visualization Performance Budget

The browser visualization is progressive enhancement. Safety, consent, readiness, and command controls remain usable through semantic HTML when WebGL is unavailable.

## Enforced budgets

| Resource | Budget |
|---|---:|
| Scene nodes | 96 |
| Scene edges | 192 |
| Device pixel ratio | 1.5 maximum |
| Render cadence | 30 FPS maximum |
| Scene-model normalization | 5 ms per maximum-size model on the CI host |

The deterministic benchmark is run with `npm run benchmark:visualizations`. Browser frame time, GPU memory, thermal behavior, and long-session stability require the named Raspberry Pi/CrowPi qualification hardware and cannot be inferred from the CI host.

