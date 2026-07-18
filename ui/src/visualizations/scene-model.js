const KINDS = new Set([
  'topology', 'provenance', 'pipeline', 'readiness', 'neighborhood', 'constellation', 'coverage', 'containment',
])

export const RESOURCE_BUDGET = Object.freeze({ nodes: 96, edges: 192, pixelRatio: 1.5, fps: 30 })

export function seededRandom(seed = 'ste-default') {
  let state = 2166136261
  for (const char of String(seed)) state = Math.imul(state ^ char.charCodeAt(0), 16777619)
  return () => {
    state += 0x6d2b79f5
    let value = state
    value = Math.imul(value ^ (value >>> 15), value | 1)
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61)
    return ((value ^ (value >>> 14)) >>> 0) / 4294967296
  }
}

function safeText(value, fallback) {
  const text = typeof value === 'string' ? value.trim() : ''
  return text.slice(0, 120) || fallback
}

export function normalizeSceneData(data = {}, kind = 'topology', seed = 'ste-default') {
  const sceneKind = KINDS.has(kind) ? kind : 'topology'
  const random = seededRandom(`${seed}:${sceneKind}`)
  const sourceNodes = Array.isArray(data.nodes) ? data.nodes : []
  const nodes = sourceNodes.slice(0, RESOURCE_BUDGET.nodes).map((node, index) => ({
    id: safeText(node?.id, `node-${index + 1}`),
    label: safeText(node?.label, `Node ${index + 1}`),
    status: ['healthy', 'warning', 'blocked', 'experimental', 'unknown'].includes(node?.status)
      ? node.status
      : 'unknown',
    scope: safeText(node?.scope, 'unspecified'),
    provenance: safeText(node?.provenance, 'fixture'),
    uncertainty: Number.isFinite(node?.uncertainty) ? Math.min(1, Math.max(0, node.uncertainty)) : 1,
    x: Number.isFinite(node?.x) ? node.x : random() * 12 - 6,
    y: Number.isFinite(node?.y) ? node.y : random() * 7 - 3.5,
    z: Number.isFinite(node?.z) ? node.z : random() * 8 - 4,
  }))
  const ids = new Set(nodes.map(({ id }) => id))
  const edges = (Array.isArray(data.edges) ? data.edges : [])
    .filter((edge) => ids.has(edge?.from) && ids.has(edge?.to) && edge.from !== edge.to)
    .slice(0, RESOURCE_BUDGET.edges)
    .map((edge) => ({ from: edge.from, to: edge.to, label: safeText(edge.label, 'evidence flow') }))
  return {
    kind: sceneKind,
    nodes,
    edges,
    timestamp: safeText(data.timestamp, 'not supplied'),
    units: safeText(data.units, 'illustrative coordinates'),
    provenance: safeText(data.provenance, 'deterministic fixture'),
    label: safeText(data.label, `${sceneKind} visualization`),
    experimental: data.experimental !== false,
  }
}
