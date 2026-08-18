/**
 * Flow authoring-rule catalog — the single source of truth for rule ids and their docs.
 *
 * Responsibility: hold each rule id (F1..F5) with a one-line summary, and render a consistent
 *   citation suffix pointing back at the authoring reference, so the validator error text and
 *   the authoring guide can never drift.
 * Inputs/Outputs: constants + a `cite(id)` formatter.
 * Edit here when: you add/rename a rule. The rules themselves are documented in
 *   `references/flow.md` (the user-facing authoring guide).
 * Do NOT: hand-build citation strings in the validator — always use `cite()`.
 */

/** Authoring guide these citations point at (path from the project root on a real project). */
const REFERENCE = '.cursor/skills/plan/drawio-diagrams/references/flow.md'

/** @type {Record<'F1'|'F2'|'F3'|'F4'|'F5'|'F6'|'F7'|'F8', {summary: string}>} */
export const RULES = {
  F1: { summary: 'unique node ids; every edge from/to references a declared node' },
  F2: { summary: 'each node has an integer row >= 0 (optional integer col >= 0)' },
  F3: { summary: 'title/subtitle/label text budgets (boxes only — a decision carries no text)' },
  F4: { summary: 'a decision (a bare diamond) has >= 2 outgoing edges, each with a descriptive guard label' },
  F5: { summary: 'at most one node per (row, col) cell' },
  F6: { summary: 'edge type is sync (default) or async (async renders dashed — a background/scheduled relationship)' },
  F7: { summary: 'unbroken chain: one entry (no incoming edge), all nodes connected; a pure async source (a feed) may originate on its own' },
  F8: { summary: 'no edge is routed through a non-endpoint node (place targets on different axes so arrows never cross a box)' },
}

/**
 * Render the trailing rule citation appended to a validator error, e.g.
 *   `[F1 — see references/flow.md → F1]`
 * @param {keyof typeof RULES} id
 * @returns {string}
 */
export function cite(id) {
  return `[${id} — see ${REFERENCE} → ${id}]`
}
