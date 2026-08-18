import { test } from 'node:test'
import assert from 'node:assert/strict'

import { mdToStorage, collectAssets } from '../lib/doc/md-to-storage.mjs'

const body = (md, opts) => mdToStorage(md, opts).body

test('headings render as storage <h2>/<h3> and drop the leading H1 by default', () => {
  const out = body('# Title\n\n## Section\n\n### Sub')
  assert.equal(/<h1>/.test(out), false, 'first H1 dropped (it is the page title)')
  assert.match(out, /<h2>Section<\/h2>/)
  assert.match(out, /<h3>Sub<\/h3>/)
})

test('inline emphasis, code and links convert', () => {
  const out = body('This is **bold**, _em_, `code` and a [link](https://x/y).')
  assert.match(out, /<strong>bold<\/strong>/)
  assert.match(out, /<em>em<\/em>/)
  assert.match(out, /<code>code<\/code>/)
  assert.match(out, /<a href="https:\/\/x\/y">link<\/a>/)
})

test('an in-page link becomes an anchor link, anchoring only the heading it targets', () => {
  const out = body('# T\n\n## Shared objects\n\n#### Object: `bag[]` entry\n\nSee [the bag entry](#object-bag-entry).')
  assert.match(
    out,
    /<ac:link ac:anchor="object-bag-entry"><ac:plain-text-link-body><!\[CDATA\[the bag entry\]\]><\/ac:plain-text-link-body><\/ac:link>/,
  )
  assert.match(
    out,
    /<h4><ac:structured-macro ac:name="anchor"><ac:parameter ac:name="">object-bag-entry<\/ac:parameter><\/ac:structured-macro>Object: <code>bag\[\]<\/code> entry<\/h4>/,
  )
  assert.equal(/<h2><ac:structured-macro/.test(out), false, 'unreferenced heading carries no anchor macro')
})

test('text is escaped through the shared xhtml choke point', () => {
  const out = body('a & b < c > d')
  assert.match(out, /a &amp; b &lt; c &gt; d/)
})

test('an escaped pipe stays inside its cell instead of splitting the row', () => {
  const out = body('| Surface | Construction |\n| --- | --- |\n| Footer link | `"footer\\|" + trimmed text` |')
  assert.match(out, /<tr><td>Footer link<\/td><td><code>"footer\|" \+ trimmed text<\/code><\/td><\/tr>/)
  assert.equal(/<td>[^<]*\\/.test(out), false, 'no stranded backslash leaks into a cell')
})

test('two tables in one run of pipe lines stay two tables', () => {
  const out = body('| A | B |\n| --- | --- |\n| a | b |\n| Surface | Construction |\n| --- | --- |\n| Footer link | x |')
  assert.equal((out.match(/<table>/g) || []).length, 2, 'the second separator row starts a new table')
  assert.match(
    out,
    /<table><tbody><tr><th><strong>Surface<\/strong><\/th><th><strong>Construction<\/strong><\/th><\/tr>/,
    'its header is a header, not a data row',
  )
})

test('a header row is bold like a natively-edited table, and never double-bolded', () => {
  const out = body('| Q# | Question | **Owner** |\n| --- | --- | --- |\n| 1 | why | Jane |')
  assert.match(out, /<th><strong>Q#<\/strong><\/th><th><strong>Question<\/strong><\/th>/)
  assert.match(out, /<th><strong>Owner<\/strong><\/th>/, 'an authored bold header stays single-wrapped')
  assert.match(out, /<td>1<\/td>/, 'data cells stay plain')
})

test('unordered and ordered lists render', () => {
  const ul = body('- one\n- two')
  assert.match(ul, /<ul>/)
  assert.match(ul, /<li>one<\/li>/)
  const ol = body('1. first\n2. second')
  assert.match(ol, /<ol>/)
  assert.match(ol, /<li>first<\/li>/)
})

test('fenced code becomes a code macro with CDATA (identifiers untouched)', () => {
  const out = body('```js\nconst a = 1 < 2;\n```')
  assert.match(out, /<ac:structured-macro ac:name="code"/)
  assert.match(out, /<!\[CDATA\[[\s\S]*const a = 1 < 2;[\s\S]*\]\]>/, 'raw code kept verbatim in CDATA')
})

test('badgeMap turns backtick status words into status lozenges', () => {
  const out = body('State: `approved` now.', { badgeMap: { approved: 'Green' } })
  assert.match(out, /<ac:structured-macro ac:name="status"/)
  assert.match(out, /<ac:parameter ac:name="colour">Green<\/ac:parameter>/)
})

test('mentionMap renders a resolved @name as a user link, else plain text', () => {
  const resolved = body('Owner @Jane Doe here.', { mentionMap: { 'Jane Doe': 'acc-1' } })
  assert.match(resolved, /<ri:user ri:account-id="acc-1"/)
  const plain = body('Owner @Jane Doe here.')
  assert.equal(/<ri:user/.test(plain), false)
  assert.match(plain, /@Jane Doe/)
})

test('thematicBreak and blankParagraphs are opt-in (off by default)', () => {
  const off = body('a\n\n---\n\nb')
  assert.equal(/<hr\s*\/>/.test(off), false)
  const on = mdToStorage('a\n\n---\n\nb', { thematicBreak: true }).body
  assert.match(on, /<hr\s*\/>/)
})

test('collectAssets returns de-duplicated image src paths', () => {
  const md = '![a](./assets/x.png)\n\n![b](./assets/x.png)\n\n![c](./assets/y.png)'
  assert.deepEqual(collectAssets(md), ['./assets/x.png', './assets/y.png'])
})
