/**
 * Layout phase 5 — notes.
 *
 * Responsibility: place each note under exactly ONE participant, centered on its column at a
 *   fixed max width (1.5x a participant box), wrapped within that width. All notes share the
 *   same top y (one horizontal band below the lifelines) so they read on a single axis; the x
 *   is clamped so a note never clips off the left edge.
 * Inputs: (notes, byId, lifelineBottom). Outputs: { notes, notesRight, noteEndY }.
 * Edit here when: you change note width/anchoring/alignment.
 * Do NOT: let a note span multiple participants — the model is deliberately one-participant.
 */
import { L } from '../geometry.mjs'
import { wrapText } from '../../../lib/text.mjs'

/**
 * @param {import('../../../lib/types.mjs').Note[]} specNotes
 * @param {Map<string, import('../../../lib/types.mjs').PLayout>} byId
 * @param {number} lifelineBottom
 * @returns {{ notes: (import('../../../lib/types.mjs').Rect & {lines:string[]})[], notesRight: number, noteEndY: number }}
 */
export function buildNotes(specNotes, byId, lifelineBottom) {
  const notes = []
  // All notes align on a single y-axis: one horizontal band under the lifelines, each note
  // centered on its own participant column. (Heights may differ; they are top-aligned.)
  const noteY = lifelineBottom + L.NOTE_GAP
  const noteW = L.NOTE_MAX_W
  const notePerLine = Math.max(8, Math.floor((noteW - 24) / L.NOTE_CHAR_PX))
  let notesRight = 0
  let maxH = 0
  for (const note of specNotes || []) {
    const p = byId.get(note.under)
    const center = p ? p.xCenter : L.MARGIN_X + noteW / 2
    const lines = wrapText(note.text, notePerLine)
    const h = Math.max(L.NOTE_MIN_H, lines.length * L.NOTE_LINE_H + L.NOTE_PAD_Y)
    const x = Math.max(L.MARGIN_X, center - noteW / 2) // clamp so it never clips off-canvas
    notes.push({ x, y: noteY, w: noteW, h, lines })
    notesRight = Math.max(notesRight, x + noteW)
    maxH = Math.max(maxH, h)
  }
  const noteEndY = notes.length ? noteY + maxH + L.NOTE_STACK_GAP : noteY
  return { notes, notesRight, noteEndY }
}
