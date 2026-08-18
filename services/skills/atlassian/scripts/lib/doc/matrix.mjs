/**
 * matrix.mjs — the approval matrix: one Page Properties Report on the FSD/ISD
 * parent page that aggregates every child document's header into a single table.
 *
 * How the two halves meet. A published FSD/ISD wraps its header table in the
 * native Content Properties (`details`) macro keyed by the doc type's
 * `propertiesId` (see templates/confluence/base.njk). Confluence then reads that
 * table as key-value pairs — column 1 is the property key, column 2 its value —
 * which is why an approval row is authored `role | status | name`. The report
 * macro (`detailssummary`) on the parent renders one row per child page and one
 * column per key it is told to show.
 *
 * The report will NOT discover the keys itself: `headings` must name them, in the
 * column order we want. So this module reads them back off the children — the
 * approver roles are whatever the documents actually carry, with no role list, no
 * client vocabulary, and no per-project config hard-coded anywhere.
 *
 * Scope comes from `cql`: the `<type>` label the publisher already stamps on every
 * typed page (applyDocLabels in publish.mjs) plus `ancestor = <parent>`, so a
 * matrix only ever aggregates the children of the page it sits on. Verified on
 * Confluence Cloud, including the negative case (a foreign ancestor yields "No
 * content found", i.e. the filter really applies).
 *
 * refreshDocMatrix runs at the end of every typed publish, so the parent's report
 * cannot drift from the documents and nobody has to remember a follow-up step. It
 * is also reachable manually as `confluence.mjs doc-matrix` for a dry run or to
 * repair a parent page by hand.
 */

import domino from '@mixmark-io/domino'

const norm = (s) => String(s == null ? '' : s).replace(/\s+/g, ' ').trim()

const isElement = (node, name) => node && node.tagName && node.tagName.toLowerCase() === name

// The `details` macro carrying this propertiesId, or null. Matching the id (not
// just the macro name) keeps an FSD page out of an ISD matrix and ignores any
// unrelated Content Properties macro an author added further down the page.
function findPropertiesMacro(doc, propertiesId) {
  const wanted = norm(propertiesId)
  for (const macro of Array.from(doc.getElementsByTagName('ac:structured-macro'))) {
    if (norm(macro.getAttribute('ac:name')) !== 'details') continue
    const id = Array.from(macro.getElementsByTagName('ac:parameter')).find(
      (p) => norm(p.getAttribute('ac:name')) === 'id',
    )
    if (id && norm(id.textContent) === wanted) return macro
  }
  return null
}

/**
 * The property keys a page contributes, in document order, each tagged with the
 * header zone it came from.
 *
 * A key is the first cell of a row that actually holds a pair. The merged section
 * rows ("General ISD Information", "Astound Approval") are not keys — and they are
 * recognised structurally rather than by their text: a separator's only cell spans
 * the whole row, so the row carries fewer than two cells, and a `th` first cell is
 * a heading either way. Those same separators delimit the zones: the first section
 * is the header card, every later one is an approval group (exactly the split the
 * doc model makes). Zones matter because the matrix orders card fields and
 * approver roles differently — see orderMatrixColumns.
 *
 * @param {string} storageXhtml  a page's `body.storage.value`
 * @param {string} propertiesId  the doc type's Content Properties id
 * @returns {{key: string, zone: 'card'|'approval'}[]}  deduplicated, in document order ([] when the page has no such macro)
 */
export function collectPropertyKeys(storageXhtml, propertiesId) {
  const doc = domino.createWindow(String(storageXhtml == null ? '' : storageXhtml)).document
  const macro = findPropertiesMacro(doc, propertiesId)
  if (!macro) return []

  const table = macro.getElementsByTagName('table')[0]
  if (!table) return []

  const entries = []
  let sectionIndex = -1
  for (const tr of Array.from(table.getElementsByTagName('tr'))) {
    const cells = Array.from(tr.children).filter((c) => isElement(c, 'td') || isElement(c, 'th'))
    const isSeparator = cells.length < 2 || isElement(cells[0], 'th')
    if (isSeparator) {
      sectionIndex++
      continue
    }
    const key = norm(cells[0].textContent)
    if (!key || entries.some((e) => e.key === key)) continue
    // Rows before any separator (a hand-built table) count as card fields.
    entries.push({ key, zone: sectionIndex <= 0 ? 'card' : 'approval' })
  }
  return entries
}

/**
 * Merge several pages' key entries: first appearance wins, so the leading document
 * sets the shape and later ones only add what they introduce. Deterministic for a
 * stable child order.
 */
export function mergePropertyKeys(entryLists) {
  const merged = []
  for (const entries of entryLists || []) {
    for (const entry of entries || []) {
      if (entry?.key && !merged.some((e) => e.key === entry.key)) merged.push(entry)
    }
  }
  return merged
}

