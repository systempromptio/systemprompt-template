/**
 * Sequence authoring-rule catalog — the single source of truth for rule ids and their docs.
 *
 * Responsibility: hold each rule id (R1..R6) with a one-line summary, and render a consistent
 *   citation suffix pointing back at the authoring reference. Keeping ids here (not inline in
 *   validate) means the error text and the authoring guide can never drift.
 * Inputs/Outputs: constants + a `cite(id)` formatter.
 * Edit here when: you add/rename a rule. The rules themselves are documented in
 *   `references/sequence.md` (the user-facing authoring guide).
 * Do NOT: hand-build citation strings in the validator — always use `cite()`.
 */

/** Authoring guide these citations point at (path from the project root on a real project). */
const REFERENCE = '.cursor/skills/plan/drawio-diagrams/references/sequence.md'

/** @type {Record<'R1'|'R2'|'R3'|'R4'|'R5'|'R6', {summary: string}>} */
export const RULES = {
  R1: { summary: 'one outgoing call in flight per flow' },
  R2: { summary: 'every call is closed by a return' },
  R3: { summary: 'participant text fits the header box' },
  R4: { summary: 'message label budgets' },
  R5: { summary: 'notes anchor under one participant' },
  R6: { summary: 'calls flow left→right; only returns go right→left' },
}

/**
 * Render the trailing rule citation appended to a validator error, e.g.
 *   `[R3 — see references/sequence.md → R3]`
 * @param {keyof typeof RULES} id
 * @returns {string}
 */
export function cite(id) {
  return `[${id} — see ${REFERENCE} → ${id}]`
}
