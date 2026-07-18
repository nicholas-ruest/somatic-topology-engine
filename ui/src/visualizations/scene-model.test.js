import { describe, expect, it } from 'vitest'
import { normalizeSceneData, RESOURCE_BUDGET, seededRandom } from './scene-model.js'
import { createVisualizationFixture } from './fixtures.js'

describe('visualization scene model', () => {
  it('is deterministic for a seed', () => {
    const first = normalizeSceneData({ nodes: [{ id: 'a' }, { id: 'b' }] }, 'pipeline', 'known-seed')
    const second = normalizeSceneData({ nodes: [{ id: 'a' }, { id: 'b' }] }, 'pipeline', 'known-seed')
    expect(first).toEqual(second)
    expect([...Array(4)].map(seededRandom('x'))).toEqual([...Array(4)].map(seededRandom('x')))
  })

  it('bounds resources and removes unsafe edges', () => {
    const nodes = Array.from({ length: 140 }, (_, index) => ({ id: `n${index}`, label: 'x'.repeat(200) }))
    const edges = Array.from({ length: 250 }, () => ({ from: 'n0', to: 'n1' }))
    edges.push({ from: 'missing', to: 'n0' }, { from: 'n0', to: 'n0' })
    const model = normalizeSceneData({ nodes, edges })
    expect(model.nodes).toHaveLength(RESOURCE_BUDGET.nodes)
    expect(model.edges).toHaveLength(RESOURCE_BUDGET.edges)
    expect(model.nodes[0].label).toHaveLength(120)
  })

  it('fails descriptive metadata to conservative values', () => {
    const model = normalizeSceneData({ nodes: [{ status: 'unsupported', uncertainty: 7 }] }, 'unsupported')
    expect(model.kind).toBe('topology')
    expect(model.nodes[0]).toMatchObject({ status: 'unknown', uncertainty: 1, provenance: 'fixture' })
    expect(model.experimental).toBe(true)
  })

  it.each(['topology', 'provenance', 'pipeline', 'readiness', 'neighborhood', 'constellation', 'coverage', 'containment'])('provides a deterministic %s fixture', (kind) => {
    const fixture = createVisualizationFixture(kind)
    expect(fixture.nodes.length).toBeGreaterThanOrEqual(3)
    if (fixture.nodes.some(({ id }) => id === 'readiness')) {
      expect(fixture.nodes.find(({ id }) => id === 'readiness').status).toBe('blocked')
    }
    expect(normalizeSceneData(fixture, kind, 'fixture').kind).toBe(kind)
  })

  it('keeps personalization fixtures participant-scoped', () => {
    const fixture = createVisualizationFixture('neighborhood')
    expect(fixture.units).toContain('not cross-participant')
    expect(fixture.nodes.map(({ id }) => id)).toEqual(['memory', 'consent', 'validation'])
  })
})
