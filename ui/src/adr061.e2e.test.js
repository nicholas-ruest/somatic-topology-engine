/** @vitest-environment jsdom */
import {describe,expect,it,vi} from 'vitest'
import {createApp} from './app.js'
import {createStore,validateEnvelope} from './state/store.js'
import {canAccess,menuRoutesForRole,routeFromHash,workspaces} from './routes.js'
import {renderWorkspace} from './components/workspaces.js'
import {destructivePreview,drawer,schemaForm,statePanel} from './components/interaction.js'
import {createWorkflowController,projectWorkflow} from './state/workflows.js'
import {createGatewayClient} from './gateway/client.js'

const roles=['participant','operator','support','validation','security','release']
const states=['pending','ready','running','awaiting_human','awaiting_device','retry_wait','compensating','blocked','cancelled','succeeded','failed']
const envelope=(extra={})=>({schemaVersion:'1.0.0',sequence:1,stale:false,...extra})
const workflow=(state,extra={})=>({id:'wf-1',version:2,state,request:{workflow_type:'hardware_probe',scope:'site/a'},progress:{kind:'indeterminate'},permitted_actions:[],blocking_reasons:[],...extra})

describe('ADR-061 role and workbench scenarios',()=>{
  it('renders every authorized workbench for every role without leaking forbidden navigation',()=>{
    for(const role of roles){const visible=menuRoutesForRole(role);expect(visible.length).toBeGreaterThan(0);for(const route of workspaces){const allowed=canAccess(route,role);expect(visible.some(item=>item.id===route.id)).toBe(allowed&&route.id!=='readiness');if(allowed){const html=renderWorkspace(route);expect(html).toContain('workbench-tabs');expect(html).toContain('data-print-preserve="true"')}}}
  })
  it('redirects a forbidden deep link to an authorized workspace and omits its content',()=>{
    location.hash='#/security/incident-1?tab=evidence';const root=document.createElement('div');document.body.append(root);const app=createApp(root,createStore({role:'participant'}));expect(root.textContent).toContain('Route unavailable for this role');expect(root.textContent).not.toContain('Security event timeline');expect(root.querySelector('a[href="#/security"]')).toBeNull();app.destroy()
  })
  it('projects every Rust workflow state without converting it into browser authority',()=>{for(const state of states){const item=projectWorkflow(workflow(state));expect(item.state).toBe(state);expect(item.terminal).toBe(['cancelled','succeeded','failed'].includes(state))}})
})

describe('ADR-061 hostile and concurrency scenarios',()=>{
  it('escapes hostile values in forms, dialogs, drawers, and workspace deep links',()=>{const hostile='<img src=x onerror=alert(1)>';for(const html of [schemaForm({id:'x',title:hostile,version:1,fields:[{name:'x',label:hostile}]}),destructivePreview({action:hostile,scope:hostile,consequences:[hostile],version:1}),drawer({id:'x',title:hostile,body:'safe'})])expect(html).not.toContain('<img');expect(routeFromHash('#/radio/%3Cimg%3E').detail).toEqual([])})
  it('fails stale envelopes closed',()=>{expect(validateEnvelope(envelope({stale:true}))).toEqual({valid:false,reason:'STALE'});expect(validateEnvelope(envelope({schemaVersion:'99.0.0'}))).toEqual({valid:false,reason:'INCOMPATIBLE'})})
  it('sends an exact duplicate idempotency key unchanged for authoritative deduplication',async()=>{const fetchImpl=vi.fn(async()=>({ok:true,status:200,json:async()=>envelope()}));const client=createGatewayClient({fetchImpl,csrfToken:()=> 'csrf'});await client.command('probe',{}, {idempotencyKey:'same'});await client.command('probe',{}, {idempotencyKey:'same'});expect(fetchImpl.mock.calls.map(call=>call[1].headers['Idempotency-Key'])).toEqual(['same','same'])})
  it('keeps a conflict visible and prevents acting on an unloaded or terminal workflow',async()=>{const conflict=Object.assign(new Error('stale'),{code:'VERSION_CONFLICT',status:409});const gateway={workflow:vi.fn(async()=>workflow('ready')),workflowAction:vi.fn(async()=>{throw conflict})};const controller=createWorkflowController(gateway);await expect(controller.cancel('missing')).rejects.toMatchObject({code:'WORKFLOW_NOT_LOADED'});await controller.load('wf-1');await expect(controller.cancel('wf-1')).rejects.toMatchObject({conflict:true});expect(controller.snapshot().errors.has('wf-1')).toBe(true);const terminal=createWorkflowController({workflow:async()=>workflow('succeeded'),workflowAction:vi.fn()});await terminal.load('wf-1');await expect(terminal.resume('wf-1')).rejects.toMatchObject({code:'WORKFLOW_TERMINAL'})})
  it('never enables destructive submission without a server challenge',()=>{document.body.innerHTML=destructivePreview({action:'Reset',scope:'device/a',consequences:['Erase'],version:7});const dialog=document.querySelector('dialog');expect(dialog.getAttribute('aria-labelledby')).toBe('destructive-title');expect(dialog.querySelector('[value="confirm"]').disabled).toBe(true);expect(dialog.querySelector('[value="cancel"]')).not.toBeNull()})
})

describe('ADR-061 keyboard and semantic scenarios',()=>{
  it('exposes focusable labelled navigation, dialogs, drawers, and retry states',()=>{document.body.innerHTML=`${drawer({id:'details',title:'Details',body:'<button>Inspect</button>'})}${destructivePreview({action:'Delete',scope:'participant/a',consequences:[],version:2,challenge:{nonce:'n'}})}${statePanel('conflict',{title:'Conflict',retry:true})}`;expect(document.querySelector('[role="complementary"][aria-labelledby="details-title"]')).not.toBeNull();expect(document.querySelector('[aria-label="Close details"]')).not.toBeNull();expect(document.querySelector('dialog form[method="dialog"]')).not.toBeNull();expect(document.querySelector('[data-view-action="retry"]')).not.toBeNull()})
})
