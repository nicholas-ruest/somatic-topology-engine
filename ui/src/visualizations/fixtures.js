const nodes = [
  ['capture', 'Radio capture', 'healthy', 'acquisition'],
  ['observe', 'Signal observations', 'healthy', 'observation'],
  ['respiration', 'Respiration estimate', 'experimental', 'physiology'],
  ['inference', 'Task-specific inference', 'blocked', 'inference'],
  ['memory', 'Participant memory', 'warning', 'personalization'],
  ['device', 'OLED and RGB projection', 'healthy', 'interaction'],
  ['consent', 'Consent authority', 'healthy', 'governance'],
  ['validation', 'Validation evidence', 'warning', 'experiments'],
  ['registry', 'Model registry', 'healthy', 'models'],
  ['operations', 'Operations and recovery', 'healthy', 'operations'],
  ['security', 'Security and incidents', 'healthy', 'security'],
  ['readiness', 'Commercial readiness', 'blocked', 'release'],
].map(([id, label, status, scope]) => ({ id, label, status, scope, uncertainty: status === 'healthy' ? 0.08 : 0.7, provenance: 'ADR-057 fixture' }))

const edges = [
  ['capture', 'observe'], ['observe', 'respiration'], ['observe', 'inference'], ['consent', 'capture'],
  ['validation', 'registry'], ['registry', 'respiration'], ['registry', 'inference'], ['memory', 'inference'],
  ['inference', 'device'], ['respiration', 'device'], ['security', 'operations'], ['operations', 'readiness'],
  ['validation', 'readiness'], ['security', 'readiness'],
].map(([from, to]) => ({ from, to, label: 'bounded dependency' }))

const kindScopes = {
  provenance: ['capture', 'observe', 'respiration', 'inference', 'validation', 'registry'],
  neighborhood: ['memory', 'consent', 'validation'],
  constellation: ['registry', 'respiration', 'inference', 'validation', 'security'],
  coverage: ['capture', 'device', 'consent', 'operations', 'security', 'readiness'],
  containment: ['capture', 'observe', 'device', 'operations', 'security', 'readiness'],
  readiness: ['validation', 'registry', 'operations', 'security', 'readiness'],
}

const descriptions = {
  neighborhood: ['Participant-scoped personalization neighborhood', 'relationship distance; not cross-participant similarity'],
  constellation: ['Signed model and capability constellation', 'bounded package dependency'],
  coverage: ['Commissioning qualification coverage', 'qualified check dependency'],
  containment: ['Fault propagation and containment', 'failure boundary dependency'],
  provenance: ['Evidence provenance and window lineage', 'bounded evidence dependency'],
}

export function createVisualizationFixture(kind = 'topology') {
  const selected = kindScopes[kind]
  const fixtureNodes = selected ? nodes.filter(({ id }) => selected.includes(id)) : nodes
  const selectedIds = new Set(fixtureNodes.map(({ id }) => id))
  const [label = `${kind} system map`, units = 'illustrative topology'] = descriptions[kind] || []
  return {
    kind,
    label,
    timestamp: '2026-07-18T00:00:00Z',
    units,
    provenance: 'deterministic ADR-057 demonstration fixture',
    experimental: true,
    nodes: fixtureNodes.map((node) => ({ ...node })),
    edges: edges.filter(({ from, to }) => selectedIds.has(from) && selectedIds.has(to)).map((edge) => ({ ...edge })),
  }
}
