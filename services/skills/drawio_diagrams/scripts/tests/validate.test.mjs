/**
 * Validation contract — the spec's rules R1–R6 plus structural checks. These encode the
 * authoring contract and MUST survive the refactor unchanged.
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { validate, warnings } from '../types/sequence/validate/index.mjs'
import { validSpec, assertHasError, assertNoErrors } from './_fixtures.mjs'

/** Assert that no error string contains `needle`. */
function assertNoErrorMatching(errors, needle) {
  assert.ok(
    !errors.some((e) => e.includes(needle)),
    `expected NO error containing ${JSON.stringify(needle)}, got:\n  ${errors.join('\n  ')}`,
  )
}

test('a strictly-valid spec produces no errors', () => {
  assertNoErrors(validate(validSpec()))
})

test('participants: structural checks', () => {
  assertHasError(validate(validSpec({ participants: [] })), 'at least one participant')
  assertHasError(
    validate(validSpec({ participants: [{ id: 'x', title: 'X' }, { id: 'x', title: 'Y' }] })),
    'duplicate id',
  )
  assertHasError(
    validate(validSpec({ participants: [{ id: 'x', title: 'X', kind: 'blob' }] })),
    'kind must be',
  )
})

test('R3: participant text budgets are soft (warnings); stale `label` stays an error', () => {
  const longTitle = validSpec({
    participants: [{ id: 'x', title: 'This title is far too long' }],
    messages: [],
    notes: [],
  })
  assertNoErrorMatching(validate(longTitle), 'header box fits')
  assertHasError(warnings(longTitle), 'header box fits')

  const longSub = validSpec({
    participants: [{ id: 'x', title: 'X', subtitle: 'this subtitle is beyond the limit' }],
    messages: [],
    notes: [],
  })
  assertHasError(warnings(longSub), 'chars; it fits')

  // The stale `label` field is a structural error, not a budget — it stays a hard error.
  assertHasError(validate(validSpec({ participants: [{ id: 'x', label: 'Old' }] })), 'renamed to "title"')
})

test('messages: structural checks', () => {
  assertHasError(validate(validSpec({ messages: [{ kind: 'nope', from: 'u', to: 'a' }] })), 'kind must be one of')
  assertHasError(validate(validSpec({ messages: [{ kind: 'call', from: 'zzz', to: 'a' }] })), 'unknown "from"')
  assertHasError(
    validate(validSpec({ messages: [{ kind: 'self', from: 'a', to: 'b' }] })),
    'self message "to" must equal "from"',
  )
})

test('R1: only the active flow may act (one call in flight)', () => {
  // `a` calls `b`, then tries to call `c` before `b` returns -> a is not the active flow.
  const spec = validSpec({
    participants: [
      { id: 'u', title: 'U', kind: 'actor' },
      { id: 'a', title: 'A' },
      { id: 'b', title: 'B' },
      { id: 'c', title: 'C' },
    ],
    messages: [
      { kind: 'call', from: 'u', to: 'a', text: 'go' },
      { kind: 'call', from: 'a', to: 'b', text: 'x' },
      { kind: 'call', from: 'a', to: 'c', text: 'y' },
      { kind: 'return', from: 'b', to: 'a', text: 'x' },
      { kind: 'return', from: 'c', to: 'a', text: 'y' },
      { kind: 'return', from: 'a', to: 'u', text: 'ok' },
    ],
    notes: [],
  })
  assertHasError(validate(spec), 'R1')
})

test('R2: every call must be closed by a return', () => {
  const spec = validSpec({
    messages: [{ kind: 'call', from: 'u', to: 'a', text: 'go' }],
    notes: [],
  })
  assertHasError(validate(spec), 'R2')
})

test('R4: message label budgets are soft (warnings)', () => {
  const longCall = validSpec({ messages: [{ kind: 'call', from: 'u', to: 'a', text: 'x'.repeat(21) }], notes: [] })
  assertNoErrorMatching(validate(longCall), 'message label')
  assertHasError(warnings(longCall), 'message label')

  const longSelf = validSpec({ messages: [{ kind: 'self', from: 'a', text: 'x'.repeat(41) }], notes: [] })
  assertHasError(warnings(longSelf), 'self label')
})

test('R6: a call cannot go right-to-left (only a return may)', () => {
  // `a` (index 1) tries to call `u` (index 0) — a leftward call.
  const leftwardCall = validSpec({
    messages: [
      { kind: 'call', from: 'u', to: 'a', text: 'go' },
      { kind: 'call', from: 'a', to: 'u', text: 'back' },
      { kind: 'return', from: 'a', to: 'u', text: 'ok' },
    ],
    notes: [],
  })
  assertHasError(validate(leftwardCall), 'R6')

  // A `return` that hands control rightward is also rejected.
  const rightwardReturn = validSpec({
    messages: [
      { kind: 'call', from: 'u', to: 'a', text: 'go' },
      { kind: 'return', from: 'u', to: 'a', text: 'nope' },
    ],
    notes: [],
  })
  assertHasError(validate(rightwardReturn), 'R6')
})

test('R5: note anchoring stays an error; note length is a soft warning', () => {
  // Length over budget -> warning, not error.
  const longNote = validSpec({ notes: [{ under: 'b', text: 'x'.repeat(71) }] })
  assertNoErrorMatching(validate(longNote), 'a note fits')
  assertHasError(warnings(longNote), 'a note fits')

  // Anchoring rules remain hard errors.
  assertHasError(validate(validSpec({ notes: [{ over: ['a', 'b'], text: 'hi' }] })), 'R5')
  assertHasError(validate(validSpec({ notes: [{ text: 'no anchor' }] })), 'R5')
  assertHasError(validate(validSpec({ notes: [{ under: 'zzz', text: 'hi' }] })), 'R5')
})
