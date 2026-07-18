import { createSceneFallback } from './fallback.js'
import { normalizeSceneData } from './scene-model.js'

function prefersReducedMotion() {
  return globalThis.matchMedia?.('(prefers-reduced-motion: reduce)').matches === true
}

function canUseWebGL() {
  try {
    const canvas = document.createElement('canvas')
    return Boolean(globalThis.WebGLRenderingContext && (canvas.getContext('webgl2') || canvas.getContext('webgl')))
  } catch { return false }
}

export function mountTopologyScene(container, options = {}) {
  if (!(container instanceof Element)) throw new TypeError('A DOM container is required')
  let destroyed = false; let renderer = null; let fallback = null
  const reducedMotion = options.reducedMotion ?? prefersReducedMotion()
  const lowPower = options.lowPower ?? globalThis.navigator?.connection?.saveData === true
  const render = async (data = options.data) => {
    if (destroyed) return
    renderer?.destroy(); fallback?.remove(); container.replaceChildren()
    const model = normalizeSceneData(data, options.kind, options.seed)
    fallback = createSceneFallback(model)
    fallback.hidden = canUseWebGL() && !options.disableWebGL
    container.append(fallback)
    if (fallback.hidden) {
      try {
        const { createThreeRenderer } = await import('./three-renderer.js')
        if (destroyed) return
        renderer = createThreeRenderer(container, model, { reducedMotion, lowPower })
        renderer.canvas.setAttribute('aria-hidden', 'true')
        renderer.canvas.addEventListener('webglcontextlost', (event) => {
          event.preventDefault(); renderer?.destroy(); renderer = null
          fallback.hidden = false
          fallback.querySelector('.viz-fallback__title').focus?.()
        }, { once: true })
      } catch {
        fallback.hidden = false
      }
    }
  }
  container.setAttribute('role', 'region')
  container.setAttribute('aria-label', options.ariaLabel || 'Interactive system topology with accessible fallback')
  void render()
  return {
    update(data) { return render(data) },
    pause() { renderer?.destroy(); renderer = null; if (fallback) fallback.hidden = false },
    destroy() { destroyed = true; renderer?.destroy(); fallback?.remove(); renderer = null; fallback = null; container.replaceChildren() },
  }
}

export default mountTopologyScene

