/**
 * Diagram type registry — the extensibility seam.
 *
 * Responsibility: hold the map of `type -> DiagramType` module and expose lookups. Each entry
 *   is verified against the contract at load (via assertType), so an incomplete type fails
 *   fast here instead of somewhere deep in the pipeline. The core generate/validate/render
 *   stay type-agnostic.
 * Inputs/Outputs: `getType(type)` / `listTypes()`.
 * Edit here when: you add a diagram type — add `./<type>/` exporting the uniform interface
 *   { type, title, validate, layout, emit } and register it in `registry` below.
 * Do NOT: reach into a type's internal modules from the core — only this uniform interface.
 */
import { assertType } from './contract.mjs'
import sequence from './sequence/index.mjs'
import flow from './flow/index.mjs'

const registry = {
  [sequence.type]: assertType(sequence),
  [flow.type]: assertType(flow),
}

/**
 * @param {string} type
 * @returns {import('../lib/types.mjs').DiagramType | null}
 */
export function getType(type) {
  return registry[type] || null
}

/** @returns {string[]} the registered type names */
export function listTypes() {
  return Object.keys(registry)
}

export default registry
