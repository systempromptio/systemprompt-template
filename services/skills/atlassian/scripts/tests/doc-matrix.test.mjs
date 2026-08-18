import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

import {
  buildMatrixMacro,
  collectPropertyKeys,
  matrixCql,
  mergePropertyKeys,
  orderMatrixColumns,
  readMatrixParams,
  refreshDocMatrix,
  upsertMatrixMacro,
} from '../lib/doc/matrix.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const read = (...p) => readFileSync(join(HERE, ...p), 'utf8')
const card = (key) => ({ key, zone: 'card' })
const approval = (key) => ({ key, zone: 'approval' })

test('collectPropertyKeys reads a rendered ISD header and tags each key with its zone', () => {
  const entries = collectPropertyKeys(read('__golden__', 'example.isd.storage.html'), 'isd-header')

  assert.deepEqual(entries, [
    card('WBS-Feature Name'),
    card('Project Name'),
    card('Package/Set/Batch'),
    card('Author/Owner'),
    card('ISD Status'),
    approval('FA'),
    approval('PO'),
  ])
  // Section labels live in merged <th> rows and are not key-value pairs.
  assert.ok(!entries.some((e) => e.key === 'General ISD Information'))
  assert.ok(!entries.some((e) => e.key === 'Astound Approval'))
})

test('collectPropertyKeys survives the native editor (local-ids, data-layout, macro ids)', () => {
  const entries = collectPropertyKeys(read('__fixtures__', 'native-edited.isd.storage.html'), 'isd-header')
  assert.deepEqual(entries, [
    card('WBS-Feature Name'),
    card('Author/Owner'),
    card('ISD Status'),
    approval('FA'),
  ])
})

test('collectPropertyKeys ignores a page whose Content Properties id is another type', () => {
  const fsd = read('__golden__', 'example.fsd.storage.html')
  assert.deepEqual(collectPropertyKeys(fsd, 'isd-header'), [])
  assert.ok(collectPropertyKeys(fsd, 'fsd-header').some((e) => e.key === 'SA' && e.zone === 'approval'))
})

test('collectPropertyKeys returns nothing for a page with no Content Properties macro', () => {
  const bare = '<h1>Notes</h1><table><tbody><tr><td><p>SA</p></td><td><p>approved</p></td></tr></tbody></table>'
  assert.deepEqual(collectPropertyKeys(bare, 'fsd-header'), [])
})

test('mergePropertyKeys keeps first-appearance order and appends only what is new', () => {
  const merged = mergePropertyKeys([
    [card('WBS Code'), card('ISD Status'), approval('FA'), approval('QA')],
    [card('WBS Code'), card('ISD Status'), approval('FA'), approval('BE')],
    [approval('PO')],
  ])
  assert.deepEqual(merged.map((e) => e.key), ['WBS Code', 'ISD Status', 'FA', 'QA', 'BE', 'PO'])
})

test('orderMatrixColumns runs status, author, approvers, package — and drops the rest of the card', () => {
  const { columns, dropped } = orderMatrixColumns(
    collectPropertyKeys(read('__golden__', 'example.fsd.storage.html'), 'fsd-header'),
  )

  assert.deepEqual(columns, ['FSD Status', 'Author/Owner', 'SA', 'BE', 'PO', 'Package/Set/Batch'])
  // The page title already carries the feature, and these repeat the document.
  assert.deepEqual(dropped, ['WBS-Feature Name', 'Project Name'])
})

test('orderMatrixColumns omits the trailing package column when no document carries it', () => {
  const { columns } = orderMatrixColumns([
    card('WBS Code'),
    card('Author/Owner'),
    card('ISD Status'),
    approval('FA'),
    approval('PO'),
  ])
  assert.deepEqual(columns, ['ISD Status', 'Author/Owner', 'FA', 'PO'])
})

test('orderMatrixColumns keeps approver roles exactly as the documents order them', () => {
  const { columns } = orderMatrixColumns([
    card('ISD Status'),
    approval('FA'),
    approval('FE'),
    approval('BE'),
    approval('QA'),
    approval('PO'),
  ])
  assert.deepEqual(columns, ['ISD Status', 'FA', 'FE', 'BE', 'QA', 'PO'])
})

