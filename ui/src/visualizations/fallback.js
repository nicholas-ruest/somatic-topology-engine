function element(name, className, text) {
  const node = document.createElement(name)
  if (className) node.className = className
  if (text !== undefined) node.textContent = text
  return node
}

export function createSceneFallback(model, reason = '') {
  const section = element('section', 'viz-fallback')
  section.dataset.vizFallback = 'true'
  const heading = element('h3', 'viz-fallback__title', `${model.label} — accessible data view`)
  section.append(heading)
  if (reason) section.append(element('p', 'viz-fallback__reason', reason))
  section.append(element('p', 'viz-fallback__meta',
    `Provenance: ${model.provenance}. Time: ${model.timestamp}. Units: ${model.units}. ${model.experimental ? 'Experimental or illustrative.' : 'Approved read model.'}`))
  const table = element('table', 'viz-fallback__table')
  table.createCaption().textContent = `${model.kind} nodes, uncertainty, scope, and provenance`
  const header = table.createTHead().insertRow()
  for (const title of ['Node', 'Status', 'Scope', 'Uncertainty', 'Provenance']) {
    const cell = document.createElement('th')
    cell.scope = 'col'
    cell.textContent = title
    header.append(cell)
  }
  const body = table.createTBody()
  for (const node of model.nodes) {
    const row = body.insertRow()
    for (const value of [node.label, node.status, node.scope, `${Math.round(node.uncertainty * 100)}%`, node.provenance]) {
      row.insertCell().textContent = value
    }
  }
  if (!model.nodes.length) section.append(element('p', 'viz-fallback__empty', 'No approved visualization data is available.'))
  section.append(table)
  return section
}

