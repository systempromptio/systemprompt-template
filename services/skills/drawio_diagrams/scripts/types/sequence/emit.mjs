/**
 * Emit mxGraph (.drawio) XML for a sequence diagram from the deterministic layout.
 *
 * Responsibility: serialize the {@link import('../../lib/types.mjs').LayoutModel} into
 *   uncompressed mxGraph XML (required by the browserless renderer) using only native-text
 *   (render-safe) styles from styles.mjs. It positions nothing itself — all coordinates come
 *   from the layout.
 * Inputs: (spec, opts?). Outputs: the full `<mxfile>` XML string.
 * Edit here when: you change how a model element maps to mxCell/mxGeometry XML, or the file
 *   envelope. To change coordinates, edit the layout; to change styling, edit styles.mjs.
 * Do NOT — RENDER-SAFE RULE: never emit `html=1`/`whiteSpace=wrap` or <foreignObject>; resvg
 *   drops them. Multi-line text is pre-split by the layout and joined with hard breaks here.
 */
import { esc, multiline } from '../../lib/xml.mjs'
import { cell, edge, mxfile } from '../../lib/mxgraph.mjs'
import { S } from './styles.mjs'
import { layout } from './layout/index.mjs'

/**
 * @param {import('../../lib/types.mjs').Spec} spec  parsed sequence spec (envelope + body)
 * @param {{ specYaml?: string }} [opts]  raw YAML embedded as base64 `data-spec` for round-tripping
 * @returns {string} full <mxfile> XML
 */
export function emit(spec, opts = {}) {
  const model = layout(spec)
  let body = ''

  // Lifelines (headers/figures + dashed lines). The shape draws the box/figure only; the
  // title (+ optional subtitle) are separate native-text cells so the pair can be centered
  // as a static group with distinct fonts/colours.
  for (const p of model.participants) {
    const style = p.isActor ? S.actorLifeline(p.headerH) : S.lifeline(p.headerH)
    body += cell(`p_${p.id}`, '', style, {
      x: p.left,
      y: p.headerTop,
      w: p.width,
      h: p.lifelineBottom - p.headerTop,
    })
    body += cell(`p_${p.id}_title`, esc(p.title), p.isActor ? S.titleActor : S.title, p.titleCell)
    if (p.subtitleCell) {
      body += cell(`p_${p.id}_sub`, esc(p.subtitle), S.subtitle, p.subtitleCell)
    }
  }

  // Activation bars.
  model.activations.forEach((a, i) => {
    body += cell(`act_${i}`, '', S.activation, a)
  })

  // Messages.
  model.messages.forEach((m, i) => {
    if (m.kind === 'self') {
      // Hook arrow from the big bar, out and down, back into the nested rect (drawn as an
      // activation). Label is a separate native-text cell, offset off the hook.
      body += edge(`m_${i}`, '', S.self, {
        x1: m.startX,
        y1: m.hookY,
        x2: m.endX,
        y2: m.hookY + m.drop,
        waypoints: [
          { x: m.outX, y: m.hookY },
          { x: m.outX, y: m.hookY + m.drop },
        ],
      })
      body += cell(`m_${i}_lbl`, multiline(m.lines), S.selfLabel, {
        x: m.labelX,
        y: m.labelY,
        w: m.labelW,
        h: m.labelH,
      })
    } else {
      // Label sits toward the arrowhead (layout computed labelX for a constant gap).
      // Anchor the text on the arrowhead side so the gap is independent of text width:
      // right edge for rightward arrows, left edge for leftward ones.
      const rightward = m.x2 >= m.x1
      const anchor = rightward ? 'align=right;' : 'align=left;'
      const isReturn = m.kind === 'return'
      const base = isReturn ? S.return : S.call
      // Return labels already sit clear (bigger gap); call labels ride close to the arrow,
      // so lift them a couple px up and away from the arrowhead.
      const labelOffset = isReturn ? null : { x: rightward ? -3 : 3, y: -3 }
      body += edge(`m_${i}`, esc(m.text), base + anchor, {
        x1: m.x1,
        y1: m.y,
        x2: m.x2,
        y2: m.y,
        labelX: m.labelX,
        labelOffset,
      })
    }
  })

  // Notes.
  model.notes.forEach((note, i) => {
    body += cell(`note_${i}`, multiline(note.lines), S.note, note)
  })

  return mxfile({
    id: spec.id ?? 'diagram',
    name: spec.title ?? spec.id ?? 'diagram',
    specYaml: opts.specYaml,
    width: model.width,
    height: model.height,
    body,
  })
}
