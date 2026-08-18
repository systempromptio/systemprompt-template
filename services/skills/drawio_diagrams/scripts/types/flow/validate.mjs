/**
 * Flow spec validation — structural checks (F1, F2, F4..F7) plus one geometry check (F8), and a
 * separate soft `warnings()` pass for the F3 text budgets.
 *
 * Responsibility: reject a flow spec that could not be laid out unambiguously OR that lays out with
 *   an edge crossing a box, with an error naming the rule id (see `references/flow.md`). An invalid
 *   spec produces NO files. F1/F2/F4..F7 are spec-level; F8 runs `layout` and inspects the geometry,
 *   so it is gated behind an otherwise-clean spec. The F3 text budgets are SOFT: over-budget text
 *   overflows the fixed box but still renders, so `warnings()` reports it (to inform the author)
 *   without blocking a file — box width never depends on text length (see `grid.mjs`).
 * Inputs/Outputs: a spec in, `string[]` of error messages out ([] = valid); `warnings()` returns a
 *   parallel `string[]` of non-blocking advisories.
 * Edit here when: you add/relax a flow authoring rule. Keep the (row, col) derivation in
 *   `cells.mjs` (shared with the layout) — do not re-derive it here.
 * Do NOT: assume `spec` is well-formed — the envelope is checked by `lib/spec.mjs`, but the
 *   body here may be anything a user typed.
 */
import { F } from './geometry.mjs'
import { cite } from './rules.mjs'
import { resolveCells } from './cells.mjs'
import { layout } from './layout/index.mjs'

const KINDS = new Set(['box', 'decision'])
const LABEL_MAX_CHARS = F.LABEL_MAX_CHARS * 2 // an edge label may wrap onto a second line

// A small inset so an edge running alongside a box in the grid gap never counts as "through" it;
// a genuine bypass crosses deep into the interior. Endpoints are excluded separately.
const THROUGH_INSET = 3

/**
 * Does the segment p->q cross into the axis-aligned rectangle? Liang–Barsky clip: the segment
 * intersects the rect iff the parametric overlap [t0, t1] on [0, 1] is non-empty.
 * @param {{x:number,y:number}} p
 * @param {{x:number,y:number}} q
 * @param {{x:number,y:number,w:number,h:number}} rect
 * @returns {boolean}
 */
function segmentIntersectsRect(p, q, rect) {
  const dx = q.x - p.x
  const dy = q.y - p.y
  const xmin = rect.x
  const xmax = rect.x + rect.w
  const ymin = rect.y
  const ymax = rect.y + rect.h
  let t0 = 0
  let t1 = 1
  // Clip the parameter range [t0, t1] against one boundary. `den` is the Liang–Barsky p_k and `num`
  // is q_k, so the boundary is hit at t = num/den; a parallel segment (den === 0) is inside that
  // slab iff num >= 0.
  const clip = (num, den) => {
    if (den === 0) return num >= 0
    const t = num / den
    if (den < 0) {
      if (t > t1) return false
      if (t > t0) t0 = t
    } else {
      if (t < t0) return false
      if (t < t1) t1 = t
    }
    return true
  }
  return (
    clip(p.x - xmin, -dx) &&
    clip(xmax - p.x, dx) &&
    clip(p.y - ymin, -dy) &&
    clip(ymax - p.y, dy) &&
    t0 <= t1
  )
}

/**
 * @param {import('../../lib/types.mjs').FlowSpec} spec
 * @returns {string[]}
 */
