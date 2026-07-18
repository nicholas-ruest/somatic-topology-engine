import { describe, expect, it, vi } from 'vitest'
import { createGatewayClient, GatewayError } from './client.js'

const model = (extra={}) => ({schemaVersion:'1.0.0',sequence:1,stale:false,...extra})
describe('versioned gateway client', () => {
  it('never sends mutations in fixture mode', async () => { const fetchImpl=vi.fn(); const client=createGatewayClient({fixtureMode:true,fetchImpl}); await expect(client.command('reset',{}, {idempotencyKey:'k'})).rejects.toMatchObject({code:'FIXTURE_READ_ONLY'}); expect(fetchImpl).not.toHaveBeenCalled() })
  it('fails closed when disconnected, stale, or schema incompatible', async () => {
    const disconnected=createGatewayClient({fetchImpl:()=>Promise.reject(new TypeError('offline'))})
    await expect(disconnected.read('/api/v1/status')).rejects.toMatchObject({code:'DISCONNECTED'})
    for(const bad of [model({stale:true}),model({schemaVersion:'2.0.0'})]) { const c=createGatewayClient({fetchImpl:async()=>({ok:true,status:200,json:async()=>bad})}); await expect(c.read('/api/v1/status')).rejects.toBeInstanceOf(GatewayError) }
  })
  it('sends bounded credentials, CSRF, and idempotency metadata', async () => { const fetchImpl=vi.fn(async(_url,init)=>({ok:true,status:200,json:async()=>model()})); const c=createGatewayClient({fetchImpl,csrfToken:()=> 'csrf'}); await c.command('doctor',{}, {idempotencyKey:'cmd-1'}); expect(fetchImpl.mock.calls[0][1]).toMatchObject({credentials:'same-origin',method:'POST'}); expect(fetchImpl.mock.calls[0][1].headers).toMatchObject({'X-CSRF-Token':'csrf','Idempotency-Key':'cmd-1'}) })
  it('requires idempotency before transport', async () => { const fetchImpl=vi.fn(); const c=createGatewayClient({fetchImpl}); await expect(c.command('reset',{})).rejects.toMatchObject({code:'IDEMPOTENCY_REQUIRED'}); expect(fetchImpl).not.toHaveBeenCalled() })
  it('rejects cross-origin configuration', () => { expect(()=>createGatewayClient({baseUrl:'https://outside.invalid'})).toThrowError(/same-origin/) })
  it('binds workflow actions to identity, expected version, and idempotency',async()=>{const fetchImpl=vi.fn(async()=>({ok:true,status:200,json:async()=>model()}));const c=createGatewayClient({fetchImpl,csrfToken:()=> 'csrf'});await c.workflowAction('wf/1','cancel',{expectedVersion:7},{idempotencyKey:'action-1'});expect(fetchImpl.mock.calls[0][0]).toBe('/api/v1/commands/workflow.cancel');expect(JSON.parse(fetchImpl.mock.calls[0][1].body)).toEqual({workflowId:'wf/1',expectedVersion:7});expect(fetchImpl.mock.calls[0][1].headers['Idempotency-Key']).toBe('action-1')})
})
