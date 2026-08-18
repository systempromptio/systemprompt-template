/**
 * Layout geometry invariants. These lock the *properties* of the deterministic layout
 * (monotonic rows, recursive bar sizing, non-overlapping self labels, centered headers/notes)
 * rather than exact pixel snapshots, so the refactor can move internals freely.
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { layout } from '../types/sequence/layout/index.mjs'
import { L } from '../types/sequence/geometry.mjs'
import { validSpec } from './_fixtures.mjs'

const near = (a, b, eps = 1e-6) => Math.abs(a - b) <= eps

test('columns use a variable step (tighter next to an actor)', () => {
  const { participants } = layout(validSpec())
  const [u, a, b] = participants
  assert.ok(near(a.xCenter - u.xCenter, L.COL_STEP * L.COL_STEP_ACTOR_FACTOR))
  assert.ok(near(b.xCenter - a.xCenter, L.COL_STEP * L.COL_STEP_COMPONENT_FACTOR))
})

test('message rows are monotonic top-to-bottom in spec order', () => {
  const { messages } = layout(validSpec())
  let prev = -Infinity
  for (const m of messages) {
    assert.ok(m.y >= prev - 1e-6, `row y went backwards: ${m.y} < ${prev}`)
    prev = m.y
  }
})

test('activation heights are derived recursively from the call tree', () => {
  // b holds one self (leaf-self -> own nested block); a wraps the call to b.
  const { activations, participants } = layout(validSpec())
  const barX = (p) => p.xCenter - L.BAR_W / 2
  const [, a, b] = participants
  const mainBarAt = (p) => activations.find((z) => near(z.x, barX(p)) && z.w === L.BAR_W)

  const bH = 3 * L.ACTIVATION_GAP + L.SELF_NEST_H // GAP + (GAP + self block) + GAP
  const aH = 2 * L.ACTIVATION_GAP + bH // container wrapping b's frame
  assert.ok(near(mainBarAt(b).h, bH), `b bar h=${mainBarAt(b).h} expected ${bH}`)
  assert.ok(near(mainBarAt(a).h, aH), `a bar h=${mainBarAt(a).h} expected ${aH}`)
})

test('a leaf activation is exactly MIN_ACTIVATION_HEIGHT', () => {
  const spec = {
    type: 'sequence',
    id: 't',
    participants: [
      { id: 'u', title: 'U', kind: 'actor' },
      { id: 'a', title: 'A' },
    ],
    messages: [
      { kind: 'call', from: 'u', to: 'a', text: 'go' },
      { kind: 'return', from: 'a', to: 'u', text: 'ok' },
    ],
    notes: [],
  }
  const { activations } = layout(spec)
  assert.equal(activations.length, 1)
  assert.ok(near(activations[0].h, L.MIN_ACTIVATION_HEIGHT))
})

test('a self label never overruns the next participant lifeline', () => {
  const spec = {
    type: 'sequence',
    id: 't',
    participants: [
      { id: 'u', title: 'U', kind: 'actor' },
      { id: 'a', title: 'A' },
      { id: 'b', title: 'B' },
    ],
    messages: [
      { kind: 'call', from: 'u', to: 'a', text: 'go' },
      { kind: 'self', from: 'a', text: 'do a fair amount of internal work' },
      { kind: 'return', from: 'a', to: 'u', text: 'ok' },
    ],
    notes: [],
  }
  const { messages, participants } = layout(spec)
  const self = messages.find((m) => m.kind === 'self')
  const nextCenter = participants[2].xCenter
  assert.ok(self.labelX + self.labelW <= nextCenter - L.SELF_LABEL_MARGIN + 1e-6)
})

test('a note is fixed-width and centered on its participant column', () => {
  const { notes, participants } = layout(validSpec({ notes: [{ under: 'a', text: 'hi there' }] }))
  const a = participants.find((p) => p.id === 'a')
  assert.equal(notes.length, 1)
  assert.equal(notes[0].w, L.NOTE_MAX_W)
  assert.ok(Math.abs(notes[0].x + notes[0].w / 2 - a.xCenter) < 0.5)
})

test('multiple notes share one top y (single horizontal band)', () => {
  const { notes } = layout(
    validSpec({
      notes: [
        { under: 'a', text: 'short' },
        { under: 'b', text: 'a considerably longer note that wraps onto several lines to be taller' },
      ],
    }),
  )
  assert.equal(notes.length, 2)
  assert.ok(near(notes[0].y, notes[1].y), `notes not on one y: ${notes[0].y} vs ${notes[1].y}`)
})

test('header text is a centered group inside the box', () => {
  const { participants } = layout(validSpec())
  const b = participants.find((p) => p.id === 'b') // has a subtitle
  assert.ok(Math.abs(b.titleCell.x + b.titleCell.w / 2 - b.xCenter) < 0.5)
  assert.ok(b.subtitleCell, 'subtitle cell should exist')
  assert.ok(b.titleCell.y >= b.headerTop - 1e-6)
  assert.ok(b.subtitleCell.y + L.SUBTITLE_LH <= b.headerTop + L.HEADER_BOX_H + 1e-6)
})
