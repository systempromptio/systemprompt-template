/**
 * Grid-cell resolution for flow nodes — shared by the validator and the layout so they can
 * never disagree on where a node sits.
 *
 * Rule: a node's column is its explicit `col` when given, otherwise its running position among
 * the nodes in its row that do NOT set an explicit col (in declaration order). This makes the
 * common case ("the k-th node listed in a row goes to column k") match author intuition while
 * still allowing a precise override.
 *
 * Edit here when: you change how (row, col) is derived from the spec. Both validate.mjs and
 * layout/grid.mjs depend on this being the single source of truth.
 */

/**
 * @param {import('../../lib/types.mjs').FlowNode[]} nodes
 * @returns {Map<string, {row:number, col:number}>} node id -> resolved cell
 */
export function resolveCells(nodes) {
  const implicitCount = new Map() // row -> next implicit column index
  const cells = new Map()
  for (const node of nodes || []) {
    const row = node.row
    let col
    if (Number.isInteger(node.col)) {
      col = node.col
    } else {
      col = implicitCount.get(row) ?? 0
      implicitCount.set(row, col + 1)
    }
    cells.set(node.id, { row, col })
  }
  return cells
}
