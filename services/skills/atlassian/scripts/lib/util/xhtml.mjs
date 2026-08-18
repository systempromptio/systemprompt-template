/**
 * XHTML/regex escaping primitives shared by every storage-format producer
 * (doc/md-to-storage, doc/render, doc/publish).
 *
 * Responsibility: the ONE place that turns a raw string into safe Confluence
 *   storage text/attribute content. Pure — no I/O, no Confluence knowledge.
 * Edit here when: escaping rules change (and they change EVERYWHERE at once).
 * Invariant: every dynamic value emitted into storage XHTML goes through escHtml
 *   (text) or escAttr (attribute). Do NOT reimplement these inline.
 */

// Punctuation passes through untouched: the page carries what the author wrote,
// and a pull carries it back, so the markdown round-trips without a rewrite.
/** Escape text content for storage XHTML (&, <, >). */
export const escHtml = (s) =>
  String(s == null ? '' : s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')

/** Escape an attribute value (escHtml plus double-quote). */
export const escAttr = (s) => escHtml(s).replace(/"/g, '&quot;')

/** Escape a string for safe interpolation into a RegExp source. */
export const escapeRegExp = (s) => String(s == null ? '' : s).replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
