/**
 * Emit mxGraph (.drawio) XML for a flow diagram from the deterministic layout.
 *
 * Responsibility: serialize the {@link import('../../lib/types.mjs').FlowLayoutModel} into
 *   uncompressed mxGraph XML using only native-text (render-safe) styles from styles.mjs and
 *   the shared serialization primitives in `lib/mxgraph.mjs`. It positions nothing itself — all
 *   coordinates come from the layout.
 * Inputs: (spec, opts?). Outputs: the full `<mxfile>` XML string.
 * Edit here when: you change how a model element maps to mxCell/mxGeometry XML. To change
 *   coordinates edit the layout; to change styling edit styles.mjs.
 * Do NOT — RENDER-SAFE RULE: never emit `html=1`/`whiteSpace=wrap`; the label text is pre-split
 *   by the layout and joined with hard breaks here.
 */
import { esc, multiline } from '../../lib/xml.mjs'
import { cell, edge, mxfile } from '../../lib/mxgraph.mjs'
import { S } from './styles.mjs'
import { layout } from './layout/index.mjs'

/**
 * @param {import('../../lib/types.mjs').FlowSpec} spec  parsed flow spec (envelope + body)
 * @param {{ specYaml?: string }} [opts]  raw YAML embedded as base64 `data-spec` for round-tripping
 * @returns {string} full <mxfile> XML
 */
export function emit(spec, opts = {}) {
  const model = layout(spec)
  let body = ''

  // Nodes: the shape draws the box/rhombus only; title (+ optional subtitle) are separate
  // native-text cells centered as a static group (mirrors sequence header text). A decision is a
  // bare diamond (no text cells) — its meaning lives on the branch labels.
  for (const nd of model.nodes) {
    const shape = nd.kind === 'decision' ? S.decision : S.box
    body += cell(`n_${nd.id}`, '', shape, nd.rect)
    if (nd.titleCell) {
      body += cell(`n_${nd.id}_title`, esc(nd.title), S.title, nd.titleCell)
    }
    if (nd.subtitleCell) {
      body += cell(`n_${nd.id}_sub`, esc(nd.subtitle), S.subtitle, nd.subtitleCell)
    }
  }

  // Edges: solid for a synchronous edge, dashed for an `async` one (a background/scheduled
  // relationship); explicit endpoints/waypoints; the label is a separate native-text cell placed
  // beside the arrow, its horizontal anchor appended to the style.
  model.edges.forEach((e, i) => {
    body += edge(`e_${i}`, '', e.async ? S.edgeAsync : S.edge, {
      x1: e.x1,
      y1: e.y1,
      x2: e.x2,
      y2: e.y2,
      waypoints: e.waypoints,
    })
    if (e.labelCell) {
      body += cell(`e_${i}_lbl`, multiline(e.labelLines), `${S.edgeLabel}align=${e.labelAlign};`, e.labelCell)
    }
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
