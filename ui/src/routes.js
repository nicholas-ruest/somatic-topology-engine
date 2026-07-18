export const workspaces = Object.freeze([
  ['overview','Live overview','pulse',['participant','operator','support','validation','security','release']],
  ['topology','Spatial signal topology','orbit',['operator','support','validation']],
  ['radio','Radio acquisition','antenna',['operator','support','validation']],
  ['signals','Signal observation','wave',['operator','support','validation']],
  ['physiology','Physiology estimation','lungs',['participant','operator','validation']],
  ['inference','State inference','nodes',['participant','operator','validation']],
  ['memory','Personalization memory','memory',['participant','operator']],
  ['device','Device interaction','device',['participant','operator','support']],
  ['consent','Consent & governance','shield',['participant','operator','security']],
  ['validation','Experiment validation','flask',['validation','release']],
  ['models','Models & capabilities','cube',['validation','support','release']],
  ['reliability','Observability & reliability','scope',['operator','support','security']],
  ['commissioning','Commissioning & site qualification','check',['operator','support','release']],
  ['operations','Operations & data lifecycle','terminal',['operator','support','security']],
  ['security','Security & incidents','lock',['security','support','release']],
  ['readiness','Release & commercial readiness','flag',['release','security','operator']],
].map(([id,label,icon,allowedRoles]) => Object.freeze({ id,label,icon,allowedRoles })))

const SAFE_SEGMENT=/^[a-z0-9][a-z0-9-]{0,63}$/
export function routeFromHash(hash = location.hash) {
  const raw=String(hash).replace(/^#\/?/,''); const [path,query='']=raw.split('?'); const [workspaceId,...segments]=path.split('/').filter(Boolean)
  const route=workspaces.find((item)=>item.id===workspaceId)??workspaces[0]
  const detail=segments.every(segment=>SAFE_SEGMENT.test(segment))?segments:[]; const params=new URLSearchParams(query)
  const tab=SAFE_SEGMENT.test(params.get('tab')??'')?params.get('tab'):null
  return Object.freeze({...route,detail:Object.freeze(detail),tab})
}
export function workbenchHref(workspaceId,{detail=[],tab=null}={}) { const route=workspaces.find(item=>item.id===workspaceId)
  if(!route||!detail.every(segment=>SAFE_SEGMENT.test(segment))||(tab&&!SAFE_SEGMENT.test(tab)))return '#/overview'
  return `#/${workspaceId}${detail.length?`/${detail.join('/')}`:''}${tab?`?tab=${encodeURIComponent(tab)}`:''}` }
export function canAccess(route, role) { return route.allowedRoles.includes(role) }
export function routesForRole(role) { return workspaces.filter((route) => canAccess(route, role)) }
export function menuRoutesForRole(role) { return routesForRole(role).filter((route) => route.id !== 'readiness') }
