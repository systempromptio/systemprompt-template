import { test } from 'node:test'
import assert from 'node:assert/strict'

import { referencedLocalAssetNames, planAssetLocalization } from '../lib/doc/assets.mjs'

test('referencedLocalAssetNames: keeps local basenames, drops remote/data URIs, dedupes', () => {
  const names = referencedLocalAssetNames([
    './assets/flow.png',
    'assets/flow.png', // same basename -> deduped
    'https://example.com/remote.png', // remote -> dropped
    'http://example.com/x.png', // remote -> dropped
    'data:image/png;base64,AAAA', // inline -> dropped
    './assets/photo.jpg',
  ])
  assert.deepEqual(names, ['flow.png', 'photo.jpg'])
})

test('planAssetLocalization: staged-but-not-local -> localize (wiki-added image)', () => {
  const plan = planAssetLocalization({
    referenced: ['wiki.png'],
    present: [],
    staged: ['wiki.png'],
  })
  assert.deepEqual(plan, { localize: ['wiki.png'], dangling: [] })
})

test('planAssetLocalization: referenced but nowhere -> dangling (warn)', () => {
  const plan = planAssetLocalization({
    referenced: ['ghost.png'],
    present: [],
    staged: [],
  })
  assert.deepEqual(plan, { localize: [], dangling: ['ghost.png'] })
})

test('planAssetLocalization: already-local (possibly edited) image is left untouched', () => {
  const plan = planAssetLocalization({
    referenced: ['mine.png'],
    present: ['mine.png'], // exists locally -> never overwritten from staging
    staged: ['mine.png'],
  })
  assert.deepEqual(plan, { localize: [], dangling: [] })
})

test('planAssetLocalization: mixed set partitions correctly', () => {
  const plan = planAssetLocalization({
    referenced: ['local.png', 'fromwiki.png', 'missing.png'],
    present: ['local.png'],
    staged: ['fromwiki.png'],
  })
  assert.deepEqual(plan, { localize: ['fromwiki.png'], dangling: ['missing.png'] })
})
