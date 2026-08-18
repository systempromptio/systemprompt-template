/**
 * wrapText — the layout-time wrapping used by self labels and notes (render-safe path forbids
 * CSS wrap). Locks the edge cases: explicit newlines, over-long single words, empties, defaults.
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { wrapText } from '../lib/text.mjs'

test('wraps on word boundaries to <= maxChars', () => {
  assert.deepEqual(wrapText('one two three four', 8), ['one two', 'three', 'four'])
})

test('honours explicit newlines', () => {
  assert.deepEqual(wrapText('a\nb c', 10), ['a', 'b c'])
})

test('keeps an over-long word on its own line rather than splitting', () => {
  assert.deepEqual(wrapText('supercalifragilistic x', 6), ['supercalifragilistic', 'x'])
})

test('empty / nullish input yields a single empty line', () => {
  assert.deepEqual(wrapText(''), [''])
  assert.deepEqual(wrapText(null), [''])
  assert.deepEqual(wrapText(undefined), [''])
})

test('has a sane default width (no dependency on a missing constant)', () => {
  const lines = wrapText('alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu')
  assert.ok(lines.length >= 1)
  assert.ok(lines.every((l) => l.length <= 40))
})
