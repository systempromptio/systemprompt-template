import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

import { getDocType } from '../lib/doc/types/index.mjs'
import { renderDoc } from '../lib/doc/render.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const read = (...p) => readFileSync(join(HERE, ...p), 'utf8')

// Golden snapshots were captured from the pre-refactor renderer. A diff here means
// the storage XHTML changed; if intended, re-capture the __golden__ files.
for (const type of ['fsd', 'isd']) {
  test(`renderDoc(${type}) matches the golden storage snapshot`, () => {
    const md = read('__fixtures__', `example.${type}.md`)
    const model = getDocType(type).parse(md)
    const out = renderDoc(model, { type, includeToc: true })
    const gold = read('__golden__', `example.${type}.storage.html`)
    assert.equal(out, gold)
  })
}

test('renderDoc throws on an invalid model (validation on by default)', () => {
  const bad = { title: '', header: { sections: [] }, references: [], body: '', documentCard: {}, approvals: [] }
  assert.throws(() => renderDoc(bad, { type: 'fsd' }), /Document is invalid/)
})

test('renderDoc emits classic storage macros (status lozenge, TOC)', () => {
  const md = read('__fixtures__', 'example.fsd.md')
  const model = getDocType('fsd').parse(md)
  const out = renderDoc(model, { type: 'fsd', includeToc: true })
  assert.match(out, /<ac:structured-macro ac:name="status"/, 'status lozenge present')
  assert.match(out, /<ac:structured-macro ac:name="toc"/, 'TOC macro present')
})

test('chrome cells render their inline markdown instead of leaking the syntax', () => {
  const model = getDocType('isd').parse(read('__fixtures__', 'example.isd.md'))
  model.header.sections[0].rows.push([
    'Jira Reference',
    '[NAVA-38](https://example.atlassian.net/browse/NAVA-38) - Epic',
  ])
  model.references.push({
    material: 'Tags runtime',
    href: 'https://example.com/tags',
    text: 'Overview',
    card: false,
    notes: 'The `_satellite` runtime writes utag_data first',
  })

  const out = renderDoc(model, { type: 'isd' })

  assert.match(out, /<a href="https:\/\/example\.atlassian\.net\/browse\/NAVA-38">NAVA-38<\/a>/)
  assert.match(out, /<code>_satellite<\/code>/)
  assert.doesNotMatch(out, /\[NAVA-38\]\(/, 'no raw link syntax reaches the page')
  assert.ok(!out.includes('`_satellite`'), 'no literal backticks reach the page')
})

test('an in-page link anchors its target heading across body parts', () => {
  const model = getDocType('isd').parse(read('__fixtures__', 'example.isd.md'))
  model.body = [
    '## Events',
    '',
    'See [the bag entry](#object-bag-entry).',
    '',
    '## Shared objects',
    '',
    '### Object: `bag[]` entry',
    '',
    'Fields.',
    '',
  ].join('\n')

  const out = renderDoc(model, { type: 'isd' })
  assert.match(out, /<ac:link ac:anchor="object-bag-entry">/, 'link renders as an anchor link')
  assert.match(
    out,
    /<ac:structured-macro ac:name="anchor"><ac:parameter ac:name="">object-bag-entry<\/ac:parameter>/,
    'target heading in another body part still gets its anchor macro',
  )
})

test('renderDoc emits the Linked Jira Tickets macro only when pageId + appId are given', () => {
  const md = read('__fixtures__', 'example.isd.md')
  const model = getDocType('isd').parse(md)

  // Without the ids the section is omitted (goldens above stay unchanged).
  const bare = renderDoc(model, { type: 'isd', includeToc: true })
  assert.doesNotMatch(bare, /Linked Jira Tickets/)
  assert.doesNotMatch(bare, /ac:name="jira"/)

  // With them, the H2 + jira macro appear before the Document Change Log, driven
  // by the issuesWithRemoteLinksByGlobalId JQL and bound to the cloudId.
  const withLinks = renderDoc(model, {
    type: 'isd',
    includeToc: true,
    pageId: '1262551085',
    jiraAppId: 'app-xyz',
    jiraCloudId: 'cloud-123',
  })
  assert.match(withLinks, /<h2>Linked Jira Tickets<\/h2>/)
  assert.match(withLinks, /<ac:structured-macro ac:name="jira"/)
  assert.match(
    withLinks,
    /issuesWithRemoteLinksByGlobalId\("appId=app-xyz&amp;pageId=1262551085"\)/,
  )
  assert.match(withLinks, /<ac:parameter ac:name="serverId">cloud-123<\/ac:parameter>/)
  assert.ok(
    withLinks.indexOf('Linked Jira Tickets') < withLinks.indexOf('Document Change Log'),
    'Linked Jira Tickets precedes Document Change Log',
  )
})

test('a line break authored in a table cell publishes as a real break, not escaped text', () => {
  const model = getDocType('isd').parse(read('__fixtures__', 'example.isd.md'))
  model.body += [
    '',
    '## Break Probe',
    '',
    '| # | Scenario | THEN |',
    '| --- | --- | --- |',
    '| 1 | Container loads | the platform is present<br>the container reads consent |',
    '',
  ].join('\n')

  const out = renderDoc(model, { type: 'isd' })
  assert.match(out, /present<br \/>the container/)
  assert.ok(!out.includes('&lt;br&gt;'), 'the break is not published as literal text')
})
