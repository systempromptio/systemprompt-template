/**
 * Flow-diagram geometry — the spatial constants the flow layout derives everything from.
 *
 * Responsibility: all flow-specific numbers (box sizing, grid gaps, char budgets, lane/label
 *   spacing). Unlike sequence (fixed-width headers), flow boxes are CONTENT-sized: width grows
 *   with the text (clamped), so a box with a subtitle is wider than a bare one.
 * Inputs/Outputs: pure constants (`F`).
 * Edit here when: you want to change flow spacing/sizing or a character budget. For
 *   colours/fonts/strokes (shared across all types) edit `lib/design.mjs`; for how a style
 *   string is assembled, edit `styles.mjs`.
 * Do NOT: reintroduce colour/font tokens here — they live in `lib/design.mjs`.
 *
 * NOTE on text widths: like sequence's notes/self labels, box/label widths use a deterministic
 * character-count * px heuristic (never real glyph metrics), so output is byte-identical across
 * machines. The per-char values slightly OVER-estimate so text never overflows its box.
 */

// Fixed box width. Every box (and the decision diamond) is exactly this wide so the whole diagram
// lines up as a uniform grid. Chosen compact (close to the old "Load all tags" size) yet wide
// enough that a normal 16-char title still fits; the F3 budgets below are derived from it.
const BOX_W = 160

export const F = {
  MARGIN_X: 24,
  MARGIN_Y: 24,
  // Grid gaps: the empty space between a column/row and the next. ROW_GAP must comfortably fit
  // a vertical arrow plus its side label; COL_GAP fits a horizontal arrow plus its label.
  COL_GAP: 130,
  ROW_GAP: 82, // trimmed ~15% off the original 96 to tighten the vertical rhythm
  // Box: FIXED width AND height. Every box is identical in size; the title+subtitle group is
  // centered inside. Text that would overflow is rejected up front by the F3 budgets below.
  BOX_W,
  BOX_H: 64,
  BOX_PAD_X: 18,
  TITLE_LH: 20, // title line-box height
  SUBTITLE_LH: 16, // subtitle line-box height
  // F3 text budgets, sized to fit BOX_W (chars * px/char + 2*PAD_X <= BOX_W):
  TITLE_MAX_CHARS: 16, // 16*7.6 + 2*18 ~= 158 <= 160
  SUBTITLE_MAX_CHARS: 18, // 18*6.2 + 2*18 ~= 148 <= 160
  // Decision rhombus: carries NO text (all meaning is on the branch arrows). Same WIDTH as a box
  // so the spine stays aligned, a touch taller for a clean diamond.
  DECISION_W: BOX_W,
  DECISION_H: 90,
  // Edges.
  LANE_GAP: 26, // spacing between parallel arrows that share one node side
  ARROW_END_SIZE: 7,
  LABEL_SIDE_GAP: 8, // perpendicular gap between a label and its arrow line
  LABEL_LH: 15, // edge-label line height
  LABEL_CHAR_PX: 6.4, // approx px/char for edge labels (default font size)
  LABEL_MAX_CHARS: 28, // an edge label wraps beyond this many chars (F3)
}
