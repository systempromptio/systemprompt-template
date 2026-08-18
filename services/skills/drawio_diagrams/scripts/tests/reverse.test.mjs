/**
 * Reverse pull round-trip: a generated `.drawio` embeds the full spec as base64
 * `data-spec`, so `readEmbeddedSpec` + `specToBlock` reconstruct the authoring
 * block without parsing draw.io geometry. This is the inverse of
 * `extractDrawioBlocks` + the generator's header injection.
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import YAML from 'yaml'
import { readEmbeddedSpec, specToBlock, extractDrawioBlocks } from '../lib/spec.mjs'
import { validSpec } from './_fixtures.mjs'

function drawioWith(spec, format = 'drawio-diagrams/v1') {
  const b64 = Buffer.from(YAML.stringify(spec), 'utf8').toString('base64')
  return `<mxfile><diagram id="d" name="n" data-spec="${b64}" data-spec-format="${format}"><root/></diagram></mxfile>`
}

test('readEmbeddedSpec decodes the base64 data-spec back to the full spec', () => {
  const spec = validSpec()
  assert.deepEqual(readEmbeddedSpec(drawioWith(spec)), spec)
})

test('readEmbeddedSpec rejects a missing or mismatched data-spec-format', () => {
  assert.throws(() => readEmbeddedSpec('<mxfile><diagram/></mxfile>'), /data-spec-format/)
  assert.throws(() => readEmbeddedSpec(drawioWith(validSpec(), 'drawio-diagrams/v2')), /v2/)
})

test('specToBlock is the inverse of extractDrawioBlocks (header type/id, body scenario)', () => {
  const spec = validSpec()
  const block = specToBlock(spec)

  assert.ok(block.startsWith('```drawio:sequence:fixture\n'))
  // Header carries type/id; the body must not repeat them.
  assert.ok(!/^type:/m.test(block))
  assert.ok(!/^id:/m.test(block))

  const [parsed] = extractDrawioBlocks(block)
  assert.equal(parsed.type, 'sequence')
  assert.equal(parsed.id, 'fixture')

  // Re-injecting the header reproduces the original spec exactly.
  const rebuilt = { type: parsed.type, id: parsed.id, ...YAML.parse(parsed.body) }
  assert.deepEqual(rebuilt, spec)
})

test('specToBlock emits compact one-line items and auto-quotes comma text', () => {
  const block = specToBlock({
    type: 'sequence',
    id: 'x',
    title: 'T',
    participants: [{ id: 'a', title: 'A', kind: 'actor' }],
    messages: [{ kind: 'self', from: 'a', text: 'Show notice, read stored consent' }],
  })

  // Leaf items render inline as `{ … }`, one per line.
  assert.ok(block.includes('- { id: a, title: A, kind: actor }'))
  // A value with a comma is quoted by the serializer (safe without hand-quoting).
  assert.ok(block.includes('text: "Show notice, read stored consent"'))
  // The outer keys stay block-style.
  assert.ok(/^participants:$/m.test(block))
  assert.ok(/^messages:$/m.test(block))
})
