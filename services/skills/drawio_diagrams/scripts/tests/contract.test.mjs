/**
 * Type-contract tests — every registered diagram type must satisfy the uniform interface, and
 * assertType must reject incomplete modules. This is what future types (e.g. activity) rely on.
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { assertType } from '../types/contract.mjs'
import registry, { getType, listTypes } from '../types/index.mjs'

test('every registered type satisfies the contract', () => {
  assert.ok(listTypes().length >= 1)
  for (const name of listTypes()) {
    const t = getType(name)
    assert.doesNotThrow(() => assertType(t), `type "${name}" must satisfy the contract`)
    assert.equal(t.type, name)
  }
  assert.equal(registry[listTypes()[0]].type, listTypes()[0])
})

test('assertType rejects incomplete modules', () => {
  assert.throws(() => assertType(null), TypeError)
  assert.throws(() => assertType({ type: 'x', title: 'X', validate() {}, layout() {} }), /emit/)
  assert.throws(() => assertType({ type: '', title: 'X', validate() {}, layout() {}, emit() {} }), /type/)
  assert.doesNotThrow(() =>
    assertType({ type: 'x', title: 'X', validate() {}, layout() {}, emit() {} }),
  )
})
