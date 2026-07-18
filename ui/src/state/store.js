const ALLOWED_SCHEMA = '1.0.0'
const roles = ['participant', 'operator', 'support', 'validation', 'security', 'release']

export function createStore(initial = {}) {
  let state = Object.freeze({ role: 'operator', connected: true, fixtureMode: true, density: 'comfortable', pending: {}, ...initial })
  const subscribers = new Set()
  return Object.freeze({
    getState: () => state,
    subscribe(fn) { subscribers.add(fn); return () => subscribers.delete(fn) },
    dispatch(action) {
      if (action.type === 'role' && roles.includes(action.value)) state = Object.freeze({ ...state, role: action.value })
      if (action.type === 'connection') state = Object.freeze({ ...state, connected: Boolean(action.value) })
      if (action.type === 'receipt') state = Object.freeze({ ...state, pending: { ...state.pending, [action.id]: action.status } })
      subscribers.forEach((fn) => fn(state))
    },
  })
}

export function validateEnvelope(model) {
  if (!model || model.schemaVersion !== ALLOWED_SCHEMA || !Number.isSafeInteger(model.sequence)) return { valid: false, reason: 'INCOMPATIBLE' }
  if (model.stale) return { valid: false, reason: 'STALE' }
  return { valid: true }
}
