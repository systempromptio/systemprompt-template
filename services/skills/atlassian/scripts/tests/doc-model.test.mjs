import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

import {
  parseDoc,
  serializeDoc,
  parseBody,
  collectHeadings,
  stripMention,
} from '../lib/doc/model.mjs'
import { getDocType } from '../lib/doc/types/index.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const fixture = (name) => readFileSync(join(HERE, '__fixtures__', name), 'utf8')

const BODY_SECTIONS = {
  fsd: ['In Scope Functional Requirements', 'Deferred Requirements', 'Change Requests'],
  isd: ['Requirements', 'Deferred Requirements', 'Change Requests'],
}

test('parseDoc extracts title, header card, references and body', () => {
  const model = parseDoc(fixture('example.fsd.md'), { bodySections: BODY_SECTIONS.fsd })
  assert.ok(model.title, 'title is non-empty')
  assert.ok(model.header.sections.length >= 1, 'has at least the header card section')
  assert.equal(model.header.sections[0].label, 'General FSD Information')
  assert.ok(model.references.length >= 1, 'parses Reference Materials rows')
  assert.match(model.body, /In Scope Functional Requirements/)
})

test('parseDoc pulls the chrome->body divider into the body when authored', () => {
  const md = [
    '# Sample',
    '',
    '## General FSD Information',
    '|  |  |',
    '| --- | --- |',
    '| Status | draft |',
    '',
    '---',
    '',
    '## In Scope Functional Requirements',
    '',
    'Body text.',
  ].join('\n')
  const model = parseDoc(md, { bodySections: BODY_SECTIONS.fsd })
  assert.match(model.body, /^---/, 'leading divider is preserved on the body')
})

test('parseDoc is fence-aware: a "## " inside a code fence is not a body start', () => {
  const md = [
    '# Sample',
    '',
    '## General FSD Information',
    '|  |  |',
    '| --- | --- |',
    '| Status | draft |',
    '',
    '```md',
    '## In Scope Functional Requirements',
    '```',
    '',
    '## In Scope Functional Requirements',
    '',
    'Real body.',
  ].join('\n')
  const model = parseDoc(md, { bodySections: BODY_SECTIONS.fsd })
  assert.match(model.body, /^## In Scope Functional Requirements\n\nReal body\.$/)
})

for (const type of ['fsd', 'isd']) {
  test(`parse -> serialize -> parse is stable for ${type}`, () => {
    const dt = getDocType(type)
    const model = dt.parse(fixture(`example.${type}.md`))
    const once = serializeDoc(model)
    const twice = serializeDoc(dt.parse(once))
    assert.equal(once, twice, 'second serialization is byte-identical')
  })

  test(`serialized ${type} preserves the title and header card`, () => {
    const dt = getDocType(type)
    const model = dt.parse(fixture(`example.${type}.md`))
    const md = serializeDoc(model)
    const reparsed = dt.parse(md)
    assert.equal(reparsed.title, model.title)
    assert.equal(reparsed.header.sections[0].label, model.header.sections[0].label)
    assert.equal(reparsed.references.length, model.references.length)
  })
}

test('collectHeadings is fence-aware and reports level + title', () => {
  const md = ['# T', '## A', '```', '## Fenced', '```', '### B'].join('\n')
  const heads = collectHeadings(md)
  assert.deepEqual(
    heads.map((h) => `${h.level}:${h.title}`),
    ['1:T', '2:A', '3:B'],
  )
})

test('parseBody returns a fence-aware H2/H3 outline (no requirement extraction)', () => {
  const body = ['## In Scope Functional Requirements', '### RQ-701 - Thing', '## Change Requests'].join('\n')
  const { sections } = parseBody(body)
  assert.deepEqual(
    sections.map((s) => `${s.level}:${s.title}`),
    ['2:In Scope Functional Requirements', '3:RQ-701 - Thing', '2:Change Requests'],
  )
  assert.equal('requirements' in { sections }, false)
})

test('stripMention drops a single leading @ and trims', () => {
  assert.equal(stripMention('@Jane Doe'), 'Jane Doe')
  assert.equal(stripMention('  Jane Doe '), 'Jane Doe')
  assert.equal(stripMention(''), '')
  assert.equal(stripMention(null), '')
})

test('parseDoc rejects non-string input', () => {
  assert.throws(() => parseDoc(null), TypeError)
})
