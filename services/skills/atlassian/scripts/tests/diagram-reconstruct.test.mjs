import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, existsSync, writeFileSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { reconstructDiagramsInDoc } from '../lib/diagrams/reconstruct.mjs'
import { REVERSE_CLI } from '../lib/diagrams/reverse-cli.mjs'

// A self-describing `.drawio`: reverse.mjs only needs the embedded base64 spec,
// not real draw.io geometry.
function drawioWithSpec(specYaml) {
  const b64 = Buffer.from(specYaml, 'utf8').toString('base64')
  return `<mxfile><diagram data-spec-format="drawio-diagrams/v1" data-spec="${b64}"></diagram></mxfile>`
}

const SPEC = 'type: sequence\nid: flow\ntitle: My Flow\nparticipants:\n  - { id: a, title: A }\n'

function makeStubDownload(record) {
  return async (pageId, att, destPath) => {
    record.push({ pageId, title: att.title, destPath })
    if (att.title.endsWith('.drawio')) writeFileSync(destPath, drawioWithSpec(SPEC))
    else writeFileSync(destPath, Buffer.from([0x89, 0x50, 0x4e, 0x47]))
    return { fileName: att.title, destPath }
  }
}

test('reconstructDiagramsInDoc downloads sources, reverses, and replaces the flat image in place', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'recon-'))
  const assetsDir = join(dir, 'assets', '123')
  const attachments = [
    { title: 'flow.drawio', id: 'att1' },
    { title: 'flow.png', id: 'att2' },
  ]
  const downloaded = []

  const md0 = '# Doc\n\nIntro.\n\n![Flow](../../assets/123/flow.png)\n\nOutro.\n'
  const { md, pulled, diagramPngTitles } = await reconstructDiagramsInDoc({
    md: md0,
    pageId: '123',
    attachments,
    assetsDir,
    imageRelPrefix: '../../assets/123',
    downloadAttachment: makeStubDownload(downloaded),
    reverseCli: REVERSE_CLI,
  })

  assert.deepEqual(pulled, ['flow'])
  assert.ok(diagramPngTitles.has('flow.png'))
  // Both sources landed on disk under clean slug names.
  assert.ok(existsSync(join(assetsDir, 'flow.drawio')))
  assert.ok(existsSync(join(assetsDir, 'flow.png')))
  // Editable block spliced in place of the flat image (before it), no duplication.
  assert.equal((md.match(/```drawio:sequence:flow/g) || []).length, 1)
  assert.equal((md.match(/!\[[^\]]*\]\(\.\.\/\.\.\/assets\/123\/flow\.png\)/g) || []).length, 1)
  assert.ok(md.indexOf('```drawio:sequence:flow') < md.indexOf('](../../assets/123/flow.png)'))
  assert.ok(md.includes('Intro.'))
  assert.ok(md.includes('Outro.'))
})

test('reconstructDiagramsInDoc skips an existing png when skipExistingPng is set', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'recon-'))
  const assetsDir = join(dir, 'assets', '9')
  const attachments = [
    { title: 'flow.drawio', id: 'att1' },
    { title: 'flow.png', id: 'att2' },
  ]

  // First run creates the assets dir + the clean png (like the export localize step).
  const downloaded = []
  await reconstructDiagramsInDoc({
    md: '![Flow](./assets/flow.png)\n',
    pageId: '9',
    attachments,
    assetsDir,
    imageRelPrefix: './assets',
    downloadAttachment: makeStubDownload(downloaded),
    reverseCli: REVERSE_CLI,
    skipExistingPng: false,
  })

  // Pretend the on-disk png is the canonical bytes we don't want re-fetched.
  const preExisting = Buffer.from('already-here')
  writeFileSync(join(assetsDir, 'flow.png'), preExisting)

  const downloaded2 = []
  await reconstructDiagramsInDoc({
    md: '![Flow](./assets/flow.png)\n',
    pageId: '9',
    attachments,
    assetsDir,
    imageRelPrefix: './assets',
    downloadAttachment: makeStubDownload(downloaded2),
    reverseCli: REVERSE_CLI,
    skipExistingPng: true,
  })

  // .drawio re-downloaded, but the pre-existing png was left untouched.
  assert.ok(downloaded2.some((d) => d.title === 'flow.drawio'))
  assert.ok(!downloaded2.some((d) => d.title === 'flow.png'))
  assert.deepEqual(readFileSync(join(assetsDir, 'flow.png')), preExisting)
})

test('reconstructDiagramsInDoc is a no-op when the page has no .drawio attachments', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'recon-'))
  const attachments = [{ title: 'photo.png', id: 'att1' }]
  const downloaded = []
  const md0 = '# Doc\n\n![Photo](./assets/photo.png)\n'
  const { md, pulled, diagramPngTitles } = await reconstructDiagramsInDoc({
    md: md0,
    pageId: '5',
    attachments,
    assetsDir: join(dir, 'assets', '5'),
    imageRelPrefix: './assets',
    downloadAttachment: makeStubDownload(downloaded),
    reverseCli: REVERSE_CLI,
  })
  assert.equal(md, md0)
  assert.deepEqual(pulled, [])
  assert.equal(diagramPngTitles.size, 0)
  assert.equal(downloaded.length, 0)
})
