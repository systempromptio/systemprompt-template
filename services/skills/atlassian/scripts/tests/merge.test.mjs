import { test } from 'node:test'
import assert from 'node:assert/strict'

import { splitHeadBody, mergeDocBody, chromeDrift } from '../lib/doc/merge.mjs'
import { getDocType } from '../lib/doc/types/index.mjs'

const BODY_SECTIONS = getDocType('fsd').bodySections

// Build a canonical FSD document with adjustable chrome + body so tests can vary
// one axis at a time (chrome vs body) and prove the merge is body-scoped.
function fsd({ status = 'on review', pageId = '123', approvals = [], body }) {
  const lines = [
    '# Store Locator FSD',
    '',
    `- Confluence: https://wiki.example/pages/${pageId}`,
    `- Page ID: ${pageId}`,
    '',
    '## General FSD Information',
    '',
    '|  |  |',
    '| --- | --- |',
    '| WBS-Feature Name | Store Locator |',
    `| FSD Status | ${status} |`,
  ]
  if (approvals.length) {
    lines.push('', '### Approvals', '', '|  |  |  |', '| --- | --- | --- |')
    for (const row of approvals) lines.push(`| ${row} |`)
  }
  lines.push('', body.trim(), '')
  return lines.join('\n')
}

const BASE_BODY = [
  '## In Scope Functional Requirements',
  '',
  '### RQ-1 - Alpha',
  '',
  'Original alpha.',
  '',
  '### RQ-2 - Beta',
  '',
  'Original beta.',
].join('\n')

// ─── splitHeadBody ───────────────────────────────────────────────────────────

