import { describe, expect, it, vi } from 'vitest'
import { createStore, validateEnvelope } from './store.js'

describe('policy-safe client store', () => {
  it('accepts known roles and rejects invented roles', () => { const s=createStore(); s.dispatch({type:'role',value:'security'}); expect(s.getState().role).toBe('security'); s.dispatch({type:'role',value:'admin'}); expect(s.getState().role).toBe('security') })
  it('notifies subscribers and tracks receipts ephemerally', () => { const s=createStore(); const f=vi.fn(); s.subscribe(f); s.dispatch({type:'receipt',id:'safe-id',status:'pending'}); expect(f).toHaveBeenCalledOnce(); expect(s.getState().pending['safe-id']).toBe('pending') })
  it('fails closed on schema mismatch, malformed sequence, and stale data', () => { expect(validateEnvelope({schemaVersion:'2.0.0',sequence:1})).toEqual({valid:false,reason:'INCOMPATIBLE'}); expect(validateEnvelope({schemaVersion:'1.0.0',sequence:1,stale:true})).toEqual({valid:false,reason:'STALE'}); expect(validateEnvelope({schemaVersion:'1.0.0',sequence:1,stale:false})).toEqual({valid:true}) })
})
