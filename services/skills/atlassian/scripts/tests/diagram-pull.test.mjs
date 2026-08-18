import { test } from 'node:test'
import assert from 'node:assert/strict'

import { collectDrawioAttachments, upsertDiagramBlock } from '../lib/diagrams/pull.mjs'

const att = (title) => ({ title, id: `att-${title}` })
const block = (id) => '```drawio:sequence:' + id + '\nparticipants:\n  - { id: a, title: A }\n```'

test('collectDrawioAttachments picks .drawio and pairs the sibling .png by basename', () => {
  const atts = [att('flow.drawio'), att('flow.png'), att('lonely.drawio'), att('photo.png')]
  const pairs = collectDrawioAttachments(atts)

  assert.equal(pairs.length, 2)
  const flow = pairs.find((p) => p.id === 'flow')
  assert.equal(flow.drawio.title, 'flow.drawio')
  assert.equal(flow.png.title, 'flow.png')

  const lonely = pairs.find((p) => p.id === 'lonely')
  assert.equal(lonely.png, null) // no sibling .png
})

test('upsertDiagramBlock appends the block + image when the id is absent', () => {
  const md = '# Doc\n\nIntro.\n'
  const out = upsertDiagramBlock(md, {
    id: 'flow',
    block: block('flow'),
    imageMarkdown: '![Flow](./assets/flow.png)',
  })
  assert.ok(out.startsWith('# Doc\n\nIntro.\n\n```drawio:sequence:flow'))
  assert.ok(out.includes('![Flow](./assets/flow.png)'))
})

test('upsertDiagramBlock replaces an existing block (and its image) in place, no dup', () => {
  const md = [
    '# Doc',
    '',
    '```drawio:sequence:flow',
    'participants:',
    '  - { id: OLD, title: OLD }',
    '```',
    '',
    '![Old](./assets/flow.png)',
    '',
    'Trailing prose.',
  ].join('\n')

  const out = upsertDiagramBlock(md, {
    id: 'flow',
    block: block('flow'),
    imageMarkdown: '![Flow](./assets/flow.png)',
  })

  assert.ok(!out.includes('OLD')) // old body gone
  assert.ok(out.includes('title: A')) // new body in
  assert.ok(out.includes('Trailing prose.')) // surrounding content preserved
  // Exactly one block + one image (no duplication).
  assert.equal((out.match(/```drawio:sequence:flow/g) || []).length, 1)
  assert.equal((out.match(/!\[[^\]]*\]\(\.\/assets\/flow\.png\)/g) || []).length, 1)
})

test('upsertDiagramBlock replaces a bare exported image in place when no block exists yet', () => {
  const md = '# Doc\n\nIntro.\n\n![Old alt](../../assets/9/flow.png)\n\nAfter.\n'
  const out = upsertDiagramBlock(md, {
    id: 'flow',
    block: block('flow'),
    imageMarkdown: '![Flow](../../assets/9/flow.png)',
  })

  // Exactly one block + one image, spliced where the flat image was (not appended).
  assert.equal((out.match(/```drawio:sequence:flow/g) || []).length, 1)
  assert.equal((out.match(/!\[[^\]]*\]\([^)]*\/flow\.png\)/g) || []).length, 1)
  assert.ok(out.indexOf('```drawio:sequence:flow') < out.indexOf('![Flow]'))
  assert.ok(out.includes('Intro.'))
  assert.ok(out.includes('After.'))
})

test('upsertDiagramBlock does not confuse a sibling image whose name ends with the id', () => {
  // `myflow.png` must NOT be treated as the `flow` diagram image.
  const md = '# Doc\n\n![Other](./assets/myflow.png)\n'
  const out = upsertDiagramBlock(md, {
    id: 'flow',
    block: block('flow'),
    imageMarkdown: '![Flow](./assets/flow.png)',
  })
  assert.ok(out.includes('![Other](./assets/myflow.png)')) // untouched
  assert.ok(out.includes('```drawio:sequence:flow')) // appended at end
})

test('upsertDiagramBlock is idempotent across repeated pulls', () => {
  const entry = { id: 'flow', block: block('flow'), imageMarkdown: '![Flow](./assets/flow.png)' }
  const once = upsertDiagramBlock('# Doc\n', entry)
  const twice = upsertDiagramBlock(once, entry)
  assert.equal(once, twice)
})