test('splitHeadBody cuts at the first body-section H2 and keeps meta in the head', () => {
  const doc = fsd({ body: BASE_BODY })
  const { head, body, found } = splitHeadBody(doc, BODY_SECTIONS)
  assert.equal(found, true)
  assert.match(head, /## General FSD Information/)
  assert.match(head, /- Page ID: 123/, 'meta stays in the head (model would drop it)')
  assert.doesNotMatch(head, /In Scope Functional Requirements/)
  assert.match(body, /^## In Scope Functional Requirements/)
})

test('splitHeadBody reports found=false when no body section is present', () => {
  const doc = ['# Title', '', '## General FSD Information', '', '| a | b |'].join('\n')
  const { body, found } = splitHeadBody(doc, BODY_SECTIONS)
  assert.equal(found, false)
  assert.equal(body, '')
})

// ─── Clean, non-overlapping body merges ──────────────────────────────────────

test('clean 3-way: independent local + wiki body edits both apply, no conflict', () => {
  const base = fsd({ body: BASE_BODY })
  const ours = fsd({ body: BASE_BODY.replace('Original alpha.', 'Local alpha edit.') })
  const theirs = fsd({ body: BASE_BODY.replace('Original beta.', 'Wiki beta edit.') })

  const { merged, conflicts } = mergeDocBody({ base, ours, theirs, bodySections: BODY_SECTIONS })
  assert.equal(conflicts, 0)
  assert.match(merged, /Local alpha edit\./)
  assert.match(merged, /Wiki beta edit\./)
  assert.doesNotMatch(merged, /<<<<<<</)
})

// ─── Genuine overlap → conflict markers in the working copy ───────────────────

test('conflicting edits to the same body line produce diff3 conflict markers', () => {
  const base = fsd({ body: BASE_BODY })
  const ours = fsd({ body: BASE_BODY.replace('Original alpha.', 'Local wins.') })
  const theirs = fsd({ body: BASE_BODY.replace('Original alpha.', 'Wiki wins.') })

  const { merged, conflicts } = mergeDocBody({ base, ours, theirs, bodySections: BODY_SECTIONS })
  assert.ok(conflicts > 0, 'reports at least one conflict hunk')
  assert.match(merged, /<<<<<<< working copy/)
  assert.match(merged, /\|\|\|\|\|\|\| base \(discovery baseline\)/)
  assert.match(merged, />>>>>>> confluence \(live\)/)
  assert.match(merged, /Local wins\./)
  assert.match(merged, /Wiki wins\./)
})

// ─── Body-scoping: chrome differences never conflict ─────────────────────────

test('chrome differing across base/ours/theirs does NOT conflict; ours chrome is kept', () => {
  const base = fsd({ status: 'draft', pageId: '111', body: BASE_BODY })
  const ours = fsd({ status: 'on review', pageId: '999', body: BASE_BODY })
  const theirs = fsd({
    status: 'approved',
    pageId: '111',
    approvals: ['Jane Doe | QA | approved'],
    body: BASE_BODY,
  })

  const { merged, conflicts, chromeChanges } = mergeDocBody({
    base, ours, theirs, bodySections: BODY_SECTIONS,
  })
  assert.equal(conflicts, 0, 'chrome is never fed to the text merge')
  assert.doesNotMatch(merged, /<<<<<<</)
  assert.match(merged, /FSD Status \| on review/, 'status kept from ours')
  assert.match(merged, /Page ID: 999/, 'meta kept from ours')
  assert.doesNotMatch(merged, /FSD Status \| approved/, 'wiki status is not merged in')
  // Wiki-side chrome edit is surfaced for the human to port, not silently merged.
  assert.ok(chromeChanges.some((l) => /Jane Doe/.test(l)), 'wiki approval surfaced as drift')
  assert.doesNotMatch(merged, /Jane Doe/, 'wiki approval is not auto-merged into the body/chrome')
})

// ─── Idempotency: no-op republish makes no change ────────────────────────────

test('idempotent: ours == theirs (already synced) yields no changes, no conflict', () => {
  const base = fsd({ body: BASE_BODY })
  const synced = fsd({ body: BASE_BODY.replace('Original alpha.', 'Shared edit.') })

  const { merged, conflicts } = mergeDocBody({
    base, ours: synced, theirs: synced, bodySections: BODY_SECTIONS,
  })
  assert.equal(conflicts, 0)
  assert.equal(merged.trim(), synced.trim(), 'merged equals the working copy (no needless churn)')
})

test('idempotent: identical base/ours/theirs round-trips the working copy unchanged', () => {
  const doc = fsd({ body: BASE_BODY })
  const { merged, conflicts, chromeChanges } = mergeDocBody({
    base: doc, ours: doc, theirs: doc, bodySections: BODY_SECTIONS,
  })
  assert.equal(conflicts, 0)
  assert.equal(chromeChanges.length, 0)
  assert.equal(merged.trim(), doc.trim())
})

// ─── chromeDrift ─────────────────────────────────────────────────────────────

test('chromeDrift surfaces wiki-added chrome lines and ignores instance meta', () => {
  const base = fsd({ status: 'draft', pageId: '111', body: BASE_BODY })
  const theirs = fsd({
    status: 'draft',
    pageId: '222', // different meta → must be ignored
    approvals: ['Jane Doe | QA | approved'],
    body: BASE_BODY,
  })
  const { head: baseHead } = splitHeadBody(base, BODY_SECTIONS)
  const { head: theirsHead } = splitHeadBody(theirs, BODY_SECTIONS)

  const drift = chromeDrift(baseHead, theirsHead)
  assert.ok(drift.some((l) => /Jane Doe/.test(l)), 'new approval row is drift')
  assert.ok(!drift.some((l) => /Page ID/i.test(l)), 'meta lines are never drift')
  assert.ok(!drift.some((l) => /Confluence:/i.test(l)), 'meta lines are never drift')
})

// ─── Guard ───────────────────────────────────────────────────────────────────

test('mergeDocBody throws when the working copy has no body section', () => {
  const noBody = ['# Title', '', '## General FSD Information', '', '| a | b |'].join('\n')
  const ok = fsd({ body: BASE_BODY })
  assert.throws(
    () => mergeDocBody({ base: ok, ours: noBody, theirs: ok, bodySections: BODY_SECTIONS }),
    /no body section/i,
  )
})
