// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'
import { createSceneFallback } from './fallback.js'
import { normalizeSceneData } from './scene-model.js'

describe('visualization fallback', () => {
  it('provides an equivalent semantic table and metadata', () => {
    const model = normalizeSceneData({
      label: 'Evidence pipeline', timestamp: '2026-07-18T00:00:00Z', units: 'events', provenance: 'signed report',
      nodes: [{ id: 'capture', label: 'Capture', status: 'healthy', scope: 'room-a', uncertainty: 0.12, provenance: 'artifact-1' }],
    }, 'pipeline')
    const fallback = createSceneFallback(model, 'WebGL unavailable')
    expect(fallback.querySelector('caption').textContent).toContain('pipeline nodes')
    expect(fallback.textContent).toContain('WebGL unavailable')
    expect(fallback.textContent).toContain('Capture')
    expect(fallback.textContent).toContain('12%')
    expect(fallback.querySelectorAll('th')).toHaveLength(5)
  })

  it('does not interpret labels as markup', () => {
    const fallback = createSceneFallback(normalizeSceneData({ nodes: [{ label: '<img src=x onerror=alert(1)>' }] }))
    expect(fallback.querySelector('img')).toBeNull()
    expect(fallback.textContent).toContain('<img')
  })
})

