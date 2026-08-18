/**
 * Pure stories.md helpers: doc-link extraction (used to back-link a Story to its
 * FSD/ISD page) and the Confluence remote-link payload builder.
 */

import { test } from 'node:test'
import assert from 'node:assert/strict'

import { extractDocLink, buildConfluenceRemoteLink } from '../lib/jira/stories-md.mjs'

test('extractDocLink pulls url + pageId + title from an Acceptance criteria deep link', () => {
  const block = [
    '### TBD-1 — 3PI.CON — Consent Management',
    '',
    '**Acceptance criteria:** [3PI.CON — Acceptance Criteria](https://astounddigital.atlassian.net/wiki/spaces/Charles/pages/1262551085/Consent+Manager+Integration+ISD#Acceptance-Criteria)',
    '',
  ].join('\n')

  assert.deepEqual(extractDocLink(block), {
    url: 'https://astounddigital.atlassian.net/wiki/spaces/Charles/pages/1262551085/Consent+Manager+Integration+ISD',
    pageId: '1262551085',
    title: 'Consent Manager Integration ISD',
  })
})

test('extractDocLink falls back to a FSD/ISD reference link', () => {
  const block = [
    '### TBD-2 — WBS-12 — Store Locator',
    '',
    '**References:**',
    '',
    '- FSD: [Store Locator](https://acme.atlassian.net/wiki/spaces/DEV/pages/98765/Store+Locator+FSD)',
  ].join('\n')

  assert.deepEqual(extractDocLink(block), {
    url: 'https://acme.atlassian.net/wiki/spaces/DEV/pages/98765/Store+Locator+FSD',
    pageId: '98765',
    title: 'Store Locator FSD',
  })
})

test('extractDocLink returns nulls for an unpublished (local file) reference', () => {
  const block = [
    '### TBD-3 — WBS-9 — Something',
    '',
    '- FSD: [feature](../fsd/feature.md#Acceptance-Criteria)',
  ].join('\n')

  assert.deepEqual(extractDocLink(block), { url: null, pageId: null, title: '' })
})

test('buildConfluenceRemoteLink emits a Confluence-typed link with the globalId', () => {
  const payload = buildConfluenceRemoteLink({
    appId: 'abc-123',
    pageId: '1262551085',
    url: 'https://x.atlassian.net/wiki/spaces/K/pages/1262551085/Doc',
    title: 'Doc',
  })

  assert.equal(payload.globalId, 'appId=abc-123&pageId=1262551085')
  assert.equal(payload.application.type, 'com.atlassian.confluence')
  assert.deepEqual(payload.object, {
    url: 'https://x.atlassian.net/wiki/spaces/K/pages/1262551085/Doc',
    title: 'Doc',
  })
})

test('buildConfluenceRemoteLink falls back to the url as title', () => {
  const payload = buildConfluenceRemoteLink({ appId: 'a', pageId: '1', url: 'https://x/p', title: '' })
  assert.equal(payload.object.title, 'https://x/p')
})
