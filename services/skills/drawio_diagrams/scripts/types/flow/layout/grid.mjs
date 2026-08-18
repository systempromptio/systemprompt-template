/**
 * Flow layout phase 0 — the manual grid: node sizing + placement.
 *
 * Responsibility: turn each node into a content-sized rectangle placed on the author's grid.
 *   Columns are the union of all resolved column indices (see cells.mjs) sorted ascending, so a
 *   node sits under any other node sharing its column (the "spine" alignment). Column width =
 *   widest node in that column; row height = tallest node in that row. Node text is emitted as a
 *   statically-centered group of native-text cells (title + optional subtitle), mirroring
 *   sequence's header text.
 * Inputs: {@link import('../../../lib/types.mjs').FlowSpec}.
 * Outputs: { nodes, byId, gridRight, gridBottom }.
 * Edit here when: you change node sizing, grid spacing, or header-text centering.
 * Do NOT: place edges/labels here — that is `edges.mjs`, which reads these node rectangles.
 */
import { F } from '../geometry.mjs'
import { resolveCells } from '../cells.mjs'

/**
 * Node size. Boxes are a FIXED width×height (F3 budgets guarantee the text fits); a decision is a
 * fixed diamond of the same width. `groupH` is the height of the centered title(+subtitle) group.
 */
function nodeSize(kind, subtitle) {
  if (kind === 'decision') return { w: F.DECISION_W, h: F.DECISION_H, groupH: 0 }
  const groupH = F.TITLE_LH + (subtitle ? F.SUBTITLE_LH : 0)
  return { w: F.BOX_W, h: F.BOX_H, groupH }
}

/**
 * @param {import('../../../lib/types.mjs').FlowSpec} spec
 * @returns {{ nodes: import('../../../lib/types.mjs').FNodeLayout[], byId: Map<string, import('../../../lib/types.mjs').FNodeLayout>, gridRight: number, gridBottom: number }}
 */
export function buildGrid(spec) {
  const cells = resolveCells(spec.nodes)

  // Measure every node first (we need per-column/per-row maxima before we can place anything).
  const measured = spec.nodes.map((node) => {
    const kind = node.kind ?? 'box'
    const title = node.title ?? node.id
    const subtitle = node.subtitle ?? null
    const { w, h, groupH } = nodeSize(kind, subtitle)
    const { row, col } = cells.get(node.id)
    return { node, kind, title, subtitle, w, h, groupH, row, col }
  })

  // Union of used rows/cols -> sequential slots (a large explicit col does not create gaps).
  const rowVals = [...new Set(measured.map((m) => m.row))].sort((a, b) => a - b)
  const colVals = [...new Set(measured.map((m) => m.col))].sort((a, b) => a - b)
  const rowSlot = new Map(rowVals.map((v, i) => [v, i]))
  const colSlot = new Map(colVals.map((v, i) => [v, i]))

  const colWidth = colVals.map((v) => Math.max(...measured.filter((m) => m.col === v).map((m) => m.w)))
  const rowHeight = rowVals.map((v) => Math.max(...measured.filter((m) => m.row === v).map((m) => m.h)))

  const colLeft = []
  let x = F.MARGIN_X
  for (let s = 0; s < colWidth.length; s++) {
    colLeft[s] = x
    x += colWidth[s] + F.COL_GAP
  }
  const rowTop = []
  let y = F.MARGIN_Y
  for (let s = 0; s < rowHeight.length; s++) {
    rowTop[s] = y
    y += rowHeight[s] + F.ROW_GAP
  }
  const colCenter = (slot) => colLeft[slot] + colWidth[slot] / 2
  const rowCenter = (slot) => rowTop[slot] + rowHeight[slot] / 2

  const nodes = measured.map((m) => {
    const cx = colCenter(colSlot.get(m.col))
    const cy = rowCenter(rowSlot.get(m.row))
    const rect = { x: cx - m.w / 2, y: cy - m.h / 2, w: m.w, h: m.h }
    // A decision renders as a bare diamond — no title/subtitle text cells.
    const isDecision = m.kind === 'decision'
    const groupTop = rect.y + (rect.h - m.groupH) / 2
    const titleCell = isDecision ? null : { x: rect.x, y: groupTop, w: rect.w, h: F.TITLE_LH }
    const subtitleCell =
      !isDecision && m.subtitle
        ? { x: rect.x, y: groupTop + F.TITLE_LH, w: rect.w, h: F.SUBTITLE_LH }
        : null
    return {
      id: m.node.id,
      kind: m.kind,
      row: m.row,
      col: m.col,
      rect,
      titleCell,
      subtitleCell,
      title: m.title,
      subtitle: m.subtitle,
      titleLines: [m.title],
      subtitleLines: m.subtitle ? [m.subtitle] : [],
    }
  })

  const byId = new Map(nodes.map((nd) => [nd.id, nd]))
  const lastCol = colWidth.length - 1
  const lastRow = rowHeight.length - 1
  const gridRight = colLeft[lastCol] + colWidth[lastCol]
  const gridBottom = rowTop[lastRow] + rowHeight[lastRow]
  return { nodes, byId, gridRight, gridBottom }
}
