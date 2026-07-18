import { describe, expect, it, vi } from 'vitest'
import { createWorkflowController, projectWorkflow, WorkflowClientError } from './workflows.js'

const projection=(extra={})=>({id:'wf-1',version:2,state:'ready',request:{workflow_type:'calibration',scope:'site/a'},progress:{kind:'percent',value:0},permitted_actions:['start','cancel'],blocking_reasons:[],...extra})

describe('authoritative workflow client state',()=>{
  it('projects bounded progress and indeterminate progress without inventing percentages',()=>{
    expect(projectWorkflow(projection({progress:{kind:'percent',value:140}})).progress).toBe(100)
    expect(projectWorkflow(projection({progress:{kind:'indeterminate'}}))).toMatchObject({progress:null,indeterminate:true})
    expect(()=>projectWorkflow(projection({state:'invented'}))).toThrow(WorkflowClientError)
  })
  it('starts, resumes, confirms, cancels, and passes the authoritative expected version',async()=>{
    let version=2; const gateway={startWorkflow:vi.fn(async()=>projection()),workflowAction:vi.fn(async(_id,action)=>projection({version:++version,state:action==='cancel'?'cancelled':action==='start'?'running':'ready'}))}
    const controller=createWorkflowController(gateway); await controller.start('calibration',{scope:'site/a'},'stable-key'); await controller.confirm('wf-1',{nonce:'server-nonce',typedScope:'site/a'}); await controller.resume('wf-1'); await controller.cancel('wf-1')
    expect(gateway.startWorkflow).toHaveBeenCalledWith('calibration',{scope:'site/a'},{idempotencyKey:'stable-key'})
    expect(gateway.workflowAction.mock.calls.map(call=>call[1])).toEqual(['confirm','start','cancel'])
    expect(gateway.workflowAction.mock.calls.map(call=>call[2].expectedVersion)).toEqual([2,3,4])
  })
  it('retains newer projections when delayed stream events arrive',async()=>{
    const gateway={workflow:vi.fn(async()=>projection({version:9,state:'running'}))}; const controller=createWorkflowController(gateway)
    await controller.load('wf-1'); expect(projectWorkflow(projection({version:8})).version).toBe(8)
    expect(controller.snapshot().workflows.get('wf-1').version).toBe(9)
  })
  it('surfaces version conflicts as actionable client errors',async()=>{
    const conflict=Object.assign(new Error('stale'),{code:'VERSION_CONFLICT',status:409}); const gateway={workflow:vi.fn(async()=>projection()),workflowAction:vi.fn(async()=>{throw conflict})}
    const controller=createWorkflowController(gateway); await controller.load('wf-1'); await expect(controller.cancel('wf-1')).rejects.toMatchObject({conflict:true,code:'VERSION_CONFLICT'})
    expect(controller.snapshot().errors.get('wf-1').conflict).toBe(true)
  })
  it('projects secret-free receipts and terminal state',()=>{const item=projectWorkflow(projection({state:'succeeded',receipt:{outcome:'succeeded',affected_resources:['model/a'],evidence_digests:['sha256:a'],audit_digest:'sha256:b',warnings:[],compensated:false}})); expect(item).toMatchObject({terminal:true,receipt:{outcome:'succeeded',auditDigest:'sha256:b'}}); expect(item.receipt).not.toHaveProperty('payload')})
})
