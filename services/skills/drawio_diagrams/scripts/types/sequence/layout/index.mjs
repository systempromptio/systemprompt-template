/**
 * Deterministic geometry for a sequence diagram — the orchestrator.
 *
 * Responsibility: run the layout phases in order and assemble the {@link import('../../../lib/types.mjs').LayoutModel}
 *   consumed by `emit`. The LLM never authors coordinates; everything here is derived from
 *   the spec so a given spec always renders identically.
 *     phase 0  columns.mjs      -> participant columns + header text
 *     phase 1  callTree.mjs     -> activation (call) tree
 *     phase 2/3 activations.mjs -> recursive sizing + row Y, then bar rectangles
 *     phase 4  messages.mjs     -> arrows + labels
 *     phase 5  notes.mjs        -> notes under participants
 * Edit here when: you add/reorder a layout phase or change how the pieces compose into the
 *   final model or the diagram width/height. For a specific phase's math, edit that module.
 * Do NOT: put phase-specific geometry inline here — keep this a thin composition root.
 */
import { L } from '../geometry.mjs'
import { buildColumns } from './columns.mjs'
import { buildCallTree } from './callTree.mjs'
import { placeFrames, buildActivationRects } from './activations.mjs'
import { buildMessages } from './messages.mjs'
import { buildNotes } from './notes.mjs'

/**
 * @param {import('../../../lib/types.mjs').Spec} spec
 * @returns {import('../../../lib/types.mjs').LayoutModel}
 */
export function layout(spec) {
  const src = spec.messages || []
  const { participants, byId, columnCenter } = buildColumns(spec)

  // Structure -> vertical geometry.
  const { root, allFrames } = buildCallTree(src)
  const { rows, maxY } = placeFrames(root, allFrames, src.length)

  // Horizontal geometry (arrows + labels).
  const { messages, selfActivations, contentRight } = buildMessages({
    src,
    byId,
    participants,
    columnCenter,
    rows,
  })

  // Activation rectangles: main bars first, then nested self rectangles ON TOP of them.
  const activations = buildActivationRects(allFrames, byId, maxY)
  activations.push(...selfActivations)

  const lifelineBottom = maxY + L.LIFELINE_PAD
  for (const p of participants) p.lifelineBottom = lifelineBottom
  const boxesRight = Math.max(...participants.map((p) => p.right))

  const { notes, notesRight, noteEndY } = buildNotes(spec.notes, byId, lifelineBottom)

  const bottom = notes.length ? noteEndY - L.NOTE_STACK_GAP : lifelineBottom
  const width = Math.max(contentRight, boxesRight, notesRight) + L.MARGIN_X
  const height = bottom + L.MARGIN_TOP

  return { participants, messages, activations, notes, width, height }
}
