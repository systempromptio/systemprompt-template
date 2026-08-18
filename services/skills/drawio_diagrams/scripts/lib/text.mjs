/**
 * Pure text utilities shared across diagram types.
 *
 * Responsibility: text shaping that must happen in the LAYOUT (never in styles), because
 *   the render-safe path forbids `whiteSpace=wrap` — the layout pre-splits multi-line text
 *   into explicit lines and the emitter joins them with hard breaks.
 * Inputs/Outputs: strings in, string[] out. No I/O, no DOM.
 * Edit here when: you need a new text helper reused by more than one type.
 * Do NOT: measure real glyph widths here (that is drawio2svg's job via text-measure.mjs);
 *   this is a deterministic character-count approximation only.
 */

const DEFAULT_MAX_CHARS = 40

/**
 * Wrap `text` to at most `maxChars` per line, honouring any explicit newlines. Word-based:
 * a single word longer than `maxChars` is kept on its own line rather than split.
 * @param {string} text
 * @param {number} [maxChars]
 * @returns {string[]} at least one line (possibly a single empty string)
 */
export function wrapText(text, maxChars = DEFAULT_MAX_CHARS) {
  const out = []
  for (const rawLine of String(text ?? '').split(/\r?\n/)) {
    const words = rawLine.split(/\s+/).filter(Boolean)
    if (words.length === 0) {
      out.push('')
      continue
    }
    let line = ''
    for (const w of words) {
      if (line && line.length + 1 + w.length > maxChars) {
        out.push(line)
        line = w
      } else {
        line = line ? `${line} ${w}` : w
      }
    }
    if (line) out.push(line)
  }
  return out.length ? out : ['']
}
