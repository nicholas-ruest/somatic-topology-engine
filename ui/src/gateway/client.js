import { validateEnvelope } from '../state/store.js'

const MAX_BODY_BYTES = 64 * 1024
const MAX_STREAM_BUFFER = 64

export class GatewayError extends Error {
  constructor(code, message, { status = 0, retryable = false } = {}) {
    super(message); this.name = 'GatewayError'; this.code = code; this.status = status; this.retryable = retryable
  }
}

export function createGatewayClient({ baseUrl = '', fetchImpl = globalThis.fetch, eventSourceFactory, csrfToken = () => '', fixtureMode = false, timeoutMs = 8_000 } = {}) {
  if (baseUrl && new URL(baseUrl, location.origin).origin !== location.origin) throw new GatewayError('ORIGIN_REJECTED', 'Gateway must be same-origin')

  async function request(path, { method = 'GET', body, idempotencyKey, signal } = {}) {
    if (!fetchImpl) throw new GatewayError('DISCONNECTED', 'Gateway transport unavailable', { retryable: true })
    const mutation = method !== 'GET' && method !== 'HEAD'
    if (fixtureMode && mutation) throw new GatewayError('FIXTURE_READ_ONLY', 'Fixture mode cannot execute production mutations')
    if (mutation && !idempotencyKey) throw new GatewayError('IDEMPOTENCY_REQUIRED', 'Mutation requires an idempotency key')
    const serialized = body === undefined ? undefined : JSON.stringify(body)
    if (serialized && new TextEncoder().encode(serialized).byteLength > MAX_BODY_BYTES) throw new GatewayError('BODY_TOO_LARGE', 'Command body exceeds the bounded gateway contract')
    const timeout = AbortSignal.timeout(timeoutMs)
    const combined = signal ? AbortSignal.any([signal, timeout]) : timeout
    let response
    try {
      response = await fetchImpl(`${baseUrl}${path}`, { method, body: serialized, signal: combined, credentials: 'same-origin', headers: { 'Accept':'application/json', ...(serialized?{'Content-Type':'application/json'}:{}), ...(mutation?{'X-CSRF-Token':csrfToken(),'Idempotency-Key':idempotencyKey}:{}) } })
    } catch (error) { throw new GatewayError(error.name === 'TimeoutError' ? 'TIMEOUT' : 'DISCONNECTED', 'Gateway request unavailable', { retryable: true }) }
    let payload
    try { payload = await response.json() } catch { throw new GatewayError('INVALID_RESPONSE', 'Gateway returned an invalid response', { status: response.status }) }
    if (!response.ok) throw new GatewayError(payload.code ?? 'COMMAND_REJECTED', payload.message ?? 'Command rejected', { status: response.status, retryable: response.status >= 500 })
    const validation = validateEnvelope(payload)
    if (!validation.valid) throw new GatewayError(validation.reason, `Read model rejected: ${validation.reason}`)
    return payload
  }

  function stream(path, onModel, onState = () => {}) {
    if (fixtureMode || !eventSourceFactory) { onState({ state:'unavailable', reason:fixtureMode?'FIXTURE_READ_ONLY':'DISCONNECTED' }); return { close() {} } }
    let lastSequence = -1; const buffer = []
    const source = eventSourceFactory(`${baseUrl}${path}`, { withCredentials:true })
    source.onmessage = (event) => {
      let model
      try { model=JSON.parse(event.data) } catch { onState({state:'unavailable',reason:'INVALID_RESPONSE'}); return }
      const valid=validateEnvelope(model)
      if (!valid.valid || model.sequence <= lastSequence) { onState({state:'unavailable',reason:valid.reason ?? 'SEQUENCE_REJECTED'}); return }
      lastSequence=model.sequence; buffer.push(model); if(buffer.length>MAX_STREAM_BUFFER) buffer.shift(); onModel(model)
    }
    source.onerror = () => onState({state:'unavailable',reason:'DISCONNECTED'})
    return { close:()=>source.close(), snapshot:()=>buffer.slice() }
  }

  return Object.freeze({
    read: (path, options) => request(path, { ...options, method:'GET' }),
    query: (query, options={}) => request('/api/v1/query', { method:'POST', body:query, idempotencyKey:options.idempotencyKey ?? crypto.randomUUID(), signal:options.signal }),
    command: (name, body, options={}) => request(`/api/v1/commands/${encodeURIComponent(name)}`, { method:'POST', body, idempotencyKey:options.idempotencyKey, signal:options.signal }),
    workflow: (id, options) => request(`/api/v1/workflows/${encodeURIComponent(id)}`, { ...options, method:'GET' }),
    startWorkflow: (type, body, options={}) => request(`/api/v1/commands/workflow.${encodeURIComponent(type)}`, { method:'POST', body, idempotencyKey:options.idempotencyKey ?? crypto.randomUUID(), signal:options.signal }),
    workflowAction: (id, action, body, options={}) => request(`/api/v1/commands/workflow.${encodeURIComponent(action)}`, { method:'POST', body:{workflowId:id,...body}, idempotencyKey:options.idempotencyKey ?? crypto.randomUUID(), signal:options.signal }),
    workflowStream: (id, onModel, onState) => stream(`/api/v1/workflows/${encodeURIComponent(id)}/stream`, onModel, onState),
    stream,
  })
}
