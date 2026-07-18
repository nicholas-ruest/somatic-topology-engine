import { describe, expect, it } from 'vitest'
import { workspaces, routesForRole, canAccess, routeFromHash, workbenchHref } from './routes.js'

describe('role-scoped routes', () => {
  it('defines every ADR-057 functional workspace exactly once', () => { expect(workspaces).toHaveLength(16); expect(new Set(workspaces.map(x=>x.id)).size).toBe(16) })
  it('keeps validation and security areas role-bound', () => { expect(routesForRole('participant').map(x=>x.id)).not.toContain('security'); expect(canAccess(workspaces.find(x=>x.id==='validation'),'operator')).toBe(false) })
  it('always exposes a safe overview', () => { for(const role of ['participant','operator','support','validation','security','release']) expect(routesForRole(role)[0].id).toBe('overview') })
  it('bounds deep links to safe identifiers and tabs',()=>{expect(routeFromHash('#/radio/capture-12?tab=evidence')).toMatchObject({id:'radio',detail:['capture-12'],tab:'evidence'});expect(routeFromHash('#/radio/%3Cscript%3E')).toMatchObject({id:'radio',detail:[]});expect(workbenchHref('radio',{detail:['capture-12'],tab:'evidence'})).toBe('#/radio/capture-12?tab=evidence');expect(workbenchHref('radio',{detail:['../secret']})).toBe('#/overview')})
})
