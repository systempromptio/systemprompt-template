/**
 * XML emission: structural correctness + the hard RENDER-SAFE invariant (native <text>
 * only — no html=1 / foreignObject / whiteSpace=wrap, which the browserless rasterizer drops).
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { emit } from '../types/sequence/emit.mjs'
import { COLORS } from '../lib/design.mjs'
import { validSpec } from './_fixtures.mjs'

test('emits a well-formed uncompressed mxfile', () => {
  const xml = emit(validSpec())
  assert.ok(xml.startsWith('<mxfile>'))
  assert.ok(xml.includes('<mxGraphModel'))
  assert.ok(xml.trimEnd().endsWith('</mxfile>'))
})

test('RENDER-SAFE: no html labels, foreignObject, or wrap', () => {
  const xml = emit(validSpec())
  assert.ok(!xml.includes('html=1'), 'html=1 must never appear')
  assert.ok(!/foreignObject/i.test(xml), 'foreignObject must never appear')
  assert.ok(!xml.includes('whiteSpace=wrap'), 'whiteSpace=wrap must never appear')
})

test('participant titles and message labels are present as text', () => {
  const xml = emit(validSpec())
  for (const s of ['Shopper', 'Web Browser', 'SF Next', 'Edge runtime', 'GET /', 'Read cache']) {
    assert.ok(xml.includes(s), `expected label ${JSON.stringify(s)} in the XML`)
  }
})

test('uses the bundled Inter font and the accent for the actor', () => {
  const xml = emit(validSpec())
  assert.ok(xml.includes('fontFamily=Inter'))
  assert.ok(xml.includes(`fontColor=${COLORS.ACCENT}`), 'actor title should use the accent colour')
})

test('embeds the raw spec as base64 when provided', () => {
  const yaml = 'type: sequence\nid: fixture\n'
  const xml = emit(validSpec(), { specYaml: yaml })
  assert.ok(xml.includes('data-spec="'))
  assert.ok(xml.includes('data-spec-format="drawio-diagrams/v1"'))
  const b64 = /data-spec="([^"]+)"/.exec(xml)[1]
  assert.equal(Buffer.from(b64, 'base64').toString('utf8'), yaml)
})

test('emission is deterministic', () => {
  assert.equal(emit(validSpec()), emit(validSpec()))
})
