import * as THREE from 'three'
import { RESOURCE_BUDGET,seededRandom } from './scene-model.js'

const COLOR={healthy:0x3fd67a,warning:0xffca62,blocked:0xff5e7c,experimental:0x4ea1ff,unknown:0x5b6673}

export function createThreeRenderer(container,model,options={}){
  const scene=new THREE.Scene();scene.background=new THREE.Color(0x0a0e13);scene.fog=new THREE.FogExp2(0x0a0e13,.045)
  const camera=new THREE.PerspectiveCamera(42,1,.1,100);camera.position.set(0,4.4,16);camera.lookAt(0,0,0)
  const renderer=new THREE.WebGLRenderer({antialias:!options.lowPower,alpha:false,powerPreference:options.lowPower?'low-power':'high-performance'})
  renderer.setPixelRatio(Math.min(globalThis.devicePixelRatio||1,RESOURCE_BUDGET.pixelRatio));renderer.outputColorSpace=THREE.SRGBColorSpace;container.append(renderer.domElement)
  const disposables=[];const track=(...items)=>(disposables.push(...items),items[0])

  const grid=new THREE.GridHelper(24,24,0x1f5d3c,0x14261d);track(grid.geometry,grid.material);grid.position.y=-3.3;grid.material.transparent=true;grid.material.opacity=.22;scene.add(grid)
  const innerGrid=new THREE.GridHelper(12,12,0x3fd67a,0x1c4a30);track(innerGrid.geometry,innerGrid.material);innerGrid.position.y=-3.28;innerGrid.material.transparent=true;innerGrid.material.opacity=.12;scene.add(innerGrid)

  const nodeGeo=track(new THREE.IcosahedronGeometry(options.lowPower ? .14 : .24,options.lowPower?0:2));const nodeMat=track(new THREE.MeshBasicMaterial({vertexColors:true,transparent:true,opacity:.95}))
  const nodes=new THREE.InstancedMesh(nodeGeo,nodeMat,RESOURCE_BUDGET.nodes);nodes.count=model.nodes.length
  const haloGeo=track(new THREE.TorusGeometry(.36,.012,4,32));const haloMat=track(new THREE.MeshBasicMaterial({color:0x4ea1ff,transparent:true,opacity:.28,blending:THREE.AdditiveBlending,depthWrite:false}));const halos=new THREE.InstancedMesh(haloGeo,haloMat,RESOURCE_BUDGET.nodes);halos.count=model.nodes.length
  const matrix=new THREE.Matrix4(),rotation=new THREE.Matrix4().makeRotationX(Math.PI/2)
  model.nodes.forEach((node,index)=>{const scale=.75+(1-node.uncertainty)*1.2;matrix.makeScale(scale,scale,scale).setPosition(node.x,node.y,node.z);nodes.setMatrixAt(index,matrix);nodes.setColorAt(index,new THREE.Color(COLOR[node.status]));matrix.copy(rotation).setPosition(node.x,node.y,node.z);halos.setMatrixAt(index,matrix)})
  nodes.instanceMatrix.needsUpdate=true;halos.instanceMatrix.needsUpdate=true;scene.add(nodes,halos)

  const byId=new Map(model.nodes.map(node=>[node.id,node]));const edgePoints=[];const edgePairs=[]
  for(const edge of model.edges){const from=byId.get(edge.from),to=byId.get(edge.to);edgePoints.push(from.x,from.y,from.z,to.x,to.y,to.z);edgePairs.push([from,to])}
  const edgeGeo=track(new THREE.BufferGeometry());edgeGeo.setAttribute('position',new THREE.Float32BufferAttribute(edgePoints,3));const edgeMat=track(new THREE.LineBasicMaterial({color:0x4ea1ff,transparent:true,opacity:.34,blending:THREE.AdditiveBlending}));scene.add(new THREE.LineSegments(edgeGeo,edgeMat))
  const glowMat=track(new THREE.LineBasicMaterial({color:0x3fd67a,transparent:true,opacity:.09,blending:THREE.AdditiveBlending}));const glow=new THREE.LineSegments(edgeGeo,glowMat);glow.scale.setScalar(1.006);scene.add(glow)

  const pulseGeo=track(new THREE.SphereGeometry(.055,6,6));const pulseMat=track(new THREE.MeshBasicMaterial({color:0xbdf0d3,transparent:true,opacity:.9,blending:THREE.AdditiveBlending,depthWrite:false}));const pulses=new THREE.InstancedMesh(pulseGeo,pulseMat,Math.max(1,Math.min(edgePairs.length,RESOURCE_BUDGET.edges)));pulses.count=edgePairs.length;scene.add(pulses)

  const random=seededRandom(`renderer:${model.kind}:${model.provenance}`),dustCount=options.lowPower?80:220,dustPositions=new Float32Array(dustCount*3);for(let i=0;i<dustCount;i++){dustPositions[i*3]=(random()-.5)*18;dustPositions[i*3+1]=(random()-.5)*10;dustPositions[i*3+2]=(random()-.5)*12}
  const dustGeo=track(new THREE.BufferGeometry());dustGeo.setAttribute('position',new THREE.BufferAttribute(dustPositions,3));const dustMat=track(new THREE.PointsMaterial({color:0x4ea1ff,size:.025,transparent:true,opacity:.3,blending:THREE.AdditiveBlending,depthWrite:false}));const dust=new THREE.Points(dustGeo,dustMat);scene.add(dust)

  const coreGeo=track(new THREE.TorusKnotGeometry(.8,.018,80,8,2,5));const coreMat=track(new THREE.MeshBasicMaterial({color:model.kind==='readiness'?0xff5e78:0x3fd67a,transparent:true,opacity:.18,blending:THREE.AdditiveBlending,wireframe:true}));const core=new THREE.Mesh(coreGeo,coreMat);scene.add(core)

  let frame=0,stopped=false,last=0
  const resize=()=>{const width=Math.max(container.clientWidth,320),height=Math.max(container.clientHeight,260);renderer.setSize(width,height,false);camera.aspect=width/height;camera.updateProjectionMatrix()}
  const draw=(time=0)=>{if(stopped)return;if(time-last>=1000/RESOURCE_BUDGET.fps){if(!options.reducedMotion&&!options.lowPower){const t=time*.0001;nodes.rotation.y=Math.sin(t*.32)*.13;halos.rotation.y=nodes.rotation.y;dust.rotation.y=t*.08;core.rotation.x=t*.18;core.rotation.y=t*.28;edgePairs.forEach(([from,to],i)=>{const p=(t*.65+i/Math.max(1,edgePairs.length))%1;matrix.makeTranslation(from.x+(to.x-from.x)*p,from.y+(to.y-from.y)*p,from.z+(to.z-from.z)*p);pulses.setMatrixAt(i,matrix)});pulses.instanceMatrix.needsUpdate=true}renderer.render(scene,camera);last=time}frame=requestAnimationFrame(draw)}
  resize();draw();globalThis.addEventListener?.('resize',resize)
  return{canvas:renderer.domElement,renderOnce:()=>renderer.render(scene,camera),destroy(){stopped=true;cancelAnimationFrame(frame);globalThis.removeEventListener?.('resize',resize);for(const item of disposables)item.dispose?.();renderer.dispose();renderer.forceContextLoss?.();renderer.domElement.remove()}}
}
