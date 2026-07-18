import { createHash } from 'node:crypto'
import { readdir, readFile, writeFile } from 'node:fs/promises'
import { relative, resolve, sep } from 'node:path'

const dist = resolve('dist')

async function filesUnder(directory) {
  const entries = await readdir(directory, { withFileTypes: true })
  const nested = await Promise.all(entries.map(async (entry) => {
    const path = resolve(directory, entry.name)
    return entry.isDirectory() ? filesUnder(path) : [path]
  }))
  return nested.flat()
}

const paths = (await filesUnder(dist))
  .filter((path) => !path.endsWith(`${sep}asset-manifest.json`))
  .sort()

const assets = await Promise.all(paths.map(async (path) => {
  const content = await readFile(path)
  return {
    path: relative(dist, path).split(sep).join('/'),
    sha256: createHash('sha256').update(content).digest('hex'),
    bytes: content.byteLength,
  }
}))

await writeFile(resolve(dist, 'asset-manifest.json'), `${JSON.stringify({ assets }, null, 2)}\n`)
console.log(JSON.stringify({ manifest: 'dist/asset-manifest.json', assets: assets.length }))