export function validate(spec) {
  const errors = []
  const nodes = spec.nodes
  if (!Array.isArray(nodes) || nodes.length === 0) {
    errors.push('nodes: required, a non-empty list of { id, row, ... }')
    return errors
  }

  // --- nodes: shape, ids, budgets (F1/F2/F3) ---
  const ids = new Set()
  nodes.forEach((node, i) => {
    const at = `nodes[${i}]${node && node.id ? ` (${node.id})` : ''}`
    if (!node || typeof node !== 'object' || Array.isArray(node)) {
      errors.push(`${at}: must be a mapping { id, row, ... }`)
      return
    }
    if (!node.id || typeof node.id !== 'string') {
      errors.push(`${at}: id is required (a stable string) ${cite('F1')}`)
    } else if (ids.has(node.id)) {
      errors.push(`${at}: duplicate node id "${node.id}" ${cite('F1')}`)
    } else {
      ids.add(node.id)
    }
    if (!Number.isInteger(node.row) || node.row < 0) {
      errors.push(`${at}: row must be an integer >= 0 ${cite('F2')}`)
    }
    if (node.col != null && (!Number.isInteger(node.col) || node.col < 0)) {
      errors.push(`${at}: col, when set, must be an integer >= 0 ${cite('F2')}`)
    }
    if (node.kind != null && !KINDS.has(node.kind)) {
      errors.push(`${at}: kind must be one of box | decision (got "${node.kind}") ${cite('F1')}`)
    }
    // Text budgets (F3) are NOT enforced here — they are soft, reported by `warnings()` below:
    // over-budget text overflows the fixed box but the spec still renders.
  })

  // --- F5: at most one node per (row, col) ---
  const cells = resolveCells(nodes)
  const seenCell = new Map()
  for (const node of nodes) {
    const c = cells.get(node.id)
    if (!c) continue
    const key = `${c.row}:${c.col}`
    if (seenCell.has(key)) {
      errors.push(
        `nodes: "${node.id}" and "${seenCell.get(key)}" both occupy cell (row ${c.row}, col ${c.col}); ` +
          `set an explicit col to disambiguate ${cite('F5')}`,
      )
    } else {
      seenCell.set(key, node.id)
    }
  }

  // --- edges: references + guards (F1/F3/F4) ---
  const edges = spec.edges ?? []
  if (!Array.isArray(edges)) {
    errors.push('edges: must be a list of { from, to, text? }')
    return errors
  }
  const outByNode = new Map() // node id -> outgoing edges
  edges.forEach((edge, i) => {
    const at = `edges[${i}]`
    if (!edge || typeof edge !== 'object' || Array.isArray(edge)) {
      errors.push(`${at}: must be a mapping { from, to, text? }`)
      return
    }
    for (const key of ['from', 'to']) {
      if (!edge[key]) errors.push(`${at}: ${key} is required ${cite('F1')}`)
      else if (!ids.has(edge[key])) errors.push(`${at}: ${key} "${edge[key]}" is not a declared node ${cite('F1')}`)
    }
    if (edge.from && edge.to && edge.from === edge.to) {
      errors.push(`${at}: from and to are the same node ("${edge.from}"); self-loops are not supported in v1 ${cite('F1')}`)
    }
    if (edge.type != null && edge.type !== 'sync' && edge.type !== 'async') {
      errors.push(`${at}: type must be "sync" (default) or "async" (got "${edge.type}") ${cite('F6')}`)
    }
    if (edge.from) {
      const list = outByNode.get(edge.from) ?? []
      list.push(edge)
      outByNode.set(edge.from, list)
    }
  })

  // --- F4: a decision fans out with >= 2 guarded branches, each to a distinct target ---
  for (const node of nodes) {
    if (node?.kind !== 'decision') continue
    const out = outByNode.get(node.id) ?? []
    if (out.length < 2) {
      errors.push(`nodes: decision "${node.id}" needs >= 2 outgoing edges (branches) ${cite('F4')}`)
    }
    const seenTargets = new Set()
    for (const e of out) {
      if (!e.text || !String(e.text).trim()) {
        errors.push(
          `edges: branch ${node.id} -> ${e.to} from a decision needs a descriptive guard label ` +
            `(text) — the diamond has no text, so the condition must live on the arrow ${cite('F4')}`,
        )
      }
      // Two branches into the same node are not a choice — the outcomes must be distinct, or the
      // decision is pointless (and both labels pile onto one arrow). This also rejects a decision
      // that loops every branch back to an earlier node instead of reaching real outcomes.
      if (e.to) {
        if (seenTargets.has(e.to)) {
          errors.push(
            `edges: decision "${node.id}" sends more than one branch to "${e.to}"; each branch must ` +
              `reach a distinct outcome, or it is not a real choice ${cite('F4')}`,
          )
        } else {
          seenTargets.add(e.to)
        }
      }
    }
  }

  // --- F7: unbroken chain — one entry, all nodes connected; a pure async source may originate ---
  // Consider only well-formed edges (both endpoints declared, no self-loop); malformed refs are
  // already F1 errors and would only cascade noise here.
  const realEdges = edges.filter((e) => e && ids.has(e.from) && ids.has(e.to) && e.from !== e.to)
  const indeg = new Map([...ids].map((id) => [id, 0]))
  const outEdges = new Map([...ids].map((id) => [id, []]))
  const adj = new Map([...ids].map((id) => [id, new Set()])) // undirected adjacency
  for (const e of realEdges) {
    indeg.set(e.to, indeg.get(e.to) + 1)
    outEdges.get(e.from).push(e)
    adj.get(e.from).add(e.to)
    adj.get(e.to).add(e.from)
  }
  // A pure async source (a feed) has no incoming edge and every outgoing edge is async — it is a
  // legitimate standalone origin (it attaches to the systems it feeds via async edges), so it does
  // not count as a second entry.
  const isPureAsyncSource = (id) => {
    const outs = outEdges.get(id)
    return indeg.get(id) === 0 && outs.length > 0 && outs.every((e) => e.type === 'async')
  }
  const entries = [...ids].filter((id) => indeg.get(id) === 0 && !isPureAsyncSource(id))
  if (entries.length === 0) {
    errors.push(`nodes: no entry — every node has an incoming edge, so the flow has no start ${cite('F7')}`)
  } else if (entries.length > 1) {
    errors.push(
      `nodes: ${entries.length} entries (${entries.join(', ')}) have no incoming edge; a flow starts ` +
        `from ONE entry — connect the others into the chain, or make a background source's edges async ${cite('F7')}`,
    )
  }
  // Every node must join one connected picture (no isolated node / detached island). Async edges
  // count as connections, so a feed source is part of the graph.
  if (ids.size > 1) {
    const start = [...ids][0]
    const seen = new Set([start])
    const stack = [start]
    while (stack.length) {
      const n = stack.pop()
      for (const m of adj.get(n)) if (!seen.has(m)) { seen.add(m); stack.push(m) }
    }
    if (seen.size < ids.size) {
      const missing = [...ids].filter((id) => !seen.has(id))
      errors.push(
        `nodes: ${missing.join(', ')} not connected to the rest of the flow; every node must join one ` +
          `unbroken chain (a node with nowhere to connect does not belong on this diagram) ${cite('F7')}`,
      )
    }
  }

  // --- F8: no edge is routed through a non-endpoint node ---
  // A geometry check (unlike F1..F7, which are spec-level): only run once the spec is otherwise
  // valid, so `layout` is safe. When a source and a far target are collinear with a third node
  // between them, the edge is drawn straight across that node — reject it so the author fans the
  // targets onto different axes (see F8 in the reference).
  if (errors.length === 0) {
    const model = layout(spec)
    const rectById = new Map(model.nodes.map((nd) => [nd.id, nd.rect]))
    for (const e of model.edges) {
      const pts = [{ x: e.x1, y: e.y1 }, ...(e.waypoints ?? []), { x: e.x2, y: e.y2 }]
      for (const nd of model.nodes) {
        if (nd.id === e.from || nd.id === e.to) continue
        const r = rectById.get(nd.id)
        const inset = {
          x: r.x + THROUGH_INSET,
          y: r.y + THROUGH_INSET,
          w: r.w - 2 * THROUGH_INSET,
          h: r.h - 2 * THROUGH_INSET,
        }
        const crosses = pts.slice(0, -1).some((p, i) => segmentIntersectsRect(p, pts[i + 1], inset))
        if (crosses) {
          errors.push(
            `edges: ${e.from} -> ${e.to} is routed straight through node "${nd.id}"; place the targets ` +
              `on different axes (e.g. one down the spine, the other in a side column at the source's row) ` +
              `so the arrow does not cross a box ${cite('F8')}`,
          )
        }
      }
    }
  }

  return errors
}

