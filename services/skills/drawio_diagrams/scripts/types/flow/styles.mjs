/**
 * Flow-diagram render-safe styles — the mxGraph style strings the flow emitter attaches.
 *
 * Responsibility: assemble every cell/edge style from the shared design tokens
 *   (`lib/design.mjs`), reusing sequence's visual language (same ink, accent-free, stroke
 *   hierarchy, corner radius, two-tier header text) so flow reads as part of the same family.
 * Inputs/Outputs: design tokens in, style strings (`S`) out.
 * Edit here when: you need a new styled element, or to change how an element maps tokens to a
 *   style string. To change a colour/weight globally, edit `lib/design.mjs`.
 * Do NOT — RENDER-SAFE RULE: never add `html=1` or `whiteSpace=wrap`. Those make drawio2svg
 *   emit <foreignObject> labels the browserless resvg rasterizer drops. Only native SVG <text>;
 *   node/label text is pre-split by the layout (see lib/text.mjs).
 */
import { COLORS, FONT, TYPE, STROKE, RADIUS } from '../../lib/design.mjs'
import { F } from './geometry.mjs'

const { INK, INK_BODY, MUTED, PANEL } = COLORS

export const S = {
  // Box participant: a rounded rectangle drawn like a sequence header box (navy border, subtle
  // panel fill). Text is emitted as SEPARATE native-text cells (title/subtitle) so the pair can
  // be centered as a static group — the shape itself carries no label.
  box:
    `rounded=1;absoluteArcSize=1;arcSize=${RADIUS};strokeColor=${INK};strokeWidth=${STROKE.EMPHASIS};` +
    `fillColor=${PANEL};${FONT}`,
  // Decision: a rhombus in the same ink/fill; text is again separate native-text cells.
  decision: `rhombus;strokeColor=${INK};strokeWidth=${STROKE.EMPHASIS};fillColor=${PANEL};${FONT}`,
  // Two-tier node text (mirrors sequence): title bold navy, optional subtitle smaller grey.
  title: `text;html=0;strokeColor=none;fillColor=none;align=center;verticalAlign=middle;fontStyle=1;fontSize=${TYPE.TITLE_FS};fontColor=${INK};${FONT}`,
  subtitle: `text;html=0;strokeColor=none;fillColor=none;align=center;verticalAlign=middle;fontStyle=0;fontSize=${TYPE.SUBTITLE_FS};fontColor=${MUTED};${FONT}`,
  // Default (synchronous) arrow: solid, block head, emphasis weight.
  edge: `endArrow=block;endFill=1;endSize=${F.ARROW_END_SIZE};curved=0;rounded=0;html=0;strokeColor=${INK_BODY};strokeWidth=${STROKE.EMPHASIS};${FONT}`,
  // Async arrow: the same block head/weight but DASHED, signalling out-of-band / background work
  // (a scheduled feed, a batch ingest, an event) rather than a synchronous request. Same
  // dashPattern as the sequence `return` so the family reads consistently; dashed is render-safe
  // (no html=1), unlike wrapped labels.
  edgeAsync: `endArrow=block;endFill=1;endSize=${F.ARROW_END_SIZE};curved=0;rounded=0;html=0;dashed=1;dashPattern=6 4;strokeColor=${INK_BODY};strokeWidth=${STROKE.EMPHASIS};${FONT}`,
  // Standalone native-text label placed beside its arrow. The emitter appends the per-edge
  // horizontal anchor (`align=left|center|right;`) since it depends on which side the label sits.
  edgeLabel: `text;html=0;strokeColor=none;fillColor=none;verticalAlign=middle;fontColor=${INK_BODY};${FONT}`,
}
