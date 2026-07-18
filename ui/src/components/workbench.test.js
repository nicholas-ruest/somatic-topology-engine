import {describe,it,expect} from 'vitest'
import {liveChart,timeline,dataTable,workflowCard,inspector,wizard} from './workbench.js'

describe('workbench primitives',()=>{
  it('renders bounded accessible charts and timelines',()=>{const html=liveChart({label:'Packet quality',values:Array.from({length:200},(_,i)=>i)});expect(html).toContain('aria-label="Packet quality chart"');expect(html.match(/ L/g).length).toBeLessThanOrEqual(240);expect(timeline()).toContain('Timeline position')})
  it('escapes tables and evidence inspectors',()=>{expect(dataTable('x',['a'],[['<script>']])).not.toContain('<script>');expect(inspector('x',[['secret','&value']])).toContain('&amp;value')})
  it('renders durable workflow and wizard state without inventing authority',()=>{const html=workflowCard({id:'wf-1',title:'Reset',progress:null,actions:['confirm','cancel'],steps:[{label:'Confirm',state:'blocked'}]});expect(html).toContain('Durable Rust workflow');expect(html).toContain('indeterminate');expect(html).toContain('data-workflow-action="confirm"');expect(wizard('Commission',['Probe','Accept'])).toContain('Step 1 / 2')})
})
