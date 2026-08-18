/**
 * Flow layout phase 1 — edges: attach sides, lane distribution, routing, and labels.
 *
 * Responsibility: turn each spec edge into drawable geometry given the placed node rectangles.
 *   The exit/entry SIDE is derived from the grid delta (below -> bottom/top, right -> right/left,
 *   etc.); a DECISION source is special — its branches leave from the LEFT/RIGHT vertices by
 *   column delta, so they fan out instead of stacking on one tip. When several edges share one
 *   node side they are spread into parallel lanes (1 -> the centre, 2 -> symmetric ±half a lane)
 *   so arrows never overlap. Same-axis diagonals get one orthogonal Z-bend; a decision branch
 *   (horizontal out, vertical in) gets a single L-bend. Labels sit beside the arrow a constant gap
 *   away: right by default, left for the left arrow of a pair, above the source leg of a branch.
 * Inputs: (spec, byId) where byId maps id -> the grid's {@link import('../../../lib/types.mjs').FNodeLayout}.
 * Outputs: { edges }.
 * Edit here when: you change side selection, lane spacing, routing, or label placement.
 * Do NOT: size/place nodes here — that is `grid.mjs`.
 */
import { F } from '../geometry.mjs'
import { wrapText } from '../../../lib/text.mjs'

const centerOf = (nd) => ({ x: nd.rect.x + nd.rect.w / 2, y: nd.rect.y + nd.rect.h / 2 })

/**
 * The exit/entry sides for an edge, from the grid delta.
 * - Source (a box): when the target is in a DIFFERENT COLUMN, leave by the facing horizontal side
 *   (right/left) so a cross-column edge routes in the inter-column gap instead of shooting
 *   vertically across the spine and over the nodes between the two rows; otherwise (same column)
 *   leave top/bottom by row delta.
 * - Source (a DECISION): fans its branches out of the LEFT/RIGHT vertices by column delta (or the
 *   bottom/top tip when the branch runs straight down/up), so branches never pile on one tip.
 * - Target: entered by ROW when the rows differ — from its BOTTOM when the source sits below it,
 *   from its TOP when the source sits above — so an upward feed lands cleanly on the bottom edge
 *   rather than the side; only a same-row target is entered from the facing left/right side. A box
 *   source that leaves by the side (column difference) therefore makes a clean L-bend into the
 *   target's top/bottom.
 */
function sidesFor(src, tgt) {
  const colDiff = tgt.col !== src.col
  const rowDiff = tgt.row !== src.row

  let srcSide
  if (src.kind === 'decision') {
    if (tgt.col > src.col) srcSide = 'right'
    else if (tgt.col < src.col) srcSide = 'left'
    else srcSide = tgt.row > src.row ? 'bottom' : 'top'
  } else if (colDiff) {
    srcSide = tgt.col > src.col ? 'right' : 'left'
  } else {
    srcSide = tgt.row > src.row ? 'bottom' : 'top'
  }

  let tgtSide
  if (rowDiff) {
    tgtSide = tgt.row > src.row ? 'top' : 'bottom'
  } else {
    tgtSide = tgt.col > src.col ? 'left' : 'right'
  }

  return [srcSide, tgtSide]
}

/** The absolute attach point on `side` of node `nd`, nudged by `offset` along that side. */
function attachPoint(nd, side, offset) {
  const c = centerOf(nd)
  const r = nd.rect
  switch (side) {
    case 'bottom':
      return { x: c.x + offset, y: r.y + r.h }
    case 'top':
      return { x: c.x + offset, y: r.y }
    case 'right':
      return { x: r.x + r.w, y: c.y + offset }
    default: // left
      return { x: r.x, y: c.y + offset }
  }
}

/**
 * @param {import('../../../lib/types.mjs').FlowSpec} spec
 * @param {Map<string, import('../../../lib/types.mjs').FNodeLayout>} byId
 * @returns {{ edges: import('../../../lib/types.mjs').FEdgeLayout[] }}
 */
