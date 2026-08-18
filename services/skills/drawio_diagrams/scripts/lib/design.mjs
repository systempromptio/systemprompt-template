/**
 * Shared design tokens — the type-agnostic visual language for every diagram type.
 *
 * Responsibility: the single source of truth for colour, typography, stroke hierarchy and
 *   corner radius. Any diagram type (sequence today, activity later) builds its style
 *   strings from these tokens so the whole family reads as one coherent system.
 * Inputs/Outputs: none — pure constants.
 * Edit here when: you want to change the palette, fonts, stroke weights, or radius across
 *   ALL diagram types. For spacing/geometry of a specific type, edit that type's geometry.
 * Do NOT: put per-type spatial numbers (widths, gaps, char budgets) here — those live in
 *   `types/<type>/geometry.mjs`.
 *
 * Approach (from the UX/UI research): monochrome near-black ink + ONE accent; message TYPES
 * are distinguished by line style + arrowhead, never by colour, so output is colourblind-
 * and print-safe. All text tokens pass WCAG AA on white.
 */

/** Colour palette. */
export const COLORS = {
  INK: '#172B4D', // near-black navy: titles, box borders (avoids harsh pure #000)
  INK_BODY: '#374151', // call/self labels, notes
  MUTED: '#666666', // subtitles, return labels, captions (5.7:1 on white)
  LINE: '#8A94A6', // lifelines, activation borders (>=3:1 non-text)
  ACCENT: '#0C66E4', // the single accent: actor / primary path
  PANEL: '#F7F8F9', // subtle note fill
}

/** Font family. Bundled Inter (see render.mjs) => byte-identical output on every OS. */
export const FONT_FAMILY = 'Inter'

/** The mxGraph style fragment that pins the font family. */
export const FONT = `fontFamily=${FONT_FAMILY};`

/**
 * Type scale — only the font sizes actually emitted as `fontSize=` in styles. Message/label
 * styles intentionally omit a size and inherit draw.io's default; do not add sizes to them
 * without a deliberate visual change (it would break the emit golden).
 */
export const TYPE = {
  TITLE_FS: 13, // participant title (bold)
  SUBTITLE_FS: 11, // participant subtitle (0.85 * title)
}

/**
 * Stroke-width hierarchy — the main "polished vs amateur" signal. Consistency of these three
 * weights carries most of the professional feel.
 */
export const STROKE = {
  EMPHASIS: 1.5, // box borders + synchronous call arrows + self arrows
  RETURN: 1.25, // return arrows (dashed, open head)
  HAIRLINE: 1, // lifelines + activation-bar borders
}

/** Corner radius for boxes (absolute px; flat, no gradients/shadows). */
export const RADIUS = 7
