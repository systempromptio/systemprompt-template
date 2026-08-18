/**
 * Minimal XML helpers for emitting mxGraph (.drawio) markup.
 *
 * Responsibility: escaping, multi-line label encoding, and compact number formatting shared
 *   by every type's emitter.
 * Edit here when: you need another primitive shared across emitters.
 * Do NOT: build element/style strings here — that is each emitter's job.
 */

/** Escape a string for use in XML text / attribute values. */
export function esc(value) {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

/**
 * Encode a multi-line label for an mxCell `value` attribute.
 * Newlines become `&#10;` (draw.io's line break in non-HTML labels).
 */
export function multiline(lines) {
  return lines.map((l) => esc(l)).join('&#10;')
}

/** Round to at most 2 decimals and drop trailing zeros, for compact coords. */
export function n(value) {
  return Number.isFinite(value) ? String(Math.round(value * 100) / 100) : '0'
}
