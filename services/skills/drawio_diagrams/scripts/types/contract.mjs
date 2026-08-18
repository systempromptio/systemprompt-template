/**
 * The diagram-type contract — the enforced shape of the extensibility seam.
 *
 * Responsibility: define what a diagram type module MUST provide and fail fast (at registry
 *   load) if one is incomplete, so a half-built type can never reach the pipeline.
 * Inputs/Outputs: a candidate module in, a validated {@link import('../lib/types.mjs').DiagramType} out (throws otherwise).
 * Edit here when: the uniform type interface itself changes (e.g. you add a required
 *   `parse` for reverse .drawio -> spec). Then update every registered type to match.
 * Do NOT: bake sequence-specific assumptions in here — this must stay type-agnostic.
 */

const REQUIRED_STRINGS = ['type', 'title']
const REQUIRED_FNS = ['validate', 'layout', 'emit']

/**
 * Assert a module satisfies the DiagramType contract; returns it unchanged when valid.
 * @param {any} mod
 * @returns {import('../lib/types.mjs').DiagramType}
 */
export function assertType(mod) {
  if (!mod || typeof mod !== 'object') {
    throw new TypeError(`diagram type must be an object, got ${typeof mod}`)
  }
  for (const key of REQUIRED_STRINGS) {
    if (typeof mod[key] !== 'string' || !mod[key]) {
      throw new TypeError(`diagram type "${mod.type ?? '?'}" is missing a non-empty string "${key}"`)
    }
  }
  for (const fn of REQUIRED_FNS) {
    if (typeof mod[fn] !== 'function') {
      throw new TypeError(`diagram type "${mod.type}" is missing required function "${fn}()"`)
    }
  }
  return mod
}
