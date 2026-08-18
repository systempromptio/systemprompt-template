/**
 * Sequence-diagram render-safe styles — the mxGraph style strings the emitter attaches.
 *
 * Responsibility: assemble every cell/edge style from the shared design tokens
 *   (`lib/design.mjs`), so visual identity lives in one place and this file only wires
 *   tokens into the exact style fragments draw.io understands.
 * Inputs/Outputs: design tokens in, style strings (`S`) out.
 * Edit here when: you need a new styled element, or to change how an element maps tokens to
 *   a style string. To change a colour/weight globally, edit `lib/design.mjs` instead.
 * Do NOT — RENDER-SAFE RULE: never add `html=1` or `whiteSpace=wrap`. Those make drawio2svg
 *   emit <foreignObject> labels that the browserless resvg rasterizer silently drops. Only
 *   native SVG <text> is allowed; wrapping is pre-computed by the layout (see lib/text.mjs).
 */
import { COLORS, FONT, TYPE, STROKE, RADIUS } from '../../lib/design.mjs'

const { INK, INK_BODY, MUTED, LINE, ACCENT, PANEL } = COLORS

export const S = {
  lifeline: (headerH) =>
    `shape=umlLifeline;perimeter=lifelinePerimeter;container=0;collapsible=0;` +
    `recursiveResize=0;outlineConnect=0;portConstraint=eastwest;rounded=1;absoluteArcSize=1;arcSize=${RADIUS};` +
    `strokeColor=${INK};strokeWidth=${STROKE.EMPHASIS};align=center;fontStyle=1;size=${headerH};${FONT}`,
  // Actor lifeline: stick figure in the header band; drawn in the accent colour so the eye
  // lands on the human/primary path (the one reserved use of colour).
  actorLifeline: (headerH) =>
    `shape=umlLifeline;participant=umlActor;perimeter=lifelinePerimeter;container=0;` +
    `collapsible=0;recursiveResize=0;outlineConnect=0;portConstraint=eastwest;` +
    `strokeColor=${ACCENT};strokeWidth=${STROKE.EMPHASIS};align=center;verticalLabelPosition=top;verticalAlign=bottom;fontStyle=1;size=${headerH};${FONT}`,
  call: `endArrow=block;endFill=1;endSize=7;curved=0;rounded=0;html=0;verticalAlign=bottom;strokeColor=${INK_BODY};strokeWidth=${STROKE.EMPHASIS};fontColor=${INK_BODY};${FONT}`,
  // Return: dashed + OPEN head (UML reply), muted grey, regular weight (no italic — it loses
  // legibility when rasterized small).
  return: `endArrow=open;endFill=0;dashed=1;dashPattern=6 4;endSize=8;curved=0;rounded=0;html=0;verticalAlign=bottom;strokeColor=${MUTED};strokeWidth=${STROKE.RETURN};fontColor=${MUTED};${FONT}`,
  self: `endArrow=block;endFill=1;curved=0;rounded=0;html=0;verticalAlign=middle;align=left;strokeColor=${INK_BODY};strokeWidth=${STROKE.EMPHASIS};${FONT}`,
  // Standalone native-text label (no box) for self-message text, placed deterministically.
  selfLabel: `text;html=0;strokeColor=none;fillColor=none;align=left;verticalAlign=middle;fontColor=${INK_BODY};${FONT}`,
  // Participant header text: a native-text title (bold) and optional subtitle (smaller, dark
  // grey) emitted as separate cells so the pair can be centered as a static group.
  title: `text;html=0;strokeColor=none;fillColor=none;align=center;verticalAlign=middle;fontStyle=1;fontSize=${TYPE.TITLE_FS};fontColor=${INK};${FONT}`,
  titleActor: `text;html=0;strokeColor=none;fillColor=none;align=center;verticalAlign=middle;fontStyle=1;fontSize=${TYPE.TITLE_FS};fontColor=${ACCENT};${FONT}`,
  subtitle: `text;html=0;strokeColor=none;fillColor=none;align=center;verticalAlign=middle;fontStyle=0;fontSize=${TYPE.SUBTITLE_FS};fontColor=${MUTED};${FONT}`,
  activation:
    'points=[[0,0,0,0,5],[0,1,0,0,-5],[1,0,0,0,5],[1,1,0,0,-5]];' +
    'perimeter=orthogonalPerimeter;outlineConnect=0;targetShapes=umlLifeline;' +
    `portConstraint=eastwest;rounded=1;fillColor=#ffffff;strokeColor=${LINE};strokeWidth=${STROKE.HAIRLINE};`,
  note:
    `shape=note;size=14;rounded=0;shadow=0;spacingLeft=6;spacingRight=6;spacingTop=4;` +
    `align=left;verticalAlign=top;strokeColor=${LINE};fillColor=${PANEL};fontColor=${INK_BODY};${FONT}`,
}
