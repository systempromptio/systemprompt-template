/**
 * Shared test fixtures + tiny assertion helpers.
 * Specs here are self-contained (not read from disk) so tests are hermetic.
 */
import assert from 'node:assert/strict'

/** A minimal, strictly-valid sequence spec that satisfies every rule (R1–R6). */
export function validSpec(overrides = {}) {
  return {
    type: 'sequence',
    id: 'fixture',
    title: 'Fixture',
    participants: [
      { id: 'u', title: 'Shopper', kind: 'actor' },
      { id: 'a', title: 'Web Browser' },
      { id: 'b', title: 'SF Next', subtitle: 'Edge runtime' },
    ],
    messages: [
      { kind: 'call', from: 'u', to: 'a', text: 'Open' },
      { kind: 'call', from: 'a', to: 'b', text: 'GET /' },
      { kind: 'self', from: 'b', text: 'Read cache' },
      { kind: 'return', from: 'b', to: 'a', text: 'HTML' },
      { kind: 'return', from: 'a', to: 'u', text: 'Rendered' },
    ],
    notes: [{ under: 'b', text: 'Edge SSR once per request.' }],
    ...overrides,
  }
}

/**
 * A minimal, valid `flow` spec — the consent-integration diagram used as the flow golden.
 * Boxes only (one with a subtitle) + two parallel arrows sharing a node side (cmp <-> sfnext) so
 * lane distribution and opposite-side labels are covered.
 */
export function flowSpec(overrides = {}) {
  return {
    type: 'flow',
    id: 'consent-integration',
    title: 'Consent integration overview',
    nodes: [
      { id: 'shopper', row: 0, title: 'Shopper' },
      { id: 'cmp', row: 1, title: 'CMP' },
      { id: 'gtm', row: 1, title: 'GTM/Adobe', subtitle: '3rd-party tags' },
      { id: 'sfnext', row: 2, title: 'SF Next' },
    ],
    edges: [
      { from: 'shopper', to: 'cmp', text: 'accept/reject/custom' },
      { from: 'cmp', to: 'gtm', text: 'native integration' },
      { from: 'cmp', to: 'sfnext', text: 'notify' },
      { from: 'sfnext', to: 'cmp', text: 'subscribe' },
    ],
    ...overrides,
  }
}

/** Assert that at least one error string contains `needle`. */
export function assertHasError(errors, needle) {
  assert.ok(
    errors.some((e) => e.includes(needle)),
    `expected an error containing ${JSON.stringify(needle)}, got:\n  ${errors.join('\n  ') || '(none)'}`,
  )
}

/** Assert there are no errors. */
export function assertNoErrors(errors) {
  assert.deepEqual(errors, [], `expected no errors, got:\n  ${errors.join('\n  ')}`)
}
