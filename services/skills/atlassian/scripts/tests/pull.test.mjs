/**
 * Shared typed reverse-pull core (`lib/doc/pull.mjs`) tests.
 *
 * The core wraps `storageToDoc` (already covered by storage-to-doc.test.mjs) and
 * adds the one bit of I/O both callers share: download the body's non-diagram
 * images into the assets dir while SKIPPING the generated-diagram PNGs (which
 * reconstruct.mjs owns). We assert that split, the account-id resolution pass,
 * and that the markdown matches the type-free driver's output.
 */

import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, existsSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { pullTypedMarkdown } from '../lib/doc/pull.mjs'
import { storageToDoc } from '../lib/doc/storage-to-doc.mjs'

const STORAGE = `<ac:layout><ac:layout-cell><table><tbody>
<tr><th colspan="3"><p><strong>General ISD Information</strong></p></th></tr>
<tr><td><p>Author/Owner</p></td><td colspan="2"><p><ac:link><ri:user ri:account-id="id-9"/></ac:link></p></td></tr>
</tbody></table></ac:layout-cell></ac:layout><hr/>
<h2>Integration Specification</h2>
<p><ac:image ac:alt="Flow"><ri:attachment ri:filename="flow.png"/></ac:image></p>
<p><ac:image ac:alt="Arch"><ri:attachment ri:filename="arch.png"/></ac:image></p>`

// arch.png/.drawio is a generated diagram pair; flow.png is a plain body image.
const ATTACHMENTS = [
  { title: 'flow.png', downloadLink: '/download/flow.png' },
  { title: 'arch.png', downloadLink: '/download/arch.png' },
  { title: 'arch.drawio', downloadLink: '/download/arch.drawio' },
]

function fakeDownloader(downloaded) {
  return async (_pageId, att, destPath) => {
    downloaded.push(att.title)
    writeFileSync(destPath, 'x')
    return { fileName: att.title, destPath }
  }
}

test('pullTypedMarkdown downloads body images but skips diagram PNGs', async () => {
  const assetsDir = mkdtempSync(join(tmpdir(), 'pull-core-'))
  const downloaded = []
  const seenIds = []

  const { markdown, imageFilenames, diagramPngTitles } = await pullTypedMarkdown({
    type: 'isd',
    pageId: '123',
    storageXhtml: STORAGE,
    title: 'Payment Gateway ISD',
    imageRelPrefix: './assets',
    assetsDir,
    attachments: ATTACHMENTS,
    downloadAttachment: fakeDownloader(downloaded),
    resolveAccountId: (id) => {
      seenIds.push(id)
      return 'Nina Analyst'
    },
  })

  // The generated-diagram PNG is recognised from its `.drawio` sibling and left
  // to reconstruct.mjs; only the plain body image is fetched here.
  assert.deepEqual([...diagramPngTitles], ['arch.png'])
  assert.deepEqual(downloaded, ['flow.png'])
  assert.ok(existsSync(join(assetsDir, 'flow.png')))
  assert.ok(!existsSync(join(assetsDir, 'arch.png')))

  // Both images are still referenced in the markdown under `./assets`.
  assert.match(markdown, /!\[Flow\]\(\.\/assets\/flow\.png\)/)
  assert.match(markdown, /!\[Arch\]\(\.\/assets\/arch\.png\)/)
  assert.deepEqual(imageFilenames, ['flow.png', 'arch.png'])

  // Account ids are resolved to display names (reverse of the publish mentionMap).
  assert.deepEqual(seenIds, ['id-9'])
  assert.match(markdown, /Nina Analyst/)
})

test('pullTypedMarkdown markdown equals the type-free storageToDoc output', async () => {
  const assetsDir = mkdtempSync(join(tmpdir(), 'pull-core-'))
  const { markdown } = await pullTypedMarkdown({
    type: 'isd',
    pageId: '123',
    storageXhtml: STORAGE,
    title: 'Payment Gateway ISD',
    imageRelPrefix: './assets',
    assetsDir,
    attachments: ATTACHMENTS,
    downloadAttachment: fakeDownloader([]),
    resolveAccountId: () => 'Nina Analyst',
  })

  const { markdown: viaDriver } = await storageToDoc({
    type: 'isd',
    storageXhtml: STORAGE,
    title: 'Payment Gateway ISD',
    imageRelPrefix: './assets',
    resolveAccountId: () => 'Nina Analyst',
  })

  assert.equal(markdown, viaDriver)
})
