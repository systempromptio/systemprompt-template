/**
 * Layout phases 2 & 3 — size frames recursively and assign each message its row Y; then
 * turn the sized frames into activation-bar rectangles.
 *
 * Responsibility: derive vertical geometry from the call tree (never from arrows). A leaf
 *   frame is MIN_ACTIVATION_HEIGHT; a container is ACTIVATION_GAP + sum(children, SIBLING_GAP
 *   between — halved to SELF_SIBLING_GAP where a self block abuts a sibling) + ACTIVATION_GAP.
 *   A single pre-order walk with one advancing cursor keeps rows monotonic top-to-bottom even
 *   for overlapping/unclosed frames.
 * Inputs: the call tree + row count. Outputs: { rows, maxY } and (separately) bar rectangles.
 * Edit here when: you change bar heights, inter-frame gaps, or row placement.
 * Do NOT: read arrow positions to infer heights — that inversion is the bug this replaced.
 */
import { L } from '../geometry.mjs'

/**
 * Recursively size frames (mutating `top`/`bottom`) and resolve each message's row Y.
 * @param {import('./callTree.mjs').Frame} root
 * @param {import('./callTree.mjs').Frame[]} allFrames
 * @param {number} rowCount  number of messages (one row each)
 * @returns {{ rows: {y:number|null}[], maxY: number }}
 */
export function placeFrames(root, allFrames, rowCount) {
  const rows = Array.from({ length: rowCount }, () => ({ y: null }))
  const startY = L.MARGIN_TOP + L.HEADER_H + L.MSG_TOP_GAP
  let cursor = startY

  const place = (frame, isRoot) => {
    frame.top = cursor
    if (!isRoot && frame.actions.length === 0) {
      cursor = frame.top + L.MIN_ACTIVATION_HEIGHT // leaf: fixed minimal height
      frame.bottom = cursor
      if (frame.closeIndex >= 0) rows[frame.closeIndex].y = frame.bottom
      return
    }
    if (!isRoot) cursor += L.ACTIVATION_GAP // top inner pad (first child hugs the top)
    frame.actions.forEach((action, k) => {
      if (k > 0) {
        // A self block sits tighter against its neighbours: use half the sibling gap when
        // either this action or the previous one is a self.
        const prev = frame.actions[k - 1]
        const adjacentSelf = action.type === 'self' || prev.type === 'self'
        cursor += adjacentSelf ? L.SELF_SIBLING_GAP : L.SIBLING_GAP
      }
      if (action.type === 'call') {
        rows[action.index].y = cursor
        place(action.frame, false)
        cursor = action.frame.bottom
      } else if (action.type === 'self') {
        cursor += L.ACTIVATION_GAP // gap before the self block, like a bar's first-arrow top pad
        rows[action.index].y = cursor // block top
        cursor += L.SELF_NEST_H
      } else {
        rows[action.index].y = cursor
      }
    })
    if (!isRoot) cursor += L.ACTIVATION_GAP // bottom inner pad (return sits below last child)
    frame.bottom = cursor
    if (!isRoot && frame.closeIndex >= 0) rows[frame.closeIndex].y = frame.bottom
  }
  place(root, true)

  const maxY = cursor
  // Fallback for any row a malformed spec left unplaced (keeps output well-formed).
  let fallbackY = maxY
  rows.forEach((r) => {
    if (r.y == null) {
      fallbackY += L.ROW_STEP
      r.y = fallbackY
    }
  })

  return { rows, maxY }
}

/**
 * One rectangle per sized frame (a participant active more than once gets several). An
 * unclosed CONTAINER (still doing work) extends to the last content row; an unclosed LEAF
 * (nothing to its right) keeps its minimal height.
 * @param {import('./callTree.mjs').Frame[]} allFrames
 * @param {Map<string, import('../../../lib/types.mjs').PLayout>} byId
 * @param {number} maxY
 * @returns {import('../../../lib/types.mjs').Rect[]}
 */
export function buildActivationRects(allFrames, byId, maxY) {
  const activations = []
  for (const frame of allFrames) {
    const p = byId.get(frame.participantId)
    if (!p || !p.hasBar) continue
    const stretches = !frame.closed && frame.actions.length > 0
    const bottom = stretches ? maxY : frame.bottom
    activations.push({ x: p.xCenter - L.BAR_W / 2, y: frame.top, w: L.BAR_W, h: Math.max(0, bottom - frame.top) })
  }
  return activations
}
