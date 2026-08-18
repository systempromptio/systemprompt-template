/**
 * Emit-golden snapshot — the refactor safety net.
 *
 * The layout/emit math is only ever reorganized, never changed, so the emitted mxGraph XML
 * for the fixture spec must stay byte-identical across the whole enterprise refactor. Any
 * diff here means an accidental behavior change: investigate before regenerating the golden.
 *
 * To regenerate intentionally (after a deliberate visual change), run:
 *   node --input-type=module -e "import {emit} from './scripts/types/sequence/emit.mjs';\
 *   import {validSpec} from './scripts/tests/_fixtures.mjs';import {writeFileSync} from 'node:fs';\
 *   writeFileSync('scripts/tests/__golden__/sequence-fixture.drawio', emit(validSpec(), {specYaml:'type: sequence\\nid: fixture\\n'}))"
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { emit } from '../types/sequence/emit.mjs'
import { validSpec } from './_fixtures.mjs'

const here = dirname(fileURLToPath(import.meta.url))
const GOLDEN = join(here, '__golden__', 'sequence-fixture.drawio')

test('emit output matches the golden snapshot byte-for-byte', () => {
  const expected = readFileSync(GOLDEN, 'utf8')
  const actual = emit(validSpec(), { specYaml: 'type: sequence\nid: fixture\n' })
  assert.equal(actual, expected)
})
