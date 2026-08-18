/**
 * Flow emit-golden snapshot — the refactor safety net for `type: flow`.
 *
 * The layout/emit math is only ever reorganized, never changed, so the emitted mxGraph XML for
 * the fixture spec must stay byte-identical. Any diff here means an accidental behavior change:
 * investigate before regenerating the golden.
 *
 * To regenerate intentionally (after a deliberate visual change), run:
 *   node --input-type=module -e "import {emit} from './scripts/types/flow/emit.mjs';\
 *   import {flowSpec} from './scripts/tests/_fixtures.mjs';import {writeFileSync} from 'node:fs';\
 *   writeFileSync('scripts/tests/__golden__/flow-fixture.drawio', emit(flowSpec(), {specYaml:'type: flow\\nid: consent-integration\\n'}))"
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { emit } from '../types/flow/emit.mjs'
import { flowSpec } from './_fixtures.mjs'

const here = dirname(fileURLToPath(import.meta.url))
const GOLDEN = join(here, '__golden__', 'flow-fixture.drawio')

test('flow emit output matches the golden snapshot byte-for-byte', () => {
  const expected = readFileSync(GOLDEN, 'utf8')
  const actual = emit(flowSpec(), { specYaml: 'type: flow\nid: consent-integration\n' })
  assert.equal(actual, expected)
})