test('buildMatrixMacro emits only verified parameters and escapes the CQL', () => {
  const macro = buildMatrixMacro({
    propertiesId: 'isd-header',
    cql: matrixCql('isd', '1215823913'),
    headings: ['ISD Status', 'FA'],
    firstColumn: 'Document',
  })

  assert.match(macro, /ac:name="detailssummary"/)
  assert.match(macro, /<ac:parameter ac:name="id">isd-header<\/ac:parameter>/)
  assert.match(macro, /<ac:parameter ac:name="headings">ISD Status,FA<\/ac:parameter>/)
  assert.match(macro, /<ac:parameter ac:name="firstcolumn">Document<\/ac:parameter>/)
  // Storage format is XHTML: the CQL's quotes must not break the parameter.
  assert.match(macro, /label = &quot;isd&quot; and ancestor = 1215823913/)
  assert.ok(!macro.includes('"isd" and'))
})

test('upsertMatrixMacro appends when the parent carries no report yet', () => {
  const { body, action } = upsertMatrixMacro('<h1>ISD</h1><p>Index of specs.</p>', '<MACRO/>', 'isd-header')
  assert.equal(action, 'inserted')
  assert.equal(body, '<h1>ISD</h1><p>Index of specs.</p>\n<MACRO/>')
})

test('upsertMatrixMacro replaces our report in place, keeping the surrounding page', () => {
  const old =
    '<ac:structured-macro ac:name="detailssummary" ac:schema-version="2">' +
    '<ac:parameter ac:name="id">isd-header</ac:parameter>' +
    '<ac:parameter ac:name="headings">ISD Status</ac:parameter>' +
    '</ac:structured-macro>'
  const page = `<h1>ISD</h1>${old}<p>Ask the SA lead before editing.</p>`

  const { body, action } = upsertMatrixMacro(page, '<MACRO/>', 'isd-header')
  assert.equal(action, 'replaced')
  assert.equal(body, '<h1>ISD</h1><MACRO/><p>Ask the SA lead before editing.</p>')
})

test('upsertMatrixMacro leaves a report belonging to another id alone', () => {
  const foreign =
    '<ac:structured-macro ac:name="detailssummary" ac:schema-version="2">' +
    '<ac:parameter ac:name="id">release-notes</ac:parameter>' +
    '</ac:structured-macro>'
  const { body, action } = upsertMatrixMacro(foreign, '<MACRO/>', 'isd-header')
  assert.equal(action, 'inserted')
  assert.ok(body.startsWith(foreign), 'the foreign report survives verbatim')
  assert.ok(body.endsWith('<MACRO/>'))
})

// A fake Confluence just rich enough for refreshDocMatrix: one parent with children,
// each carrying a storage body and labels. PUTs are recorded, never applied, so a
// test can assert on what the publisher would have written.
function fakeConfluence({ parentBody = '<h1>ISD</h1>', children = [] }) {
  const puts = []
  const state = { parentBody }
  const api = async (path, opts) => {
    if (opts?.method === 'PUT') {
      puts.push(opts.body)
      state.parentBody = opts.body.body.value
      return {}
    }
    if (/\/children\b/.test(path)) return { results: children.map(({ id, title }) => ({ id, title })) }
    if (/\/labels\b/.test(path)) {
      const child = children.find((c) => path.includes(String(c.id)))
      return { results: (child?.labels || []).map((name) => ({ name })) }
    }
    const child = children.find((c) => path.includes(String(c.id)))
    if (child) return { body: { storage: { value: child.storage } } }
    return { status: 'current', title: 'ISD', version: { number: 7 }, body: { storage: { value: state.parentBody } } }
  }
  return { api, puts, state }
}

const isdChild = (id, extra = '') => ({
  id,
  title: `Doc ${id}`,
  labels: ['isd'],
  storage:
    '<ac:structured-macro ac:name="details"><ac:parameter ac:name="id">isd-header</ac:parameter><ac:rich-text-body>' +
    '<table><tbody>' +
    '<tr><th colspan="3"><p>General ISD Information</p></th></tr>' +
    '<tr><td><p>Author/Owner</p></td><td colspan="2"><p>Dane</p></td></tr>' +
    '<tr><td><p>ISD Status</p></td><td colspan="2"><p>draft</p></td></tr>' +
    '<tr><th colspan="3"><p>Astound Approval</p></th></tr>' +
    `<tr><td><p>FA</p></td><td><p>approved</p></td><td><p>Ben</p></td></tr>${extra}` +
    '</tbody></table></ac:rich-text-body></ac:structured-macro>',
})

test('refreshDocMatrix writes the report once and then reports it unchanged', async () => {
  const wiki = fakeConfluence({ children: [isdChild(11)] })
  const args = { api: wiki.api, type: 'isd', parentId: '1', propertiesId: 'isd-header' }

  const first = await refreshDocMatrix(args)
  assert.equal(first.action, 'inserted')
  assert.deepEqual(first.columns, ['ISD Status', 'Author/Owner', 'FA'])
  assert.equal(wiki.puts.length, 1)

  // The publisher runs this after EVERY typed publish, so an unchanged rebuild must
  // not add a version to the parent's history.
  const second = await refreshDocMatrix(args)
  assert.equal(second.action, 'unchanged')
  assert.equal(wiki.puts.length, 1, 'no second write')
})