/**
 * Soft text-budget advisories (F3). Unlike `validate`, these NEVER block a file: box width is a
 * fixed constant (see `grid.mjs`), so over-budget text simply overflows the box — the author is
 * told, but a strong model can also just read the render and trim. A `decision` carries no text,
 * so budgets apply to boxes only.
 * @param {import('../../lib/types.mjs').FlowSpec} spec
 * @returns {string[]}
 */
export function warnings(spec) {
  const warns = []
  const nodes = Array.isArray(spec?.nodes) ? spec.nodes : []
  nodes.forEach((node, i) => {
    if (!node || typeof node !== 'object' || node.kind === 'decision') return
    const at = `nodes[${i}]${node.id ? ` (${node.id})` : ''}`
    const title = node.title ?? node.id ?? ''
    if (String(title).length > F.TITLE_MAX_CHARS) {
      warns.push(`${at}: title over ${F.TITLE_MAX_CHARS} chars — it will overflow the box; shorten it ${cite('F3')}`)
    }
    if (node.subtitle != null && String(node.subtitle).length > F.SUBTITLE_MAX_CHARS) {
      warns.push(`${at}: subtitle over ${F.SUBTITLE_MAX_CHARS} chars — it will overflow the box; shorten it ${cite('F3')}`)
    }
  })
  const edges = Array.isArray(spec?.edges) ? spec.edges : []
  edges.forEach((edge, i) => {
    if (!edge || typeof edge !== 'object' || edge.text == null) return
    if (String(edge.text).length > LABEL_MAX_CHARS) {
      warns.push(`edges[${i}]: label over ${LABEL_MAX_CHARS} chars (wraps to 2 lines) — it may crowd the corridor; shorten it ${cite('F3')}`)
    }
  })
  return warns
}
