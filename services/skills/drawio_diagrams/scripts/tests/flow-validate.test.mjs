/**
 * Flow validator: the authoring rules F1..F8 (see references/flow.md).
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { validate, warnings } from '../types/flow/validate.mjs'
import { flowSpec, assertHasError, assertNoErrors } from './_fixtures.mjs'

test('the fixture spec is valid', () => {
  assertNoErrors(validate(flowSpec()))
})

test('F1: duplicate node id is rejected', () => {
  const spec = flowSpec({
    nodes: [
      { id: 'a', row: 0, title: 'A' },
      { id: 'a', row: 1, title: 'A again' },
    ],
    edges: [],
  })
  assertHasError(validate(spec), 'duplicate node id')
})

test('F1: an edge referencing an unknown node is rejected', () => {
  const spec = flowSpec({ edges: [{ from: 'shopper', to: 'ghost' }] })
  assertHasError(validate(spec), 'not a declared node')
})

test('F1: a self-loop is rejected in v1', () => {
  const spec = flowSpec({ edges: [{ from: 'cmp', to: 'cmp' }] })
  assertHasError(validate(spec), 'self-loops are not supported')
})

test('F2: a non-integer/negative row is rejected', () => {
  const spec = flowSpec({
    nodes: [{ id: 'a', row: -1, title: 'A' }, { id: 'b', row: 1, title: 'B' }],
    edges: [],
  })
  assertHasError(validate(spec), 'row must be an integer')
})

test('F3 (soft): an over-long title is a warning, not a hard error', () => {
  const spec = flowSpec({
    nodes: [{ id: 'a', row: 0, title: 'x'.repeat(40) }, { id: 'b', row: 1, title: 'B' }],
    edges: [{ from: 'a', to: 'b', text: 'go' }],
  })
  // The spec still validates (text length never blocks a render)...
  assertNoErrors(validate(spec))
  // ...but the over-budget title surfaces as an advisory.
  assertHasError(warnings(spec), 'title over')
})

test('F3 (soft): an over-long subtitle and edge label warn too', () => {
  const spec = flowSpec({
    nodes: [
      { id: 'a', row: 0, title: 'A', subtitle: 'x'.repeat(40) },
      { id: 'b', row: 1, title: 'B' },
    ],
    edges: [{ from: 'a', to: 'b', text: 'x'.repeat(80) }],
  })
  assertNoErrors(validate(spec))
  assertHasError(warnings(spec), 'subtitle over')
  assertHasError(warnings(spec), 'label over')
})

test('F4: a decision needs >= 2 guarded branches', () => {
  const oneBranch = flowSpec({
    nodes: [
      { id: 'q', row: 0, kind: 'decision', title: 'OK?' },
      { id: 'y', row: 1, title: 'Yes' },
    ],
    edges: [{ from: 'q', to: 'y', text: 'yes' }],
  })
  assertHasError(validate(oneBranch), '>= 2 outgoing')

  const missingGuard = flowSpec({
    nodes: [
      { id: 'q', row: 0, kind: 'decision', title: 'OK?' },
      { id: 'y', row: 1, col: 0, title: 'Yes' },
      { id: 'n', row: 1, col: 1, title: 'No' },
    ],
    edges: [
      { from: 'q', to: 'y', text: 'yes' },
      { from: 'q', to: 'n' },
    ],
  })
  assertHasError(validate(missingGuard), 'needs a descriptive guard label')

  const sameTarget = flowSpec({
    nodes: [
      { id: 'a', row: 0, col: 0, title: 'A' },
      { id: 'q', row: 1, col: 0, kind: 'decision', title: 'OK?' },
      { id: 'b', row: 2, col: 0, title: 'B' },
    ],
    edges: [
      { from: 'a', to: 'q', text: 'go' },
      { from: 'q', to: 'a', text: 'available' },
      { from: 'q', to: 'a', text: 'not available' },
    ],
  })
  assertHasError(validate(sameTarget), 'more than one branch')
})

test('F5: two nodes in the same (row, col) cell are rejected', () => {
  const spec = flowSpec({
    nodes: [
      { id: 'a', row: 0, col: 0, title: 'A' },
      { id: 'b', row: 0, col: 0, title: 'B' },
    ],
    edges: [],
  })
  assertHasError(validate(spec), 'occupy cell')
})

test('F6: an unknown edge type is rejected; async is accepted', () => {
  const bad = flowSpec({
    nodes: [{ id: 'a', row: 0, title: 'A' }, { id: 'b', row: 1, title: 'B' }],
    edges: [{ from: 'a', to: 'b', type: 'nope' }],
  })
  assertHasError(validate(bad), 'type must be')

  // A well-formed graph with a pure async source (feed) is accepted.
  const ok = flowSpec({
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
  assertNoErrors(validate(ok))
})

test('F7: two synchronous entries (no single start) are rejected', () => {
  const spec = flowSpec({
    nodes: [
      { id: 'shopper', row: 0, title: 'Shopper' },
      { id: 'sfnext', row: 1, title: 'SF Next' },
      { id: 'pim', row: 2, title: 'PIM' },
    ],
    // pim feeds sfnext SYNC -> pim is a second root with no incoming edge.
    edges: [
      { from: 'shopper', to: 'sfnext', text: 'browse' },
      { from: 'pim', to: 'sfnext', text: 'catalog feed' },
    ],
  })
  assertHasError(validate(spec), 'ONE entry')
})

test('F7: a pure async source (a feed) is a legitimate second origin', () => {
  // Same shape as above but the feed edge is async -> pim is exempt, shopper is the sole entry.
  const spec = flowSpec({
    nodes: [
      { id: 'shopper', row: 0, title: 'Shopper' },
      { id: 'sfnext', row: 1, title: 'SF Next' },
      { id: 'pim', row: 2, title: 'PIM' },
    ],
    edges: [
      { from: 'shopper', to: 'sfnext', text: 'browse' },
      { from: 'pim', to: 'sfnext', text: 'catalog feed', type: 'async' },
    ],
  })
  assertNoErrors(validate(spec))
})

test('F7: a detached node / island is rejected', () => {
  const spec = flowSpec({
    nodes: [
      { id: 'a', row: 0, title: 'A' },
      { id: 'b', row: 1, title: 'B' },
      { id: 'c', row: 2, title: 'C' },
      { id: 'd', row: 3, title: 'D' },
    ],
    edges: [
      { from: 'a', to: 'b', text: 'go' },
      { from: 'c', to: 'd', text: 'orphaned' },
    ],
  })
  assertHasError(validate(spec), 'not connected to the rest of the flow')
})

test('F8: an edge routed straight through a node (collinear bypass) is rejected', () => {
  // Three across, the leftmost feeding both: a -> c is drawn horizontally through b.
  const row = flowSpec({
    nodes: [
      { id: 'a', row: 0, col: 0, title: 'A' },
      { id: 'b', row: 0, col: 1, title: 'B' },
      { id: 'c', row: 0, col: 2, title: 'C' },
    ],
    edges: [
      { from: 'a', to: 'b', text: 'one' },
      { from: 'a', to: 'c', text: 'two' },
    ],
  })
  assertHasError(validate(row), 'routed straight through node "b"')

  // The same shape stacked in one column: a -> c runs vertically through b.
  const col = flowSpec({
    nodes: [
      { id: 'a', row: 0, col: 0, title: 'A' },
      { id: 'b', row: 1, col: 0, title: 'B' },
      { id: 'c', row: 2, col: 0, title: 'C' },
    ],
    edges: [
      { from: 'a', to: 'b', text: 'one' },
      { from: 'a', to: 'c', text: 'two' },
    ],
  })
  assertHasError(validate(col), 'routed straight through node "b"')
})

test('F8: fanning the targets onto different axes passes', () => {
  // The documented fix: one target down the spine, the other in a side column at the source's row.
  const spec = flowSpec({
    nodes: [
      { id: 'store', row: 0, col: 0, title: 'SF Next' },
      { id: 'search', row: 0, col: 1, title: 'Search API' },
      { id: 'commerce', row: 1, col: 0, title: 'Commerce' },
    ],
    edges: [
      { from: 'store', to: 'search', text: 'query' },
      { from: 'store', to: 'commerce', text: 'cart & pricing' },
    ],
  })
  assertNoErrors(validate(spec))
})

test('missing nodes is rejected', () => {
  assertHasError(validate({ type: 'flow', id: 'x' }), 'nodes: required')
})
