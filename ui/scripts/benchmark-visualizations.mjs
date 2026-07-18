import { performance } from 'node:perf_hooks'
import { normalizeSceneData, RESOURCE_BUDGET } from '../src/visualizations/scene-model.js'

const nodes = Array.from({ length: RESOURCE_BUDGET.nodes }, (_, index) => ({ id: `node-${index}`, uncertainty: index / 100 }))
const edges = Array.from({ length: RESOURCE_BUDGET.edges }, (_, index) => ({ from: `node-${index % 96}`, to: `node-${(index + 1) % 96}` }))
const iterations = 5_000
const start = performance.now()
for (let index = 0; index < iterations; index += 1) normalizeSceneData({ nodes, edges }, 'pipeline', `bench-${index}`)
const elapsed = performance.now() - start
const perModel = elapsed / iterations
if (perModel > 5) throw new Error(`Scene-model budget exceeded: ${perModel.toFixed(3)}ms > 5ms`)
console.log(JSON.stringify({ iterations, elapsedMs: Number(elapsed.toFixed(2)), meanMs: Number(perModel.toFixed(4)), budgetMs: 5 }))

