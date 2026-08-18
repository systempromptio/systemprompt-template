/**
 * Sequence-diagram geometry — the spatial constants the layout derives everything from.
 *
 * Responsibility: all sequence-specific numbers (widths, heights, gaps, margins, char
 *   budgets, line-heights). Columns = participants, rows = messages; every bar, arrow and
 *   label is positioned relative to these so the whole diagram compacts together.
 * Inputs/Outputs: pure constants (`L`).
 * Edit here when: you want to change sequence spacing/sizing or a character budget. For
 *   colours/fonts/strokes (shared across all types) edit `lib/design.mjs`; for how a style
 *   string is assembled, edit `styles.mjs`.
 * Do NOT: reintroduce colour/font tokens here — they moved to `lib/design.mjs` so a future
 *   type can share them.
 */

export const L = {
  MARGIN_X: 24,
  MARGIN_TOP: 48, // headroom so an actor's name fits above its stick figure
  HEADER_W: 128, // box-participant header width (narrow)
  HEADER_H: 40, // actor stick-figure band height
  HEADER_BOX_H: 57, // box-participant header height (taller: matches actor name + figure)
  // Two-tier participant text (title + optional subtitle). Box width is FIXED; text must
  // fit, so we cap characters (see validate). Limits are a char-count proxy derived from
  // HEADER_W at the two font sizes (title ~15; the smaller subtitle font fits a couple more).
  TITLE_MAX_CHARS: 15,
  SUBTITLE_MAX_CHARS: 22,
  // Message label limits. Call/return labels are ALWAYS single-line (no wrap) and short. A
  // self label may be longer: it renders in a FIXED width (bounded by the next participant
  // column so it never overlaps) and wraps onto a second, left-aligned line.
  MSG_LABEL_MAX_CHARS: 20, // call/return, single line
  SELF_LABEL_MAX_CHARS: 40, // self, wraps within a fixed width
  SELF_LABEL_MARGIN: 18, // keep-out gap before the next participant's lifeline
  SELF_LABEL_DEFAULT_W: 160, // width to use when the self is on the last participant
  SELF_LABEL_MIN_W: 60, // never shrink the label box below this
  SELF_LABEL_CHAR_PX: 6.5, // approx px per char used to decide the wrap column
  SELF_LABEL_LH: 15, // self label line height
  TITLE_LH: 17, // title line-box height
  SUBTITLE_LH: 15, // subtitle line-box height
  HEADER_TEXT_PAD: 8, // inner horizontal breathing room so text never touches the border
  ACTOR_TEXT_GAP: 5, // vertical gap between an actor's name group and its stick figure
  ACTOR_W: 30, // stick-figure width (narrow; column center is unchanged)
  COL_STEP: 250, // base horizontal distance between participant column centers
  COL_STEP_ACTOR_FACTOR: 0.7, // actor <-> component gap = 70% of COL_STEP (30% tighter)
  COL_STEP_COMPONENT_FACTOR: 0.8, // component <-> component gap = 80% of COL_STEP (20% tighter)
  MSG_TOP_GAP: 54, // header bottom -> first message row
  ROW_STEP: 46, // fallback vertical distance (orphan/unpaired messages only)
  // Activation-bar geometry is computed recursively from the call tree (see layout):
  //   leaf frame (no nested activity)        -> MIN_ACTIVATION_HEIGHT
  //   container frame (has nested activity)   -> ACTIVATION_GAP + <children> + ACTIVATION_GAP
  //   gap between two sibling child frames    -> SIBLING_GAP
  // Bar height is NOT measured from arrow positions; arrows attach to the computed edges
  // (call -> top edge, return -> bottom edge).
  ACTIVATION_GAP: 10, // top/bottom inner pad of a container frame (first child hugs the top)
  MIN_ACTIVATION_HEIGHT: 40, // fixed height of a leaf frame (a call with nothing nested)
  SIBLING_GAP: 40, // vertical gap between consecutive child frames (== MIN_ACTIVATION_HEIGHT by design)
  SELF_SIBLING_GAP: 20, // half of SIBLING_GAP — tighter gap where a self block abuts a sibling action
  // A self-message renders as a small nested activation rectangle sitting on the main bar,
  // with a short hook arrow from the big bar into it (rectangle-on-rectangle).
  SELF_NEST_H: 30, // nested self-activation height (0.75 * MIN_ACTIVATION_HEIGHT)
  SELF_NEST_DX: 5, // how far the nested rectangle is offset to the right of the main bar
  SELF_LOOP_W: 30, // outward horizontal extent of the self-loop hook (small)
  BAR_W: 10, // activation bar width
  LABEL_GAP: 10, // constant px gap between a message label and its arrowhead (length-independent)
  LIFELINE_PAD: 42, // space below last content before lifeline ends
  NOTE_GAP: 26, // gap below lifelines before notes
  NOTE_STACK_GAP: 12,
  NOTE_LINE_H: 16,
  NOTE_PAD_Y: 14,
  NOTE_MIN_H: 40,
  // A note is anchored under exactly ONE participant, centered on its column, at a fixed max
  // width of 1.5x the participant box. Text is capped and wraps within that width. Several
  // notes may exist; they all share one top y (a single horizontal band below the lifelines).
  NOTE_MAX_W: Math.round(128 * 1.5), // 1.5 * HEADER_W
  NOTE_MAX_CHARS: 70,
  NOTE_CHAR_PX: 6.5, // approx px per char used to decide the wrap column
}