export function buildEdges(spec, byId) {
  const items = (spec.edges ?? []).map((e, i) => {
    const from = byId.get(e.from)
    const to = byId.get(e.to)
    const [srcSide, tgtSide] = sidesFor(from, to)
    return { i, e, from, to, srcSide, tgtSide, async: e.type === 'async' }
  })

  // Group the two endpoints of every edge by (node, side) so we can spread shared sides.
  const groups = new Map()
  const req = (nodeId, side, item, role, other) => {
    const key = `${nodeId}|${side}`
    const list = groups.get(key) ?? []
    list.push({ item, role, other })
    groups.set(key, list)
  }
  for (const it of items) {
    req(it.from.id, it.srcSide, it, 'src', centerOf(it.to))
    req(it.to.id, it.tgtSide, it, 'tgt', centerOf(it.from))
  }

  // Assign a lane offset to each (edge, role). Order along the side by the OTHER endpoint's
  // position so lines fan out without crossing; ties broken by edge index (deterministic).
  const offsetOf = new Map()
  for (const [key, list] of groups) {
    const [nodeId, side] = key.split('|')
    const nd = byId.get(nodeId)
    const horizontal = side === 'top' || side === 'bottom'
    list.sort((a, b) => {
      const av = horizontal ? a.other.x : a.other.y
      const bv = horizontal ? b.other.x : b.other.y
      return av - bv || a.item.i - b.item.i
    })
    const count = list.length
    const span = (horizontal ? nd.rect.w : nd.rect.h) - 20 // keep lanes off the rounded corners
    let gap = F.LANE_GAP
    if (count > 1 && (count - 1) * gap > span) gap = span / (count - 1)
    list.forEach((r, idx) => {
      const offset = nd.kind === 'decision' ? 0 : (idx - (count - 1) / 2) * gap
      offsetOf.set(`${r.item.i}|${r.role}`, offset)
    })
  }

  const edges = items.map((it) => {
    const srcOff = offsetOf.get(`${it.i}|src`) ?? 0
    const tgtOff = offsetOf.get(`${it.i}|tgt`) ?? 0
    const p1 = attachPoint(it.from, it.srcSide, srcOff)
    const p2 = attachPoint(it.to, it.tgtSide, tgtOff)
    const srcVert = it.srcSide === 'top' || it.srcSide === 'bottom'
    const tgtVert = it.tgtSide === 'top' || it.tgtSide === 'bottom'

    // Orthogonal routing:
    // - same axis on both ends -> a Z with the mid-run on the shared axis (when not aligned);
    // - mixed axes (a decision branch: horizontal out of a vertex, vertical into a top/bottom)
    //   -> a single L-bend at the corner.
    let waypoints
    if (srcVert && tgtVert) {
      if (Math.abs(p1.x - p2.x) > 0.01) {
        const midY = (p1.y + p2.y) / 2
        waypoints = [{ x: p1.x, y: midY }, { x: p2.x, y: midY }]
      }
    } else if (!srcVert && !tgtVert) {
      if (Math.abs(p1.y - p2.y) > 0.01) {
        const midX = (p1.x + p2.x) / 2
        waypoints = [{ x: midX, y: p1.y }, { x: midX, y: p2.y }]
      }
    } else {
      const corner = srcVert ? { x: p1.x, y: p2.y } : { x: p2.x, y: p1.y }
      if (Math.abs(corner.x - p1.x) > 0.01 || Math.abs(corner.y - p1.y) > 0.01) waypoints = [corner]
    }

    // Label beside the arrow, a constant perpendicular gap away.
    let labelLines = []
    let labelCell = null
    let labelAlign = 'center'
    const text = it.e.text
    if (text != null && String(text).trim()) {
      labelLines = wrapText(String(text), F.LABEL_MAX_CHARS)
      const labelW = Math.max(...labelLines.map((l) => l.length)) * F.LABEL_CHAR_PX
      const labelH = labelLines.length * F.LABEL_LH
      if (srcVert !== tgtVert) {
        // Mixed axes (decision branch): pin the label to the leg leaving the SOURCE so the two
        // branches of a decision get well-separated labels next to the diamond.
        if (!srcVert) {
          const legMidX = (p1.x + p2.x) / 2 // horizontal leg from the vertex out to the corner
          labelCell = { x: legMidX - labelW / 2, y: p1.y - F.LABEL_SIDE_GAP - labelH, w: labelW, h: labelH }
          labelAlign = 'center'
        } else {
          const legMidY = (p1.y + p2.y) / 2 // vertical leg out of the tip
          labelCell = { x: p1.x + F.LABEL_SIDE_GAP, y: legMidY - labelH / 2, w: labelW, h: labelH }
          labelAlign = 'left'
        }
      } else if (waypoints) {
        // Same-axis bent edge (a diagonal between boxes): pin the label to the MIDDLE run.
        if (srcVert) {
          const segY = waypoints[0].y // the horizontal run sits at this y
          const midX = (waypoints[0].x + waypoints[1].x) / 2
          labelCell = { x: midX - labelW / 2, y: segY - F.LABEL_SIDE_GAP - labelH, w: labelW, h: labelH }
          labelAlign = 'center'
        } else {
          const segX = waypoints[0].x // the vertical run sits at this x
          const midY = (waypoints[0].y + waypoints[1].y) / 2
          labelCell = { x: segX + F.LABEL_SIDE_GAP, y: midY - labelH / 2, w: labelW, h: labelH }
          labelAlign = 'left'
        }
      } else if (srcVert) {
        const onLeft = srcOff < 0 // the left arrow of a pair puts its label on the left
        const lineX = p1.x
        const midY = (p1.y + p2.y) / 2
        if (onLeft) {
          labelCell = { x: lineX - F.LABEL_SIDE_GAP - labelW, y: midY - labelH / 2, w: labelW, h: labelH }
          labelAlign = 'right'
        } else {
          labelCell = { x: lineX + F.LABEL_SIDE_GAP, y: midY - labelH / 2, w: labelW, h: labelH }
          labelAlign = 'left'
        }
      } else {
        const midX = (p1.x + p2.x) / 2
        const lineY = p1.y
        labelCell = { x: midX - labelW / 2, y: lineY - F.LABEL_SIDE_GAP - labelH, w: labelW, h: labelH }
        labelAlign = 'center'
      }
    }

    return { from: it.from.id, to: it.to.id, x1: p1.x, y1: p1.y, x2: p2.x, y2: p2.y, waypoints, labelLines, labelCell, labelAlign, async: it.async }
  })

  return { edges }
}
