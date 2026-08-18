import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import {
  sha256File,
  formatComment,
  parseHash,
  decideAction,
  decidePrune,
  HASH_PREFIX,
} from '../lib/diagrams/attachment-sync.mjs'
import { mimeForFile } from '../lib/atlassian/attachments.mjs'

test('sha256File matches the known vector for "abc"', () => {
  const dir = mkdtempSync(join(tmpdir(), 'attsync-'))
  try {
    const f = join(dir, 'abc.txt')
    writeFileSync(f, 'abc')
    assert.equal(
      sha256File(f),
      'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
    )
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
})

test('formatComment / parseHash round-trip', () => {
  const hex = 'a'.repeat(64)
  const comment = formatComment(hex)
  assert.equal(comment, `${HASH_PREFIX}${hex}`)
  assert.equal(parseHash(comment), hex)
})

test('parseHash is case-insensitive and normalizes to lowercase', () => {
  const hex = 'A'.repeat(64)
  assert.equal(parseHash(`sha256:${hex}`), 'a'.repeat(64))
})

test('parseHash returns null for non-hash comments', () => {
  assert.equal(parseHash('draw.io diagram'), null)
  assert.equal(parseHash(''), null)
  assert.equal(parseHash(undefined), null)
  assert.equal(parseHash(null), null)
  assert.equal(parseHash('sha256:tooshort'), null)
})

test('decideAction: no attachment -> upload', () => {
  assert.equal(decideAction({ exists: false, localHash: 'x', storedHash: null }), 'upload')
})

test('decideAction: same hash -> skip', () => {
  assert.equal(decideAction({ exists: true, localHash: 'abc', storedHash: 'abc' }), 'skip')
})

test('decideAction: different hash -> update', () => {
  assert.equal(decideAction({ exists: true, localHash: 'abc', storedHash: 'def' }), 'update')
})

test('decideAction: existing attachment with unreadable hash -> update', () => {
  assert.equal(decideAction({ exists: true, localHash: 'abc', storedHash: null }), 'update')
})

test('decidePrune: managed orphan -> prune, unmanaged orphan -> warn, referenced -> keep', () => {
  const attachments = [
    { id: 'a1', title: 'kept.png', managed: true }, // referenced -> keep
    { id: 'a2', title: 'gone.png', managed: true }, // managed orphan -> prune
    { id: 'a3', title: 'client.png', managed: false }, // unmanaged orphan -> warn
  ]
  const { prune, warnings } = decidePrune({ attachments, keepTitles: ['kept.png'] })
  assert.deepEqual(prune, [{ id: 'a2', title: 'gone.png' }])
  assert.deepEqual(warnings, ['client.png'])
})

test('decidePrune: nothing to prune when every attachment is still referenced', () => {
  const attachments = [
    { id: 'a1', title: 'one.png', managed: true },
    { id: 'a2', title: 'two.drawio', managed: true },
  ]
  const { prune, warnings } = decidePrune({
    attachments,
    keepTitles: ['one.png', 'two.drawio'],
  })
  assert.deepEqual(prune, [])
  assert.deepEqual(warnings, [])
})

test('decidePrune: tolerates empty/undefined attachment list', () => {
  assert.deepEqual(decidePrune({ attachments: undefined, keepTitles: [] }), {
    prune: [],
    warnings: [],
  })
})

test('mimeForFile maps common types and falls back to octet-stream', () => {
  assert.equal(mimeForFile('diagram.png'), 'image/png')
  assert.equal(mimeForFile('PHOTO.JPG'), 'image/jpeg')
  assert.equal(mimeForFile('a.jpeg'), 'image/jpeg')
  assert.equal(mimeForFile('a.svg'), 'image/svg+xml')
  assert.equal(mimeForFile('a.gif'), 'image/gif')
  assert.equal(mimeForFile('flow.drawio'), 'application/vnd.jgraph.mxfile')
  assert.equal(mimeForFile('data.bin'), 'application/octet-stream')
})
