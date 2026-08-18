/**
 * Deterministic geometry for a flow diagram — the orchestrator.
 *
 * Responsibility: run the two layout phases and assemble the
 *   {@link import('../../../lib/types.mjs').FlowLayoutModel} consumed by `emit`, then normalise
 *   the canvas: shift all geometry so the top-left of the content sits at the margin (a label
 *   can stick out left of column 0) and compute the final width/height.
 *     phase 0  grid.mjs   -> node rectangles + header text cells
 *     phase 1  edges.mjs  -> arrows (sides + lanes + routing) + labels
 * Edit here when: you add/reorder a phase or change how the canvas extent/offset is computed.
 * Do NOT: put phase-specific geometry here — keep this a thin composition root.
 */
import { F } from '../geometry.mjs'
import { buildGrid } from './grid.mjs'
import { buildEdges } from './edges.mjs'

/** Shift every coordinate in the model by (dx, dy). */
function translate(nodes, edges, dx, dy) {
  const moveRect = (r) => {
    if (!r) return
    r.x += dx
    r.y += dy
  }
  for (const nd of nodes) {
    moveRect(nd.rect)
    moveRect(nd.titleCell)
    moveRect(nd.subtitleCell)
  }
  for (const e of edges) {
    e.x1 += dx
    e.y1 += dy
    e.x2 += dx
    e.y2 += dy
    if (e.waypoints) for (const p of e.waypoints) {
      p.x += dx
      p.y += dy
    }
    moveRect(e.labelCell)
  }
}

/**
 * @param {import('../../../lib/types.mjs').FlowSpec} spec
 * @returns {import('../../../lib/types.mjs').FlowLayoutModel}
 */
export function layout(spec) {
  const { nodes, byId, gridRight, gridBottom } = buildGrid(spec)
  const { edges } = buildEdges(spec, byId)

  // Content bounding box across nodes, edge points and labels.
  let minX = Infinity
  let minY = Infinity
  let maxX = -Infinity
  let maxY = -Infinity
  const acc = (x, y) => {
    if (x < minX) minX = x
    if (y < minY) minY = y
    if (x > maxX) maxX = x
    if (y > maxY) maxY = y
  }
  const accRect = (r) => r && (acc(r.x, r.y), acc(r.x + r.w, r.y + r.h))
  for (const nd of nodes) accRect(nd.rect)
  for (const e of edges) {
    acc(e.x1, e.y1)
    acc(e.x2, e.y2)
    if (e.waypoints) for (const p of e.waypoints) acc(p.x, p.y)
    accRect(e.labelCell)
  }
  // Fall back to the grid extent if there is no content (shouldn't happen — nodes is non-empty).
  if (!Number.isFinite(minX)) {
    minX = F.MARGIN_X
    minY = F.MARGIN_Y
    maxX = gridRight
    maxY = gridBottom
  }

  const dx = F.MARGIN_X - minX
  const dy = F.MARGIN_Y - minY
  if (Math.abs(dx) > 0.01 || Math.abs(dy) > 0.01) translate(nodes, edges, dx, dy)

  const width = maxX + dx + F.MARGIN_X
  const height = maxY + dy + F.MARGIN_Y
  return { nodes, edges, width, height }
}
