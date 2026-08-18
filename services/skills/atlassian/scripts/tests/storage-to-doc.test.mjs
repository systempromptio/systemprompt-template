/**
 * Typed reverse pull (STORAGE → canonical markdown) tests.
 *
 * The invariant is canonical-equivalence, NOT byte-equality with the authored
 * fixtures: the shared serializer (serializeDoc) owns layout/spacing, so a faithful
 * reverse of a page's storage must serialize to the SAME canonical markdown as
 * parsing the authored source. We assert that against the golden FSD/ISD pairs,
 * plus the macro-inversion rules (status/mention/image/code) and the
 * NotDocTypeError fallback signal the export CLI relies on per page.
 */

import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

import { serializeDoc } from '../lib/doc/model.mjs'
import { getDocType } from '../lib/doc/types/index.mjs'
import { storageToDoc, parseStorageToModel, NotDocTypeError } from '../lib/doc/storage-to-doc.mjs'
import { renderDoc } from '../lib/doc/render.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const fixture = (name) => readFileSync(join(HERE, '__fixtures__', name), 'utf8')
const golden = (name) => readFileSync(join(HERE, '__golden__', name), 'utf8')

const TITLES = { fsd: 'Store Locator FSD', isd: 'Payment Gateway ISD' }

for (const type of ['fsd', 'isd']) {
  test(`reverse of the golden ${type} storage equals the canonical authored markdown`, async () => {
    const authored = fixture(`example.${type}.md`)
    const canonical = serializeDoc(getDocType(type).parse(authored))

    const { markdown } = await storageToDoc({
      type,
      storageXhtml: golden(`example.${type}.storage.html`),
      title: TITLES[type],
    })

    assert.equal(markdown, canonical)
  })
}

test('parseStorageToModel recovers the header card, approval groups and references', () => {
  const { model } = parseStorageToModel(golden('example.fsd.storage.html'), {
    cardHeading: 'General FSD Information',
  })

  assert.equal(model.header.sections[0].label, 'General FSD Information')
  // The card status badge is reversed to its plain word.
  assert.deepEqual(model.header.sections[0].rows.at(-1), ['FSD Status', 'draft'])
  assert.deepEqual(
    model.header.sections.map((s) => s.label),
    ['General FSD Information', 'Astound Approval', 'Client Approval'],
  )
  // A roster status badge is reversed too.
  assert.deepEqual(model.header.sections[1].rows[0], ['SA', 'approved', 'John Reviewer'])

  // Labeled link vs. bare-url inline card are inverted distinctly.
  assert.deepEqual(model.references, [
    { material: 'Design', href: 'https://figma.com/file/abc', text: 'Figma board', card: false, notes: 'Source of truth' },
    { material: 'API spec', href: 'https://example.com/api', text: '', card: true, notes: '-' },
  ])
})

test('chrome cells round-trip the inline markdown they were authored with', () => {
  const model = getDocType('isd').parse(fixture('example.isd.md'))
  const jiraRow = ['Jira Reference', '[NAVA-38](https://example.atlassian.net/browse/NAVA-38) - Epic']
  const notes = 'The `_satellite` runtime writes utag_data first'
  model.header.sections[0].rows.push(jiraRow)
  model.references.push({
    material: 'Tags runtime',
    href: 'https://example.com/tags',
    text: 'Overview',
    card: false,
    notes,
  })

  const { model: back } = parseStorageToModel(renderDoc(model, { type: 'isd' }), {
    cardHeading: 'General ISD Information',
  })

  assert.deepEqual(back.header.sections[0].rows.at(-1), jiraRow)
  // Plain text keeps its underscores: the reverse must not emit markdown escapes
  // the forward inline renderer would print as literal backslashes.
  assert.equal(back.references.at(-1).notes, notes)
})