// Which card fields a register table wants, and where. The approver roles sit
// between the two groups. These patterns describe OUR schema's card vocabulary
// (every FSD/ISD carries a "<TYPE> Status", an "Author/Owner", and the FSD a
// "Package/Set/Batch") — never a client's or a project's own values, so they hold
// across projects. The roles themselves stay fully dynamic: whatever the approval
// groups carry becomes a column.
const LEADING_CARD_KEYS = [/\bstatus\b/i, /^author\b/i]
// Trailing: a grouping field, in practice only present on FSDs.
const TRAILING_CARD_KEYS = [/\bpackage\b/i]

/**
 * The report's column order: status, author, every approver role in document
 * order, then the package/batch field.
 *
 * The remaining card fields (WBS code, project name, Jira reference, feature name)
 * are deliberately left out — they repeat what the page title and the documents
 * themselves already say, and each one costs horizontal room the approver columns
 * need. Only allowlisted card fields appear, so a project adding a card field never
 * silently widens the matrix; the dropped keys come back to the caller to report.
 *
 * @returns {{ columns: string[], dropped: string[] }}
 */
export function orderMatrixColumns(entries) {
  const list = entries || []
  const cardKeys = list.filter((e) => e.zone === 'card').map((e) => e.key)
  const approvals = list.filter((e) => e.zone === 'approval').map((e) => e.key)
  const pick = (patterns) => patterns.flatMap((re) => cardKeys.filter((k) => re.test(k)))

  const columns = []
  for (const key of [...pick(LEADING_CARD_KEYS), ...approvals, ...pick(TRAILING_CARD_KEYS)]) {
    if (!columns.includes(key)) columns.push(key)
  }
  return { columns, dropped: cardKeys.filter((k) => !columns.includes(k)) }
}

const escapeXml = (s) =>
  String(s == null ? '' : s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')

const unescapeXml = (s) =>
  String(s == null ? '' : s)
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;|&apos;/g, "'")
    .replace(/&amp;/g, '&')

/**
 * The report's parameters, as one object both the builder and the comparison use.
 *
 * Only parameters whose effect was verified on Cloud are emitted: `id` (scope to our
 * Content Properties macro), `cql` (label + ancestor), `headings` (the columns, in
 * order) and `firstcolumn` (the page-link column's title). The macro paginates on
 * its own past ~30 rows.
 */
export function matrixParams({ propertiesId, cql, headings, firstColumn = 'Document' }) {
  return {
    id: String(propertiesId || ''),
    cql: String(cql || ''),
    headings: (headings || []).join(','),
    firstcolumn: String(firstColumn || ''),
  }
}

const PARAM_ORDER = ['id', 'cql', 'headings', 'firstcolumn']

/** The `detailssummary` (Page Properties Report) macro in storage format. */
export function buildMatrixMacro(spec) {
  const params = matrixParams(spec)
  return (
    '<ac:structured-macro ac:name="detailssummary" ac:schema-version="2">' +
    PARAM_ORDER.map((k) => `<ac:parameter ac:name="${k}">${escapeXml(params[k])}</ac:parameter>`).join('') +
    '</ac:structured-macro>'
  )
}

/**
 * The parameters of the report already on a page, or null when it has none.
 *
 * Needed because a page cannot be compared to what we wrote byte for byte: on save
 * Confluence bumps `ac:schema-version`, stamps an `ac:macro-id`, and reorders the
 * parameters. The parameters themselves are what the matrix is about — the columns
 * and the scope — so they are what we compare.
 */
export function readMatrixParams(pageStorage, propertiesId) {
  const body = String(pageStorage == null ? '' : pageStorage)
  const range = findMatrixMacroRange(body, propertiesId)
  if (!range) return null

  const macro = body.slice(range.start, range.end)
  const found = {}
  const re = /<ac:parameter\b[^>]*ac:name="([^"]+)"[^>]*>([\s\S]*?)<\/ac:parameter>/gi
  for (let m = re.exec(macro); m; m = re.exec(macro)) found[m[1]] = unescapeXml(m[2])
  return Object.fromEntries(PARAM_ORDER.map((k) => [k, found[k] || '']))
}

const sameMatrixParams = (a, b) => Boolean(a) && Boolean(b) && PARAM_ORDER.every((k) => a[k] === b[k])

// The CQL that scopes a matrix to one parent's typed children.
export const matrixCql = (type, parentId) => `label = "${type}" and ancestor = ${parentId}`

/**
 * Read the parent's children and collect the keys they contribute.
 *
 * Children come from the page tree rather than a CQL search because the search
 * index lags a fresh publish, and a matrix rebuilt right after one must see it.
 * Each candidate is then checked for the type label, so the scanned set is exactly
 * the set the report's `label = "<type>" and ancestor = …` will show — otherwise an
 * unlabelled page could widen the columns while never appearing as a row.
 *
 * `api` is injected (rather than imported) so this module stays free of the
 * credential-gated Atlassian client: publish.mjs loads that lazily to keep its dry
 * run runnable without credentials.
 */
