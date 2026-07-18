import { describe, expect, it, vi } from 'vitest'
import { createApp } from './app.js'
import { createStore } from './state/store.js'

describe('application shell', () => {
  it('renders safety state, fixture boundary, and all operator routes', () => {
    location.hash = '#/overview'
    const root = document.createElement('div'); document.body.append(root)
    const app = createApp(root, createStore(), undefined)
    expect(root.textContent).toContain('NOT_APPROVED')
    expect(root.querySelector('.critical-strip')).toBeNull()
    expect(root.querySelector('footer .release-status').textContent).toContain('NOT_APPROVED')
    expect(root.textContent).toContain('FIXTURE MODE')
    expect(root.querySelectorAll('nav a')).toHaveLength(12)
    expect(root.querySelector('nav').textContent).not.toContain('Release & commercial readiness')
    app.destroy()
  })

  it('fails route access closed and disposes a mounted scene', () => {
    location.hash = '#/topology'
    const root = document.createElement('div'); document.body.append(root)
    const destroy = vi.fn()
    const app = createApp(root, createStore({ role: 'participant' }), () => ({ destroy }))
    expect(root.textContent).toContain('Route unavailable for this role')
    expect(root.textContent).toContain('Live overview')
    app.destroy()
    expect(destroy).not.toHaveBeenCalled()
  })

  it('allows role changes only through store policy', () => {
    const root = document.createElement('div'); document.body.append(root)
    const store = createStore(); const app = createApp(root, store)
    const select = root.querySelector('#role-select'); select.value = 'security'; select.dispatchEvent(new Event('change'))
    expect(store.getState().role).toBe('security')
    expect(root.querySelector('nav').textContent).toContain('Security & incidents')
    app.destroy()
  })
  it('disables mutating commands in deterministic fixture mode', () => {
    location.hash='#/operations'; const root=document.createElement('div'); document.body.append(root)
    const gateway={command:vi.fn()}; const app=createApp(root,createStore({fixtureMode:true}),undefined,gateway)
    const doctor=[...root.querySelectorAll('button')].find(x=>x.textContent.includes('Run doctor'))
    expect(doctor.disabled).toBe(true); doctor.click(); expect(gateway.command).not.toHaveBeenCalled(); app.destroy()
  })
})
