import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { parseArgs } from '../lib/util/cli-args.mjs'
import { escHtml, escAttr, escapeRegExp } from '../lib/util/xhtml.mjs'
import {
  STATUS_VALUES,
  CLIENT_REVIEW,
  statusColour,
  isStatus,
  statusValues,
  statusColours,
  deriveClientName,
} from '../lib/doc/status-vocab.mjs'
import { parseExportHeader, stripNonContentHtml, absolutizeRootRelativeUrls } from '../lib/atlassian/html-to-markdown.mjs'
import { readDocMeta, stampDocMeta } from '../lib/doc/meta.mjs'

// ─── cli-args ──────────────────────────────────────────────────────────────────

test('parseArgs handles --key value, --key=value, positionals', () => {
  const { flags, positional } = parseArgs(['file.md', '--title', 'Hello', '--type=fsd', 'extra'])
  assert.equal(flags.title, 'Hello')
  assert.equal(flags.type, 'fsd')
  assert.deepEqual(positional, ['file.md', 'extra'])
})

test('parseArgs treats listed booleans as flags even before a value', () => {
  const { flags, positional } = parseArgs(['--dry', 'file.md'], { booleans: ['dry'] })
  assert.equal(flags.dry, true)
  assert.deepEqual(positional, ['file.md'])
})

test('parseArgs accumulates repeatable flags into an array (the --mention fix)', () => {
  const { flags } = parseArgs(['--mention', 'A=1', '--mention', 'B=2'], { repeatable: ['mention'] })
  assert.deepEqual(flags.mention, ['A=1', 'B=2'])
})

test('parseArgs: a repeatable flag with a single occurrence is still an array', () => {
  const { flags } = parseArgs(['--dir', 'x'], { repeatable: ['dir'] })
  assert.deepEqual(flags.dir, ['x'])
})

test('parseArgs: a value-less trailing flag becomes boolean true', () => {
  const { flags } = parseArgs(['--body-only'])
  assert.equal(flags['body-only'], true)
})

// ─── xhtml ─────────────────────────────────────────────────────────────────────

test('escHtml escapes &, <, > and leaves punctuation as authored', () => {
  assert.equal(escHtml('a & b < c > d'), 'a &amp; b &lt; c &gt; d')
  assert.equal(escHtml('x \u2014 y \u2013 z'), 'x \u2014 y \u2013 z')
})

test('escAttr also escapes double quotes', () => {
  assert.equal(escAttr('say "hi" & <b>'), 'say &quot;hi&quot; &amp; &lt;b&gt;')
})

test('escHtml and escapeRegExp are null-safe', () => {
  assert.equal(escHtml(null), '')
  assert.equal(escapeRegExp('a.b*c(d)'), 'a\\.b\\*c\\(d\\)')
})

// ─── status-vocab ────────────────────────────────────────────────────────────

test('statusColour maps canonical words and falls back to Grey', () => {
  assert.equal(statusColour('in progress'), 'Yellow')
  assert.equal(statusColour('on review'), 'Blue', 'no longer drifts to Yellow')
  assert.equal(statusColour('in review'), 'Blue')
  assert.equal(statusColour('approved'), 'Green')
  assert.equal(statusColour('banana'), 'Grey')
})

test('isStatus is case-insensitive, trims, and matches the vocabulary', () => {
  assert.equal(isStatus('  Approved '), true)
  assert.equal(isStatus('DRAFT'), true)
  assert.equal(isStatus('nope'), false)
  for (const s of STATUS_VALUES) assert.equal(isStatus(s), true)
})

test('the client review word resolves against the document, not the library', () => {
  assert.equal(isStatus('Acme Retail review'), false, 'unknown without a client')
  assert.equal(isStatus('Acme Retail review', 'Acme Retail'), true)
  assert.equal(statusColour('Acme Retail review', 'Acme Retail'), 'Purple')
  assert.equal(statusColour('acme retail REVIEW', 'acme retail'), 'Purple')
  assert.equal(
    isStatus('Acme review', 'Acme Retail'),
    false,
    'a drifted spelling is not the client review word',
  )
  assert.ok(statusValues('Acme Retail').includes('acme retail review'))
  assert.ok(!statusValues('Acme Retail').includes(CLIENT_REVIEW), 'placeholder never leaks')
  assert.equal(statusColours('Acme Retail')['acme retail review'], 'Purple')
})

test('deriveClientName takes the first non-Astound approval group', () => {
  const model = {
    header: {
      sections: [
        { label: 'General FSD Information', rows: [] },
        { label: 'Astound Approval', rows: [] },
        { label: 'Acme Retail Approval', rows: [] },
      ],
    },
  }
  assert.equal(deriveClientName(model), 'Acme Retail')
  assert.equal(deriveClientName({ header: { sections: [{ label: 'Card' }] } }), 'Client')
})

// ─── html-to-markdown ────────────────────────────────────────────────────────

test('parseExportHeader extracts title, pageId, version and body', () => {
  const md = [
    '# My Page',
    '- Confluence: https://x/wiki/pages/42',
    '- Page ID: 42',
    '- Version: 7',
    '- Updated: 2026-01-01',
    '',
    'Body starts here.',
  ].join('\n')
  const { title, pageId, version, body } = parseExportHeader(md)
  assert.equal(title, 'My Page')
  assert.equal(pageId, '42')
  assert.equal(version, 7)
  assert.equal(body, 'Body starts here.')
})

test('stripNonContentHtml removes head/style/script; absolutize rewrites root-relative urls', () => {
  const html = '<head><style>x{}</style></head><a href="/wiki/x">l</a><img src="/i.png">'
  const stripped = stripNonContentHtml(html)
  assert.equal(/<head|<style|<script/.test(stripped), false)
  const abs = absolutizeRootRelativeUrls('<a href="/wiki/x">l</a>', 'https://acme.example')
  assert.match(abs, /href="https:\/\/acme\.example\/wiki\/x"/)
})

// ─── doc-meta ─────────────────────────────────────────────────────────────────

test('stampDocMeta inserts and readDocMeta reads back; re-stamp is idempotent', () => {
  const dir = mkdtempSync(join(tmpdir(), 'docmeta-'))
  try {
    const f = join(dir, 'doc.md')
    writeFileSync(f, '# Title\n\n## General FSD Information\n\nbody\n')
    stampDocMeta(f, { url: 'https://x/wiki/pages/9', pageId: '9' })
    const meta = readDocMeta(readFileSync(f, 'utf8'))
    assert.equal(meta.url, 'https://x/wiki/pages/9')
    assert.equal(meta.pageId, '9')

    const before = readFileSync(f, 'utf8')
    const res = stampDocMeta(f, { url: 'https://x/wiki/pages/9', pageId: '9' })
    assert.equal(res.changed, false, 're-stamping identical meta does not rewrite')
    assert.equal(readFileSync(f, 'utf8'), before, 'no duplicate meta lines added')
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
})
