import { describe, expect, it } from 'vitest'
import { workspaces } from '../routes.js'
import { renderWorkspace } from './workspaces.js'

describe('workspace projections', () => {
  it('renders all routes without unsupported claims', () => { const all=workspaces.map(renderWorkspace).join(' '); expect(all).toContain('NOT_APPROVED'); expect(all).not.toMatch(/cardiac coherence|cognitive load score/i) })
  it('makes experimental and abstention boundaries explicit', () => { expect(renderWorkspace(workspaces.find(x=>x.id==='physiology'))).toContain('ABSTAINED'); expect(renderWorkspace(workspaces.find(x=>x.id==='inference'))).toContain('NO CLAIM') })
  it('escapes fixture-derived primitive content', async () => { const { metric }=await import('./primitives.js'); expect(metric('<img src=x>','ok')).not.toContain('<img') })
})
