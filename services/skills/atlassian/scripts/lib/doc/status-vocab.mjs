/**
 * Single source of truth for the document status vocabulary and its Confluence
 * status-macro colours. Previously the words lived in doc/model, the colours in
 * doc/render, and a divergent copy in doc/profiles (which is how "on review"
 * drifted between Blue and Yellow). Everything now reads from here.
 *
 * Responsibility: the canonical status words + their lozenge colours. Pure.
 * Edit here when: you add a status or change a colour — it takes effect in the
 *   header renderer, the validator, and the profile badge maps at once.
 */

// The client-review slot. Which organisation reviews a document is DATA in the
// document (the "<Client> Approval" group label), never library configuration,
// so the vocabulary carries a placeholder that every consumer resolves against
// the name derived from the document it is handling.
export const CLIENT_REVIEW = '<client> review'

// Canonical lowercase status words (order matters only for the "known values"
// message the validator prints).
export const STATUS_VALUES = [
  'draft',
  'not started',
  'in progress',
  'on review',
  'in review',
  'astound review',
  CLIENT_REVIEW,
  'pending answers',
  'approved',
]

// Canonical status → Confluence status-macro colour. "on review"/"in review"
// render Blue to match the fabric page; unknown values fall back to Grey.
export const STATUS_COLOURS = {
  draft: 'Grey',
  'not started': 'Grey',
  'in progress': 'Yellow',
  'on review': 'Blue',
  'in review': 'Blue',
  'astound review': 'Blue',
  [CLIENT_REVIEW]: 'Purple',
  'pending answers': 'Yellow',
  approved: 'Green',
}

/** The client's review status word, e.g. "Acme Retail" → "acme retail review". */
export function clientReviewStatus(clientName) {
  const name = String(clientName || '').trim().toLowerCase()
  return name ? `${name} review` : ''
}

// Map an authored value onto its canonical key: the client's own review word
// folds back to the placeholder, everything else is compared as written.
function canonical(text, clientName) {
  const value = String(text || '').toLowerCase().trim()
  const clientReview = clientReviewStatus(clientName)
  return clientReview && value === clientReview ? CLIENT_REVIEW : value
}

/** The vocabulary as a document sees it — the placeholder resolved to the client. */
export function statusValues(clientName) {
  const clientReview = clientReviewStatus(clientName)
  return STATUS_VALUES.map((s) => (s === CLIENT_REVIEW && clientReview ? clientReview : s))
}

/** The colour map as a document sees it (used for backtick badges in the body). */
export function statusColours(clientName) {
  const clientReview = clientReviewStatus(clientName)
  const out = {}
  for (const [word, colour] of Object.entries(STATUS_COLOURS)) {
    out[word === CLIENT_REVIEW && clientReview ? clientReview : word] = colour
  }
  return out
}

/** Resolve a status word to its lozenge colour (Grey for anything unknown). */
export function statusColour(text, clientName) {
  return STATUS_COLOURS[canonical(text, clientName)] || 'Grey'
}

// Lookup set so a renderer can tell a status cell (→ lozenge) from a
// person/label cell (→ @mention / plain text).
export const STATUS_STRINGS = new Set(STATUS_VALUES.map((s) => s.toLowerCase()))

/** True when `text` (trimmed, case-insensitive) is a canonical status word. */
export function isStatus(text, clientName) {
  return STATUS_STRINGS.has(canonical(text, clientName))
}

// The approval groups follow the convention "<Delivering org> Approval" (always
// Astound) + "<Client org> Approval"; the first non-Astound group names the
// client. Falls back to a generic "Client" when no client group is present.
export function deriveClientName(model) {
  const sections = model?.header?.sections || []
  for (const s of sections.slice(1)) {
    const label = String(s.label || '').trim()
    if (/^astound\b/i.test(label)) continue
    const name = label.replace(/\s*approvals?\s*$/i, '').trim()
    if (name) return name
  }
  return 'Client'
}
