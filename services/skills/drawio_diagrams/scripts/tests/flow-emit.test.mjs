/**
 * Flow XML emission: structural correctness + the hard RENDER-SAFE invariant (native <text>
 * only — no html=1 / foreignObject / whiteSpace=wrap, which the browserless rasterizer drops).
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { emit } from '../types/flow/emit.mjs'
import { flowSpec } from './_fixtures.mjs'

test('emits a well-formed uncompressed mxfile', () => {
  const xml = emit(flowSpec())
  assert.ok(xml.startsWith('<mxfile>'))
  assert.ok(xml.includes('<mxGraphModel'))
  assert.ok(xml.trimEnd().endsWith('</mxfile>'))
})

test('RENDER-SAFE: no html labels, foreignObject, or wrap', () => {
  const xml = emit(flowSpec())
  assert.ok(!xml.includes('html=1'), 'html=1 must never appear')
  assert.ok(!/foreignObject/i.test(xml), 'foreignObject must never appear')
  assert.ok(!xml.includes('whiteSpace=wrap'), 'whiteSpace=wrap must never appear')
})

test('node titles/subtitles and edge labels are present as text', () => {
  const xml = emit(flowSpec())
  for (const s of ['Shopper', 'CMP', 'GTM/Adobe', '3rd-party tags', 'accept/reject/custom', 'notify', 'subscribe']) {
    assert.ok(xml.includes(s), `expected label ${JSON.stringify(s)} in the XML`)
  }
})

test('a decision node emits a rhombus shape', () => {
  const spec = flowSpec({
    nodes: [
      { id: 'a', row: 0, title: 'Start work' },
      { id: 'q', row: 1, kind: 'decision', title: 'OK?' },
      { id: 'y', row: 2, col: 0, title: 'Yes path' },
      { id: 'n', row: 2, col: 1, title: 'No path' },
    ],
    edges: [
      { from: 'a', to: 'q' },
      { from: 'q', to: 'y', text: 'yes' },
      { from: 'q', to: 'n', text: 'no' },
    ],
  })
  assert.ok(emit(spec).includes('rhombus'), 'decision node should render as a rhombus')
})

test('an async edge renders dashed while sync edges stay solid', () => {
  const spec = flowSpec({
    nodes: [
      { id: 'shopper', row: 0, title: 'Shopper' },
      { id: 'sfnext', row: 1, title: 'SF Next' },
      { id: 'pim', row: 2, title: 'PIM / feed' },
    ],
    edges: [
      { from: 'shopper', to: 'sfnext', text: 'browse' },
      { from: 'pim', to: 'sfnext', text: 'catalog feed', type: 'async' },
    ],
  })
  const xml = emit(spec)
  // Exactly one dashed edge (the async feed); the sync edge carries no dashPattern.
  const dashed = xml.match(/dashPattern=6 4/g) ?? []
  assert.equal(dashed.length, 1, 'only the async edge should be dashed')
})

test('embeds the raw spec as base64 when provided', () => {
  const yaml = 'type: flow\nid: consent-integration\n'
  const xml = emit(flowSpec(), { specYaml: yaml })
  assert.ok(xml.includes('data-spec="'))
  assert.ok(xml.includes('data-spec-format="drawio-diagrams/v1"'))
  const b64 = /data-spec="([^"]+)"/.exec(xml)[1]
  assert.equal(Buffer.from(b64, 'base64').toString('utf8'), yaml)
})

test('emission is deterministic', () => {
  assert.equal(emit(flowSpec()), emit(flowSpec()))
})
