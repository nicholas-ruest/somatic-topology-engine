import './styles.css'
import { createStore } from './state/store.js'
import { createApp } from './app.js'
import { createGatewayClient } from './gateway/client.js'

function sceneAdapter(container, options) {
  let mounted
  let cancelled = false
  import('./visualizations/TopologyScene.js').then(({ mountTopologyScene }) => {
    if (!cancelled) mounted = mountTopologyScene(container, options)
  }).catch(() => {
    if (!cancelled) container.innerHTML = '<p class="scene-fallback">Spatial enhancement unavailable. Tabular evidence remains authoritative.</p>'
  })
  return { destroy() { cancelled = true; mounted?.destroy() } }
}
const fixtureMode = import.meta.env.VITE_FIXTURE_MODE !== 'false'
const store = createStore({ fixtureMode })
const gateway = createGatewayClient({ baseUrl: import.meta.env.VITE_GATEWAY_BASE ?? '', fixtureMode, csrfToken:()=>document.querySelector('meta[name="csrf-token"]')?.content ?? '' })
createApp(document.querySelector('#app'), store, sceneAdapter, gateway)
