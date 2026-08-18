/**
 * Layout phase 0 — columns and participant boxes.
 *
 * Responsibility: place participant columns left-to-right (variable step: a gap that touches
 *   an actor is tighter than component<->component) and build each participant's geometry
 *   plus its pre-positioned, statically-centered header text cells (title + optional subtitle).
 * Inputs: {@link import('../../../lib/types.mjs').Spec}. Outputs: { participants, byId, columnCenter }.
 * Edit here when: you want to change column spacing, header sizing, or header-text centering.
 * Do NOT: position bars/arrows/labels here — those are later phases relative to `columnCenter`.
 */
import { L } from '../geometry.mjs'

/**
 * @param {import('../../../lib/types.mjs').Spec} spec
 * @returns {{ participants: import('../../../lib/types.mjs').PLayout[], byId: Map<string, import('../../../lib/types.mjs').PLayout>, columnCenter: (i:number)=>number }}
 */
export function buildColumns(spec) {
  const kindOf = (p) => p.kind ?? 'box'
  const centers = []
  let cx = L.MARGIN_X + L.HEADER_W / 2
  spec.participants.forEach((p, i) => {
    if (i > 0) {
      const touchesActor = kindOf(spec.participants[i - 1]) === 'actor' || kindOf(p) === 'actor'
      cx += L.COL_STEP * (touchesActor ? L.COL_STEP_ACTOR_FACTOR : L.COL_STEP_COMPONENT_FACTOR)
    }
    centers.push(cx)
  })
  const columnCenter = (i) => centers[i]

  const participants = spec.participants.map((p, i) => {
    const kind = p.kind ?? 'box'
    const isActor = kind === 'actor'
    const halfW = (isActor ? L.ACTOR_W : L.HEADER_W) / 2
    const xCenter = columnCenter(i)
    // Box headers are taller than the actor figure (to visually match the actor's
    // name-above-figure extent); they grow upward so all header bottoms share a baseline.
    const headerH = isActor ? L.HEADER_H : L.HEADER_BOX_H
    const headerBaseline = L.MARGIN_TOP + L.HEADER_H
    const headerTop = headerBaseline - headerH
    const title = p.title ?? p.id
    const subtitle = p.subtitle ?? null

    // Header text is a static, vertically-centered group of one or two lines. A box centers
    // the group inside its rectangle; an actor stacks it just above the stick figure. With a
    // subtitle present the title simply shifts up — the box never resizes.
    const groupH = subtitle ? L.TITLE_LH + L.SUBTITLE_LH : L.TITLE_LH
    const textW = L.HEADER_W // both actor and box center their text on the box width
    const textLeft = xCenter - textW / 2
    const groupTop = isActor
      ? headerTop - groupH - L.ACTOR_TEXT_GAP // small breathing room above the stick figure
      : headerTop + (L.HEADER_BOX_H - groupH) / 2
    const titleCell = { x: textLeft, y: groupTop, w: textW, h: L.TITLE_LH }
    const subtitleCell = subtitle
      ? { x: textLeft, y: groupTop + L.TITLE_LH, w: textW, h: L.SUBTITLE_LH }
      : null

    return {
      id: p.id,
      title,
      subtitle,
      titleCell,
      subtitleCell,
      kind,
      index: i,
      isActor,
      hasBar: !isActor,
      xCenter,
      width: isActor ? L.ACTOR_W : L.HEADER_W,
      left: xCenter - halfW,
      right: xCenter + halfW,
      headerTop,
      headerH,
    }
  })
  const byId = new Map(participants.map((p) => [p.id, p]))
  return { participants, byId, columnCenter }
}
