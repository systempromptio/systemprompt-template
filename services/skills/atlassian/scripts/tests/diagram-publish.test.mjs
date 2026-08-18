import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { companionFor, collectDiagrams } from '../lib/diagrams/publish.mjs'
import { mdToStorage } from '../lib/doc/md-to-storage.mjs'

test('companionFor swaps the extension to .drawio', () => {
  assert.equal(companionFor('/a/b/flow.png'), '/a/b/flow.drawio')
  assert.equal(companionFor('/a/b/flow.svg'), '/a/b/flow.drawio')
})

test('collectDiagrams collects the sibling .drawio companion of a diagram image', () => {
  const dir = mkdtempSync(join(tmpdir(), 'diagpub-'))
  try {
    writeFileSync(join(dir, 'flow.png'), 'png')
    writeFileSync(join(dir, 'flow.drawio'), '<mxfile/>')
    writeFileSync(join(dir, 'photo.png'), 'png') // plain image, no sibling

    const r = collectDiagrams({ mdDir: dir, assetPaths: ['./flow.png', './photo.png'] })

    assert.deepEqual(r.images, [join(dir, 'flow.png'), join(dir, 'photo.png')])
    assert.deepEqual(r.companions, [join(dir, 'flow.drawio')])
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
})

test('collectDiagrams resolves assets relative to the md dir and dedupes companions', () => {
  const sub = mkdtempSync(join(tmpdir(), 'diagpub-assets-'))
  try {
    writeFileSync(join(sub, 'd.png'), 'png')
    writeFileSync(join(sub, 'd.drawio'), '<mxfile/>')

    const r = collectDiagrams({ mdDir: sub, assetPaths: ['./d.png', 'd.png'] })
    assert.deepEqual(r.images, [join(sub, 'd.png'), join(sub, 'd.png')])
    assert.deepEqual(r.companions, [join(sub, 'd.drawio')]) // deduped
  } finally {
    rmSync(sub, { recursive: true, force: true })
  }
})

test('mdToStorage strips drawio spec fences but keeps normal code blocks', () => {
  const md = [
    '# Doc',
    '',
    '```drawio:sequence:integration-flow',
    'participants:',
    '  - { id: a, title: A }',
    '```',
    '',
    '![Flow](./assets/integration-flow.png)',
    '',
    '```js',
    'const x = 1',
    '```',
  ].join('\n')

  const { body } = mdToStorage(md)

  // The spec fence must not survive into storage in any form.
  assert.ok(!body.includes('drawio:sequence'))
  assert.ok(!body.includes('participants:'))
  // A normal code block still becomes a code macro.
  assert.ok(body.includes('const x = 1'))
  assert.ok(/ac:name="code"/.test(body) || /<ac:structured-macro/.test(body))
  // The image after the block is preserved.
  assert.ok(body.includes('integration-flow.png'))
})

test('mdToStorage also strips bare ```drawio and ```diagram fences', () => {
  for (const lang of ['drawio', 'diagram']) {
    const md = ['```' + lang, 'type: sequence', 'id: x', '```'].join('\n')
    const { body } = mdToStorage(md)
    assert.ok(!body.includes('type: sequence'), `lang ${lang} should be stripped`)
  }
})
