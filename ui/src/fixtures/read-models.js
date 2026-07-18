export const fixtureEnvelope = Object.freeze({
  schemaVersion: '1.0.0', source: 'deterministic-fixture', emittedAt: '2026-07-18T12:00:00Z',
  sequence: 4207, provenance: 'fixture:adr-057:seed-57', stale: false, capabilityState: 'blocked',
})

export const fixtureMetrics = Object.freeze({
  capture: 'AUTHORIZED', packetRate: 187, gaps: 0.018, signalQuality: 0.91,
  latencyMs: 34, queue: 0.12, respiration: 'ABSTAINED', respirationReason: 'Real reference evidence absent',
  inference: 'NO CLAIM', operatingEnvelope: 'WITHIN', thermalC: 51.2, powerW: 8.4,
})

export const notices = Object.freeze([
  { level: 'blocked', title: 'Commercial readiness', text: 'NOT_APPROVED — external acceptance evidence remains incomplete.' },
  { level: 'safe', title: 'Sensing active', text: 'Authorized fixture capture is active. No production evidence is being generated.' },
])