export async function scanMatrixSources({ api, parentId, type, propertiesId }) {
  const children = await api(`pages/${parentId}/children?limit=250`)
  const scanned = (children.results || []).length
  const contributors = []
  const unlabelled = []
  const entryLists = []

  for (const child of children.results || []) {
    const page = await api(`pages/${child.id}?body-format=storage`)
    const entries = collectPropertyKeys(page?.body?.storage?.value, propertiesId)
    if (!entries.length) continue
    const labels = await api(`pages/${child.id}/labels?limit=100`)
    if (!(labels.results || []).some((l) => l.name === type)) {
      unlabelled.push({ id: child.id, title: child.title })
      continue
    }
    contributors.push({ id: child.id, title: child.title, keys: entries.length })
    entryLists.push(entries)
  }

  return { scanned, contributors, unlabelled, entries: mergePropertyKeys(entryLists) }
}

/**
 * Scan the parent's children and put the matching report on the parent page.
 *
 * Reports rather than prints, so both the CLI and the publisher can phrase the
 * outcome their own way. `action` is 'unchanged' when the live report already
 * describes the same columns and scope: this runs on every typed publish, and a
 * no-op write would otherwise add a version to the parent's history each time.
 *
 * @returns {{ scanned, contributors, unlabelled, columns, dropped, macro: string|null, action: 'inserted'|'replaced'|'unchanged'|'skipped' }}
 */
export async function refreshDocMatrix({
  api,
  type,
  parentId,
  propertiesId,
  firstColumn = 'Document',
  dry = false,
}) {
  const scan = await scanMatrixSources({ api, parentId, type, propertiesId })
  const { columns, dropped } = orderMatrixColumns(scan.entries)
  if (!columns.length) return { ...scan, columns, dropped, macro: null, action: 'skipped' }

  const spec = { propertiesId, cql: matrixCql(type, parentId), headings: columns, firstColumn }
  const macro = buildMatrixMacro(spec)
  if (dry) return { ...scan, columns, dropped, macro, action: 'skipped' }

  const current = await api(`pages/${parentId}?body-format=storage`)
  const live = current?.body?.storage?.value || ''
  if (sameMatrixParams(readMatrixParams(live, propertiesId), matrixParams(spec))) {
    return { ...scan, columns, dropped, macro, action: 'unchanged' }
  }
  const { body, action } = upsertMatrixMacro(live, macro, propertiesId)

  await api(`pages/${parentId}`, {
    method: 'PUT',
    body: {
      id: parentId,
      status: current.status,
      title: current.title,
      body: { representation: 'storage', value: body },
      version: { number: current.version.number + 1, message: `${type.toUpperCase()} approval matrix` },
    },
  })
  return { ...scan, columns, dropped, macro, action, parentTitle: current.title }
}

/**
 * Put the macro on the parent page without touching anything else.
 *
 * An existing `detailssummary` carrying our `id` is replaced where it stands, so a
 * re-run keeps the surrounding prose and the macro's position on the page; only
 * when none exists is the macro appended. Reports for another type or another
 * Content Properties id are left alone.
 *
 * @returns {{ body: string, action: 'replaced'|'inserted' }}
 */
export function upsertMatrixMacro(pageStorage, macroStorage, propertiesId) {
  const body = String(pageStorage == null ? '' : pageStorage)
  const existing = findMatrixMacroRange(body, propertiesId)
  if (existing) {
    return {
      body: body.slice(0, existing.start) + macroStorage + body.slice(existing.end),
      action: 'replaced',
    }
  }
  const sep = body && !body.endsWith('\n') ? '\n' : ''
  return { body: `${body}${sep}${macroStorage}`, action: 'inserted' }
}

// Byte range of the `detailssummary` macro carrying this propertiesId, or null.
// Scanned on the raw string (not the DOM) so the rest of the page — code-macro
// CDATA, native-editor ids, hand-authored markup — comes back out verbatim.
function findMatrixMacroRange(body, propertiesId) {
  const open = /<ac:structured-macro\b[^>]*ac:name="detailssummary"[^>]*>/gi
  const wanted = norm(propertiesId)
  for (let m = open.exec(body); m; m = open.exec(body)) {
    const closeAt = body.indexOf('</ac:structured-macro>', m.index)
    if (closeAt < 0) break
    const end = closeAt + '</ac:structured-macro>'.length
    const id = /<ac:parameter\b[^>]*ac:name="id"[^>]*>([\s\S]*?)<\/ac:parameter>/i.exec(
      body.slice(m.index, end),
    )
    if (id && norm(id[1]) === wanted) return { start: m.index, end }
  }
  return null
}
