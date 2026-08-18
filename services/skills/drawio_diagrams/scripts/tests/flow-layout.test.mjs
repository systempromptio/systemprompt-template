/**
 * Flow layout: the deterministic geometry guarantees — column auto-alignment, content sizing,
 * and parallel-edge lane distribution with opposite-side labels.
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { layout } from '../types/flow/layout/index.mjs'
import { flowSpec } from './_fixtures.mjs'

const centerX = (nd) => nd.rect.x + nd.rect.w / 2
const byId = (model, id) => model.nodes.find((n) => n.id === id)

test('nodes in the same column auto-align vertically (shared spine)', () => {
  const m = layout(flowSpec())
  // shopper (row0,col0), cmp (row1,col0), sfnext (row2,col0) share column 0.
  assert.ok(Math.abs(centerX(byId(m, 'shopper')) - centerX(byId(m, 'cmp'))) < 0.01)
  assert.ok(Math.abs(centerX(byId(m, 'cmp')) - centerX(byId(m, 'sfnext'))) < 0.01)
})

test('every box has the same fixed width (a subtitle does not widen it)', () => {
  const m = layout(flowSpec())
  const boxes = m.nodes.filter((n) => n.kind !== 'decision')
  const w0 = boxes[0].rect.w
  for (const b of boxes) assert.equal(b.rect.w, w0, `box ${b.id} should share the fixed box width`)
  assert.equal(byId(m, 'gtm').rect.w, byId(m, 'cmp').rect.w, 'a subtitle must not change the width')
})

test('two edges sharing a node side are spread into distinct lanes', () => {
  const m = layout(flowSpec())
  // edges[2] = cmp->sfnext (notify), edges[3] = sfnext->cmp (subscribe): a parallel pair.
  const notify = m.edges[2]
  const subscribe = m.edges[3]
  assert.ok(Math.abs(notify.x1 - subscribe.x1) > 1, 'parallel arrows must not overlap (different lanes)')
})

test('the labels of a parallel pair sit on opposite sides', () => {
  const m = layout(flowSpec())
  assert.notEqual(m.edges[2].labelAlign, m.edges[3].labelAlign)
})

test('an explicit col overrides the implicit within-row index', () => {
  const m = layout(
    flowSpec({
      nodes: [
        { id: 'a', row: 0, title: 'A' },
        { id: 'b', row: 0, title: 'B' },
        { id: 'c', row: 1, col: 1, title: 'C' },
      ],
      edges: [],
    }),
  )
  // c is pinned to column 1, so it aligns under b (also column 1), not under a.
  assert.ok(Math.abs(centerX(byId(m, 'c')) - centerX(byId(m, 'b'))) < 0.01)
})

test('a decision fans its branches out of the left/right vertices', () => {
  const m = layout(
    flowSpec({
      nodes: [
        { id: 'a', row: 0, col: 1, title: 'Start' },
        { id: 'q', row: 1, col: 1, kind: 'decision', title: 'ignored' },
        { id: 'y', row: 2, col: 0, title: 'Yes path' },
        { id: 'n', row: 2, col: 2, title: 'No path' },
      ],
      edges: [
        { from: 'a', to: 'q' },
        { from: 'q', to: 'y', text: 'accepted' },
        { from: 'q', to: 'n', text: 'rejected' },
      ],
    }),
  )
  const q = byId(m, 'q')
  const cy = q.rect.y + q.rect.h / 2
  const left = m.edges[1] // q -> y (down-left) leaves the left vertex
  const right = m.edges[2] // q -> n (down-right) leaves the right vertex
  assert.ok(Math.abs(left.y1 - cy) < 0.01 && Math.abs(right.y1 - cy) < 0.01, 'branches leave at the vertex height')
  assert.ok(Math.abs(left.x1 - q.rect.x) < 0.01, 'left branch leaves the left vertex')
  assert.ok(Math.abs(right.x1 - (q.rect.x + q.rect.w)) < 0.01, 'right branch leaves the right vertex')
})

test('a decision renders no title/subtitle cells', () => {
  const m = layout(
    flowSpec({
      nodes: [
        { id: 'q', row: 0, kind: 'decision', title: 'ignored' },
        { id: 'y', row: 1, col: 0, title: 'Yes' },
        { id: 'n', row: 1, col: 1, title: 'No' },
      ],
      edges: [
        { from: 'q', to: 'y', text: 'yes-ish' },
        { from: 'q', to: 'n', text: 'no-ish' },
      ],
    }),
  )
  assert.equal(byId(m, 'q').titleCell, null)
  assert.equal(byId(m, 'q').subtitleCell, null)
})

test('a cross-column box edge leaves by the side but enters the target by row', () => {
  // a (row0,col0) -> b (row1,col1): the source leaves its RIGHT side (columns differ) so the edge
  // routes through the inter-column gap instead of shooting vertically across the spine; the target
  // sits below the source, so the arrow lands on the target's TOP edge (entry is row-based).
  const m = layout(
    flowSpec({
      nodes: [
        { id: 'a', row: 0, col: 0, title: 'A' },
        { id: 'b', row: 1, col: 1, title: 'B' },
      ],
      edges: [{ from: 'a', to: 'b', text: 'go' }],
    }),
  )
  const a = byId(m, 'a')
  const b = byId(m, 'b')
  const e = m.edges[0]
  assert.ok(Math.abs(e.x1 - (a.rect.x + a.rect.w)) < 0.01, 'source leaves the right side')
  assert.ok(Math.abs(e.y2 - b.rect.y) < 0.01, 'target below the source is entered from its top edge')
})

test('an upward cross-column feed enters the target from its bottom edge', () => {
  // pim (row2,col0) -> api (row0,col1): the source is BELOW the target, so the arrow must land on
  // the target's BOTTOM edge (not the side) — an upward feed reads cleanly into the bottom.
  const m = layout(
    flowSpec({
      nodes: [
        { id: 'api', row: 0, col: 1, title: 'API' },
        { id: 'store', row: 0, col: 0, title: 'Store' },
        { id: 'pim', row: 2, col: 0, title: 'PIM' },
      ],
      edges: [
        { from: 'store', to: 'api', text: 'query' },
        { from: 'pim', to: 'api', text: 'feed', type: 'async' },
      ],
    }),
  )
  const pim = byId(m, 'pim')
  const api = byId(m, 'api')
  const feed = m.edges[1]
  assert.ok(Math.abs(feed.x1 - (pim.rect.x + pim.rect.w)) < 0.01, 'the feed leaves the source right side')
  assert.ok(Math.abs(feed.y2 - (api.rect.y + api.rect.h)) < 0.01, 'the feed enters the target bottom edge')
})

test('the async flag propagates through layout to the edge item', () => {
  const m = layout(
    flowSpec({
      nodes: [
        { id: 'shopper', row: 0, title: 'Shopper' },
        { id: 'sfnext', row: 1, title: 'SF Next' },
        { id: 'pim', row: 2, title: 'PIM' },
      ],
      edges: [
        { from: 'shopper', to: 'sfnext', text: 'browse' },
        { from: 'pim', to: 'sfnext', text: 'feed', type: 'async' },
      ],
    }),
  )
  assert.ok(!m.edges[0].async, 'a plain edge is not async')
  assert.equal(m.edges[1].async, true, 'a type:async edge carries async=true')
})

test('the canvas has a positive extent and a left margin', () => {
  const m = layout(flowSpec())
  assert.ok(m.width > 0 && m.height > 0)
  const minX = Math.min(...m.nodes.map((n) => n.rect.x), ...m.edges.flatMap((e) => (e.labelCell ? [e.labelCell.x] : [])))
  assert.ok(minX >= 0, 'nothing should be placed off the left edge of the canvas')
})
