const STATES = new Set(['pending','ready','running','awaiting_human','awaiting_device','retry_wait','compensating','blocked','cancelled','succeeded','failed'])
const TERMINAL = new Set(['cancelled','succeeded','failed'])

export class WorkflowClientError extends Error {
  constructor(code, message, { retryable = false, conflict = false } = {}) {
    super(message); this.name = 'WorkflowClientError'; this.code = code; this.retryable = retryable; this.conflict = conflict
  }
}

export function projectWorkflow(payload) {
  const value = payload?.data?.workflow ?? payload?.workflow ?? payload?.data ?? payload
  if (!value || typeof value.id !== 'string' || !Number.isSafeInteger(value.version) || value.version < 1) throw new WorkflowClientError('INVALID_WORKFLOW', 'Workflow projection is malformed')
  const state = String(value.state ?? '').toLowerCase()
  if (!STATES.has(state)) throw new WorkflowClientError('INVALID_WORKFLOW_STATE', 'Workflow state is unsupported')
  const progress = value.progress?.kind === 'percent' ? Math.max(0, Math.min(100, Number(value.progress.value))) : null
  if (progress !== null && !Number.isFinite(progress)) throw new WorkflowClientError('INVALID_PROGRESS', 'Workflow progress is malformed')
  const receipt = value.receipt ? Object.freeze({
    outcome: String(value.receipt.outcome), affectedResources: [...(value.receipt.affected_resources ?? [])],
    evidenceDigests: [...(value.receipt.evidence_digests ?? [])], warnings: [...(value.receipt.warnings ?? [])],
    auditDigest: String(value.receipt.audit_digest ?? ''), recoveryGuidance: value.receipt.recovery_guidance ?? null,
    compensated: Boolean(value.receipt.compensated),
  }) : null
  return Object.freeze({ id:value.id, version:value.version, type:value.request?.workflow_type ?? value.workflow_type,
    scope:value.request?.scope ?? value.scope, state, progress, indeterminate:progress === null,
    permittedActions:Object.freeze([...(value.permitted_actions ?? [])]), blockingReasons:Object.freeze([...(value.blocking_reasons ?? [])]),
    challenge:value.challenge ? Object.freeze({...value.challenge}) : null, receipt, terminal:TERMINAL.has(state) })
}

export function createWorkflowController(gateway) {
  const workflows = new Map(); const errors = new Map(); const listeners = new Set(); const streams = new Map()
  const notify = () => listeners.forEach(listener => listener(snapshot()))
  const snapshot = () => Object.freeze({ workflows:new Map(workflows), errors:new Map(errors) })
  const accept = payload => { const workflow=projectWorkflow(payload); const prior=workflows.get(workflow.id)
    if (prior && workflow.version <= prior.version) return prior
    workflows.set(workflow.id,workflow); errors.delete(workflow.id); notify(); return workflow }
  const fail = (id,error) => { const normalized = new WorkflowClientError(error.code ?? 'WORKFLOW_REJECTED', error.message ?? 'Workflow request rejected',
    {retryable:Boolean(error.retryable),conflict:error.status === 409 || /CONFLICT|VERSION/.test(error.code ?? '')}); errors.set(id,normalized); notify(); throw normalized }
  const command = async (id, action, body = {}) => { const current=workflows.get(id); if(!current) throw new WorkflowClientError('WORKFLOW_NOT_LOADED','Load the workflow before acting')
    if(current.terminal) throw new WorkflowClientError('WORKFLOW_TERMINAL','Terminal workflows cannot accept actions')
    try { return accept(await gateway.workflowAction(id,action,{...body,expectedVersion:current.version},{idempotencyKey:crypto.randomUUID()})) } catch(error) { return fail(id,error) } }
  return Object.freeze({
    subscribe(listener) { listeners.add(listener); return () => listeners.delete(listener) }, snapshot,
    async start(type,body,idempotencyKey=crypto.randomUUID()) { try { return accept(await gateway.startWorkflow(type,body,{idempotencyKey})) } catch(error) { return fail(`new:${type}`,error) } },
    async load(id) { try { return accept(await gateway.workflow(id)) } catch(error) { return fail(id,error) } },
    confirm(id,{nonce,typedScope}) { return command(id,'confirm',{nonce,typedScope}) },
    resume(id) { return command(id,'start') }, cancel(id) { return command(id,'cancel') }, retry(id) { return command(id,'retry') },
    watch(id) { streams.get(id)?.close(); const stream=gateway.workflowStream(id,payload=>{ try { accept(payload) } catch(error) { errors.set(id,error); notify() } },state=>{ if(state.state==='unavailable'){ errors.set(id,new WorkflowClientError(state.reason,'Workflow progress stream unavailable',{retryable:true})); notify() } }); streams.set(id,stream); return { close(){stream.close();streams.delete(id)} } },
    close() { streams.forEach(stream=>stream.close()); streams.clear(); listeners.clear() },
  })
}
