import { routeFromHash, canAccess, routesForRole, menuRoutesForRole } from './routes.js'
import { renderWorkspace } from './components/workspaces.js'
import { fixtureEnvelope, notices } from './fixtures/read-models.js'
import { validateEnvelope } from './state/store.js'
import { escapeHtml } from './components/primitives.js'
import { createVisualizationFixture } from './visualizations/fixtures.js'
import { GatewayError } from './gateway/client.js'
import { createWorkflowController } from './state/workflows.js'
import { breadcrumbs } from './components/interaction.js'

const roleOptions = ['participant','operator','support','validation','security','release']

export function createApp(root, store, sceneMount, gateway) {
  let destroyScene
  const workflowController = gateway ? createWorkflowController(gateway) : null
  const render = () => {
    destroyScene?.(); destroyScene = undefined
    const state = store.getState(); const requested = routeFromHash(); const accessible = canAccess(requested, state.role)
    const route = accessible ? requested : routesForRole(state.role)[0]
    const envelope = validateEnvelope(fixtureEnvelope)
    root.innerHTML = `<div class="ambient-field" aria-hidden="true"><i></i><i></i><i></i></div><div class="app-grid">
      <aside class="sidebar"><div class="brand"><span class="brand-mark"><i></i><b>ST</b></span><div><strong>Somatic<br>Topology</strong><small>Edge intelligence system</small></div></div>
      <p class="nav-label">System surfaces</p><nav aria-label="Functional workspaces">${menuRoutesForRole(state.role).map((r,index) => `<a href="#/${r.id}" ${r.id===route.id?'aria-current="page"':''}><span aria-hidden="true">${String(index+1).padStart(2,'0')}</span><b>${escapeHtml(r.label)}</b><i aria-hidden="true">↗</i></a>`).join('')}</nav><div class="sidebar-foot"><span><i></i>Edge node online</span><small>STE / 0.1.0</small></div></aside>
      <div class="shell"><header class="topbar"><div><p class="eyebrow"><span></span>LOCAL EDGE / ${state.connected?'LIVE LINK':'LINK LOST'}</p><h1>${escapeHtml(route.label)}</h1><p class="route-meta">Policy-filtered · Sequence ${fixtureEnvelope.sequence} · ${fixtureEnvelope.provenance}</p></div><div class="top-actions"><label><span>Access profile</span><select id="role-select">${roleOptions.map((r)=>`<option ${r===state.role?'selected':''}>${r}</option>`).join('')}</select></label><span class="fixture-flag"><i></i>FIXTURE MODE</span></div></header>
      <main id="workspace" tabindex="-1">${breadcrumbs(route)}${!accessible?'<div class="notice notice-blocked"><strong>Route unavailable for this role.</strong> Showing the nearest authorized workspace.</div>':''}${!envelope.valid?`<div class="notice notice-blocked">Read model unavailable: ${envelope.reason}</div>`:''}${notices.slice(1).map(n=>`<div class="notice notice-${n.level}"><strong>${n.title}</strong> ${n.text}</div>`).join('')}<div class="workspace-grid">${renderWorkspace(route)}</div></main>
      <footer class="statusbar"><span><i class="live-dot"></i> Deterministic simulation</span><span>Schema ${fixtureEnvelope.schemaVersion}</span><span>No production mutations</span><a class="release-status" href="#/readiness" aria-label="Release status: not approved. View readiness evidence."><i aria-hidden="true"></i>NOT_APPROVED</a></footer></div></div>`
    root.querySelector('#role-select').addEventListener('change', (event) => store.dispatch({type:'role',value:event.target.value}))
    root.querySelectorAll('button:not([data-workflow-action])').forEach((button, index) => {
      const name = button.textContent.trim().toLowerCase().replace(/[^a-z0-9]+/g,'-')
      button.dataset.command = name
      const readOnly = /(^|-)(view|review|preview|open|verify|export|prepare|dry-run)(-|$)/.test(name)
      if (state.fixtureMode && !readOnly) { button.disabled=true; button.title='Unavailable: fixture mode cannot mutate production state' }
      button.addEventListener('click', async () => {
        if (readOnly) { store.dispatch({type:'receipt',id:`local-${index}`,status:'dry-run'}); return }
        if (!gateway) return
        button.disabled=true
        try { await gateway.command(name, {}, { idempotencyKey:crypto.randomUUID() }); store.dispatch({type:'receipt',id:name,status:'accepted'}) }
        catch (error) { const code=error instanceof GatewayError?error.code:'COMMAND_REJECTED'; store.dispatch({type:'receipt',id:name,status:`rejected:${code}`}) }
        finally { if(!state.fixtureMode) button.disabled=false }
      })
    })
    root.querySelectorAll('[data-workflow-action]').forEach((button) => {
      const id=button.closest('[data-workflow-id]')?.dataset.workflowId
      if (!id || state.fixtureMode || !workflowController) { button.disabled=true; button.title=state.fixtureMode?'Unavailable: fixture mode cannot mutate production workflows':'Workflow projection is not connected'; return }
      button.addEventListener('click',async()=>{button.disabled=true;try{await workflowController.load(id);if(button.dataset.workflowAction==='resume')await workflowController.resume(id);if(button.dataset.workflowAction==='cancel')await workflowController.cancel(id)}finally{button.disabled=false}})
    })
    const sceneSlot = root.querySelector('.scene-slot')
    if (sceneSlot && sceneMount) { const mounted = sceneMount(sceneSlot, { kind: sceneSlot.dataset.scene, data: createVisualizationFixture(sceneSlot.dataset.scene), seed: 'adr-057', reducedMotion: globalThis.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false }); destroyScene = mounted?.destroy }
  }
  const unsubscribe = store.subscribe(render)
  addEventListener('hashchange', render)
  render()
  return { render, destroy() { destroyScene?.(); workflowController?.close(); unsubscribe(); removeEventListener('hashchange', render); root.innerHTML='' } }
}
