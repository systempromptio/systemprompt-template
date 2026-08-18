/**
 * Layout phase 4 — message geometry (arrows + labels).
 *
 * Responsibility: turn each message into drawable geometry using the resolved rows and the
 *   sized bars. Call/return arrows attach to bar edges and pin their label a CONSTANT pixel
 *   gap before the arrowhead (length-independent). A self renders as a small nested rectangle
 *   entered by a short hook, with a fixed-width, pre-wrapped label bounded by the next column.
 * Inputs: { src, byId, participants, columnCenter, rows }.
 * Outputs: { messages, selfActivations, contentRight }.
 * Edit here when: you change arrow endpoints, label gaps, or the self-loop shape/label.
 * Do NOT: change vertical rows/heights here — those come from the activations phase.
 */
import { L } from '../geometry.mjs'
import { wrapText } from '../../../lib/text.mjs'

// x of an arrow endpoint on `part` when connecting to `other`: the near edge of part's
// activation bar, or its lifeline center if it's an actor.
const endpointX = (part, other) => {
  if (!part.hasBar) return part.xCenter
  return other.index > part.index ? part.xCenter + L.BAR_W / 2 : part.xCenter - L.BAR_W / 2
}

/**
 * @param {{ src: import('../../../lib/types.mjs').Message[], byId: Map<string, import('../../../lib/types.mjs').PLayout>, participants: import('../../../lib/types.mjs').PLayout[], columnCenter: (i:number)=>number, rows: {y:number}[] }} args
 * @returns {{ messages: import('../../../lib/types.mjs').MsgLayout[], selfActivations: import('../../../lib/types.mjs').Rect[], contentRight: number }}
 */
export function buildMessages({ src, byId, participants, columnCenter, rows }) {
  let contentRight = Math.max(...participants.map((p) => p.right))
  const selfActivations = [] // small nested rectangles for self-messages

  const messages = src.map((m, i) => {
    const from = byId.get(m.from)
    const y = rows[i].y
    if (m.kind === 'self') {
      // A self-message is a small nested activation sitting on the main bar, entered by a
      // short hook arrow from the big bar (rectangle-on-rectangle). Geometry (all from known
      // coordinates, no measuring): main bar right edge -> out SELF_LOOP_W -> down
      // SELF_LOOP_DROP -> back into the nested rect's right edge (arrowhead).
      const xc = from.xCenter
      const hasBar = from.hasBar
      const mainR = hasBar ? xc + L.BAR_W / 2 : xc
      const nestX = (hasBar ? xc - L.BAR_W / 2 : xc) + L.SELF_NEST_DX
      const nestR = nestX + L.BAR_W
      const outX = mainR + L.SELF_LOOP_W
      // Every self renders its own nested activation rectangle (a "block") on the main bar; the
      // hook's arrowhead enters that block. This holds even when the self is immediately
      // followed by an outgoing call — the internal work still gets its own visible frame.
      const endX = nestR
      if (hasBar) selfActivations.push({ x: nestX, y, w: L.BAR_W, h: L.SELF_NEST_H })
      // Hook starts in the gap zone ABOVE the block (at blockTop - ACTIVATION_GAP) and its
      // arrowhead enters the block at mid-height.
      const hookY = y - L.ACTIVATION_GAP
      const drop = L.ACTIVATION_GAP + Math.round(L.SELF_NEST_H * 0.5)
      const labelX = outX + L.LABEL_GAP
      // Fixed width bounded by the next participant's LIFELINE (not its header box — down here
      // that column is just a thin dashed line), minus a small keep-out. This gives the label
      // the full inter-column gap; it then wraps (left-aligned) to fit.
      const nextIdx = from.index + 1
      const maxRight =
        nextIdx < participants.length
          ? columnCenter(nextIdx) - L.SELF_LABEL_MARGIN
          : labelX + L.SELF_LABEL_DEFAULT_W
      const labelW = Math.max(L.SELF_LABEL_MIN_W, maxRight - labelX)
      const perLine = Math.max(6, Math.floor(labelW / L.SELF_LABEL_CHAR_PX))
      const lines = wrapText(m.text ?? '', perLine)
      const labelH = lines.length * L.SELF_LABEL_LH
      const labelY = hookY + drop / 2 - labelH / 2 // vertically centered on the hook's arm
      contentRight = Math.max(contentRight, labelX + labelW)
      return {
        kind: 'self',
        lines,
        y,
        startX: mainR,
        outX,
        endX,
        hookY,
        drop,
        labelX,
        labelW,
        labelY,
        labelH,
        nestH: L.SELF_NEST_H,
      }
    }
    const to = byId.get(m.to)
    const x1 = endpointX(from, to)
    const x2 = endpointX(to, from)
    // Pin the label a CONSTANT pixel gap before the arrowhead (target), regardless of arrow
    // length. mxGraph edge labels use a relative position in [-1,1] (-1 = source, +1 =
    // target); solve for the position that sits LABEL_GAP px before the target. A relative
    // offset (e.g. 0.75) would make the gap scale with length -> uneven.
    const length = Math.abs(x2 - x1) || 1
    // Return labels get a slightly larger gap from the arrowhead (30%) than calls.
    const baseGap = m.kind === 'return' ? L.LABEL_GAP * 1.3 : L.LABEL_GAP
    const gap = Math.min(baseGap, length / 2 - 4)
    const labelX = 1 - (2 * gap) / length
    return { kind: m.kind, text: m.text ?? '', x1, x2, y, labelX }
  })

  return { messages, selfActivations, contentRight }
}