test('a natively-edited page (no <hr/>, local-id headings, extra layout) still recovers its full body', () => {
  const { model } = parseStorageToModel(fixture('native-edited.isd.storage.html'), {
    cardHeading: 'General ISD Information',
  })

  // The body is recovered even though the template's <hr/> dividers are gone and
  // every heading carries a native-editor local-id.
  assert.match(model.body, /^## Requirements$/m)
  assert.match(model.body, /### RQ-100 - Authorize payment/)
  assert.match(model.body, /^## Integration Specification$/m)
  assert.match(model.body, /^## Change Requests$/m)
  // A body table survives.
  assert.match(model.body, /\| Field \| Source \|/)
  assert.match(model.body, /\| amount \| order\.total \|/)
  // An inline status badge is reversed to a backtick word.
  assert.match(model.body, /`approved`/)

  // The script-owned footer is stripped, not leaked into the body.
  assert.doesNotMatch(model.body, /Document Change Log/i)
  assert.doesNotMatch(model.body, /change-history/i)
  // Reference Materials is chrome (captured separately), never in the body.
  assert.doesNotMatch(model.body, /Reference Materials/i)
  // No presentational layout wrappers or native-editor ids leak through.
  assert.doesNotMatch(model.body, /ac:layout/i)
  assert.doesNotMatch(model.body, /local-id/i)
  // The Content Properties wrapper around the header card closes before the body.
  assert.doesNotMatch(model.body, /ac:rich-text-body/i)
  // The status-badge legend paragraph is dropped too.
  assert.doesNotMatch(model.body, /Status badge values/i)

  // The header card + references still parse structurally.
  assert.equal(model.header.sections[0].label, 'General ISD Information')
  assert.deepEqual(model.references, [
    { material: 'Contract', href: 'https://example.com/contract', text: 'API contract', card: false, notes: 'v2' },
  ])
})

test('a resolved @mention is reversed to its display name via mentionNames', () => {
  const storage = `<ac:layout><ac:layout-cell><table><tbody>
<tr><th colspan="3"><p><strong>General ISD Information</strong></p></th></tr>
<tr><td><p>Author/Owner</p></td><td colspan="2"><p><ac:link><ri:user ri:account-id="557058:abc"/></ac:link></p></td></tr>
</tbody></table></ac:layout-cell></ac:layout><hr/><h2>Requirements</h2><p>Body.</p>`

  const { model } = parseStorageToModel(storage, {
    cardHeading: 'General ISD Information',
    mentionNames: { '557058:abc': 'Viktor Durnev' },
  })
  assert.deepEqual(model.header.sections[0].rows[0], ['Author/Owner', 'Viktor Durnev'])
})

test('an anchor link reverses to an in-page markdown link and its target macro is dropped', async () => {
  const storage = `<ac:layout><ac:layout-cell><table><tbody>
<tr><th colspan="3"><p><strong>General ISD Information</strong></p></th></tr>
<tr><td><p>Author/Owner</p></td><td colspan="2"><p>Viktor Durnev</p></td></tr>
</tbody></table></ac:layout-cell></ac:layout><hr/><h2>Requirements</h2>
<p>See <ac:link ac:anchor="object-bag-entry"><ac:plain-text-link-body><![CDATA[the bag entry]]></ac:plain-text-link-body></ac:link>.</p>
<h4><ac:structured-macro ac:name="anchor"><ac:parameter ac:name="">object-bag-entry</ac:parameter></ac:structured-macro>Object: <code>bag[]</code> entry</h4>`

  const { markdown } = await storageToDoc({ type: 'isd', storageXhtml: storage, title: 'X' })
  assert.match(markdown, /See \[the bag entry\]\(#object-bag-entry\)\./)
  assert.match(markdown, /#### Object: `bag\[\]` entry/)
})

test('storageToDoc resolves account-ids through resolveAccountId', async () => {
  const storage = `<ac:layout><ac:layout-cell><table><tbody>
<tr><th colspan="3"><p><strong>General ISD Information</strong></p></th></tr>
<tr><td><p>Author/Owner</p></td><td colspan="2"><p><ac:link><ri:user ri:account-id="id-1"/></ac:link></p></td></tr>
</tbody></table></ac:layout-cell></ac:layout><hr/><h2>Requirements</h2><p>Body.</p>`

  const seen = []
  const { markdown } = await storageToDoc({
    type: 'isd',
    storageXhtml: storage,
    title: 'X',
    resolveAccountId: (id) => {
      seen.push(id)
      return 'Nina Analyst'
    },
  })
  assert.deepEqual(seen, ['id-1'])
  assert.match(markdown, /\| Author\/Owner \| Nina Analyst \|/)
})

test('body macros are inverted: code fence, ac:image, backtick status', () => {
  const storage = `<ac:layout><ac:layout-cell><table><tbody>
<tr><th colspan="3"><p><strong>General ISD Information</strong></p></th></tr>
</tbody></table></ac:layout-cell></ac:layout><hr/>
<h2>Integration Specification</h2>
<ac:structured-macro ac:name="code"><ac:parameter ac:name="language">json</ac:parameter><ac:plain-text-body><![CDATA[{ "ok": true }]]></ac:plain-text-body></ac:structured-macro>
<p><ac:image ac:alt="Flow"><ri:attachment ri:filename="flow.png"/></ac:image></p>
<p>Health is <ac:structured-macro ac:name="status"><ac:parameter ac:name="colour">Green</ac:parameter><ac:parameter ac:name="title">approved</ac:parameter></ac:structured-macro>.</p>`

  const { model, imageFilenames } = parseStorageToModel(storage, {
    cardHeading: 'General ISD Information',
    imageRelPrefix: '../assets/123',
  })
  assert.match(model.body, /```json\n\{ "ok": true \}\n```/)
  assert.match(model.body, /!\[Flow\]\(\.\.\/assets\/123\/flow\.png\)/)
  assert.match(model.body, /`approved`/)
  assert.deepEqual(imageFilenames, ['flow.png'])
})

test('NotDocTypeError is thrown when the card heading is absent (enables per-page fallback)', () => {
  const generic = `<h2>Some other page</h2><p>Just a normal Confluence page.</p><hr/><p>More.</p>`
  assert.throws(
    () => parseStorageToModel(generic, { cardHeading: 'General ISD Information' }),
    NotDocTypeError,
  )
})

test('storageToDoc propagates NotDocTypeError for the wrong type', async () => {
  await assert.rejects(
    storageToDoc({ type: 'isd', storageXhtml: golden('example.fsd.storage.html'), title: 'x' }),
    (err) => err instanceof NotDocTypeError,
  )
})

test('an ordinal table survives the round trip as a pipe table, keeping its line breaks', async () => {
  const model = getDocType('isd').parse(fixture('example.isd.md'))
  const row = '| 1 | Container loads | the platform is present<br>the container reads consent |'
  model.body += ['', '## Break Probe', '', '| # | Scenario | THEN |', '| --- | --- | --- |', row].join(
    '\n',
  )

  const storage = renderDoc(model, { type: 'isd' })
  // The ordinal column makes the renderer pin widths with a <colgroup>, which is
  // exactly the shape that used to defeat the reverse table conversion.
  assert.match(storage, /<colgroup>/)

  const { markdown } = await storageToDoc({
    type: 'isd',
    storageXhtml: storage,
    title: 'Payment Gateway ISD',
  })
  assert.ok(!/^<table/m.test(markdown), 'no table falls back to raw HTML')
  assert.match(markdown, new RegExp(row.replace(/[|\\]/g, '\\$&')))
})

test('a bold header row reverses to a plain markdown header, so a re-publish is a no-op', async () => {
  const model = getDocType('isd').parse(fixture('example.isd.md'))
  const header = '| Q# | Question | Owner | Notes | Decision |'
  model.body += ['', '## Open Questions', '', header, '| --- | --- | --- | --- | --- |'].join('\n')

  const storage = renderDoc(model, { type: 'isd' })
  assert.match(storage, /<th><strong>Question<\/strong><\/th>/, 'the published header is bold')

  const { markdown } = await storageToDoc({
    type: 'isd',
    storageXhtml: storage,
    title: 'Payment Gateway ISD',
  })
  assert.match(markdown, new RegExp(header.replace(/[|\\]/g, '\\$&')))
  assert.ok(!markdown.includes('**Question**'), 'the bold wrapper does not leak into the working copy')
})

test('a cell keeps its brackets and leading hyphen unescaped, so a re-publish stays clean', async () => {
  const model = getDocType('isd').parse(fixture('example.isd.md'))
  const row = '| `products[]` | object[] | - |'
  model.body += ['', '## Escape Probe', '', '| Path | Type | Value |', '| --- | --- | --- |', row].join(
    '\n',
  )

  const { markdown } = await storageToDoc({
    type: 'isd',
    storageXhtml: renderDoc(model, { type: 'isd' }),
    title: 'Payment Gateway ISD',
  })
  assert.match(markdown, /\| `products\[\]` \| object\[\] \| - \|/)
  assert.ok(!markdown.includes('object\\['), 'no stray backslashes reach the working copy')
})
