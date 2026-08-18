/**
 * The extensibility seam: the type registry + the type-agnostic spec envelope. This is the
 * contract a future `activity` type must satisfy, so it's what the "agnosticism" claim rests on.
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { getType, listTypes } from '../types/index.mjs'
import { validateSpec, slugify, extractDrawioBlocks } from '../lib/spec.mjs'
import { validSpec, assertHasError, assertNoErrors } from './_fixtures.mjs'

test('registry exposes each type through a uniform interface', () => {
  assert.ok(listTypes().includes('sequence'))
  const t = getType('sequence')
  assert.equal(t.type, 'sequence')
  assert.equal(typeof t.title, 'string')
  for (const fn of ['validate', 'layout', 'emit']) {
    assert.equal(typeof t[fn], 'function', `type.${fn} must be a function`)
  }
  assert.equal(getType('does-not-exist'), null)
})

test('validateSpec checks the envelope then delegates to the type', () => {
  assertHasError(validateSpec({ id: 'x', participants: [], messages: [] }), 'type: required')
  assertHasError(validateSpec({ type: 'activity', id: 'x' }), 'unknown "activity"')
  assertHasError(validateSpec({ type: 'sequence', participants: [], messages: [] }), 'id: required')
  assertHasError(validateSpec('nope'), 'must be a YAML mapping')
  // A valid spec passes the envelope AND the delegated type validation.
  assertNoErrors(validateSpec(validSpec()))
})

test('slugify produces filesystem-safe slugs', () => {
  assert.equal(slugify('Consent Manager!'), 'consent-manager')
  assert.equal(slugify('  --Edge/SSR--  '), 'edge-ssr')
  assert.equal(slugify(''), 'diagram')
})

test('extractDrawioBlocks parses header type/id and preserves the scenario body', () => {
  const md = [
    '# Doc',
    '',
    '```drawio:sequence:integration-flow',
    'participants:',
    '  - { id: a, title: A }',
    '```',
    '',
    'prose',
    '',
    '```drawio:sequence:second-flow',
    'messages:',
    '  - { kind: call, from: a, to: b, text: hi }',
    '```',
  ].join('\n')
  const blocks = extractDrawioBlocks(md)
  assert.equal(blocks.length, 2)
  assert.equal(blocks[0].type, 'sequence')
  assert.equal(blocks[0].id, 'integration-flow')
  assert.ok(blocks[0].body.includes('participants:'))
  assert.equal(blocks[1].id, 'second-flow')
  assert.ok(blocks[1].body.includes('messages:'))
  // Header carries type/id; the body must NOT repeat them.
  assert.ok(!blocks[0].body.includes('type:'))
})