test('refreshDocMatrix sees its own report through Confluence storage normalization', async () => {
  // What Cloud actually returns after saving our macro: schema-version bumped, an
  // ac:macro-id stamped, parameters reordered. A byte comparison would call this a
  // change and add a parent version on every single publish.
  const normalized =
    '<p>Index.</p><ac:structured-macro ac:name="detailssummary" ac:schema-version="3" ' +
    'ac:macro-id="0ee87983-15e2-4b46-9b71-1941375ff598">' +
    '<ac:parameter ac:name="firstcolumn">Document</ac:parameter>' +
    '<ac:parameter ac:name="headings">ISD Status,Author/Owner,FA</ac:parameter>' +
    '<ac:parameter ac:name="id">isd-header</ac:parameter>' +
    '<ac:parameter ac:name="cql">label = &quot;isd&quot; and ancestor = 1</ac:parameter>' +
    '</ac:structured-macro>'
  const wiki = fakeConfluence({ parentBody: normalized, children: [isdChild(11)] })

  const r = await refreshDocMatrix({ api: wiki.api, type: 'isd', parentId: '1', propertiesId: 'isd-header' })
  assert.equal(r.action, 'unchanged')
  assert.equal(wiki.puts.length, 0)
})

test('readMatrixParams unescapes the stored CQL so it compares against the source', () => {
  const stored =
    '<ac:structured-macro ac:name="detailssummary" ac:schema-version="3">' +
    '<ac:parameter ac:name="id">fsd-header</ac:parameter>' +
    '<ac:parameter ac:name="cql">label = &quot;fsd&quot; and ancestor = 9</ac:parameter>' +
    '</ac:structured-macro>'
  const params = readMatrixParams(stored, 'fsd-header')
  assert.equal(params.cql, matrixCql('fsd', '9'))
  assert.equal(params.headings, '', 'a missing parameter reads as empty, never undefined')
  assert.equal(readMatrixParams(stored, 'isd-header'), null)
})

test('refreshDocMatrix rewrites the report when a document adds a role', async () => {
  const extraRole = '<tr><td><p>QA</p></td><td><p>not started</p></td><td><p>TBC</p></td></tr>'
  const wiki = fakeConfluence({ children: [isdChild(11)] })
  const args = { api: wiki.api, type: 'isd', parentId: '1', propertiesId: 'isd-header' }

  await refreshDocMatrix(args)
  const grown = await refreshDocMatrix({ ...args, api: fakeConfluence({
    parentBody: wiki.state.parentBody,
    children: [isdChild(11, extraRole)],
  }).api })

  assert.equal(grown.action, 'replaced')
  assert.deepEqual(grown.columns, ['ISD Status', 'Author/Owner', 'FA', 'QA'])
})

test('refreshDocMatrix skips a child that lacks the type label, and writes nothing when none qualify', async () => {
  const unlabelled = { ...isdChild(12), labels: ['scratch'] }
  const wiki = fakeConfluence({ children: [unlabelled] })

  const r = await refreshDocMatrix({ api: wiki.api, type: 'isd', parentId: '1', propertiesId: 'isd-header' })
  assert.equal(r.action, 'skipped')
  assert.deepEqual(r.columns, [])
  assert.deepEqual(r.unlabelled, [{ id: 12, title: 'Doc 12' }])
  assert.equal(wiki.puts.length, 0)
})

test('refreshDocMatrix --dry reports the columns without touching the page', async () => {
  const wiki = fakeConfluence({ children: [isdChild(11)] })
  const r = await refreshDocMatrix({
    api: wiki.api,
    type: 'isd',
    parentId: '1',
    propertiesId: 'isd-header',
    dry: true,
  })
  assert.deepEqual(r.columns, ['ISD Status', 'Author/Owner', 'FA'])
  assert.match(r.macro, /ac:name="detailssummary"/)
  assert.equal(wiki.puts.length, 0)
})

test('upsertMatrixMacro is idempotent across re-runs', () => {
  const macro = buildMatrixMacro({
    propertiesId: 'fsd-header',
    cql: matrixCql('fsd', '42'),
    headings: ['FSD Status', 'SA'],
  })
  const once = upsertMatrixMacro('<h1>FSD</h1>', macro, 'fsd-header').body
  const twice = upsertMatrixMacro(once, macro, 'fsd-header')
  assert.equal(twice.action, 'replaced')
  assert.equal(twice.body, once)
})
