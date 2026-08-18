/**
 * Shared mxGraph (.drawio) serialization primitives used by every diagram type's emitter.
 *
 * Responsibility: the ONE place that knows the concrete `<mxCell>` / `<mxGeometry>` / edge
 *   markup and the `<mxfile>` envelope (including the self-describing base64 `data-spec`), so
 *   each type's `emit` only maps its layout model to these calls and never hand-writes XML.
 * Inputs/Outputs: ids/styles/geometry in, XML fragments/strings out.
 * Edit here when: you change how a cell/edge is serialized or the file envelope. A change here
 *   affects EVERY type — the emit-golden snapshots guard against accidental drift.
 * Do NOT: build style strings or compute coordinates here — styles live in each type's
 *   styles.mjs and coordinates come from its layout. RENDER-SAFE stays the emitter's concern
 *   (never pass html=1 / whiteSpace=wrap styles through here).
 */
import { esc, n } from './xml.mjs'

/**
 * A vertex `<mxCell>` with absolute geometry.
 * @param {string} id
 * @param {string} value  already-escaped label (or '')
 * @param {string} style
 * @param {{x:number,y:number,w:number,h:number}} geo
 * @returns {string}
 */
export function cell(id, value, style, geo) {
  return (
    `        <mxCell id="${esc(id)}" value="${value}" style="${style}" vertex="1" parent="1">\n` +
    `          <mxGeometry x="${n(geo.x)}" y="${n(geo.y)}" width="${n(geo.w)}" height="${n(geo.h)}" as="geometry"/>\n` +
    `        </mxCell>\n`
  )
}

/**
 * An edge `<mxCell>` with absolute endpoints, optional explicit waypoints and a relative label
 * position. All geometry is absolute/explicit — we never rely on the renderer's edge routing.
 * @param {string} id
 * @param {string} value  already-escaped label (or '')
 * @param {string} style
 * @param {{x1:number,y1:number,x2:number,y2:number,waypoints?:{x:number,y:number}[],labelX?:number,labelOffset?:{x:number,y:number}}} points
 * @returns {string}
 */
export function edge(id, value, style, points) {
  const { x1, y1, x2, y2, waypoints, labelX, labelOffset } = points
  let inner =
    `          <mxPoint x="${n(x1)}" y="${n(y1)}" as="sourcePoint"/>\n` +
    `          <mxPoint x="${n(x2)}" y="${n(y2)}" as="targetPoint"/>\n`
  if (waypoints && waypoints.length) {
    inner +=
      `          <Array as="points">\n` +
      waypoints.map((p) => `            <mxPoint x="${n(p.x)}" y="${n(p.y)}"/>\n`).join('') +
      `          </Array>\n`
  }
  // An absolute pixel nudge of the label (screen coords: -x left, -y up), on top of the
  // relative position. Used to lift labels a hair off the arrow.
  if (labelX != null && labelOffset) {
    inner += `          <mxPoint x="${n(labelOffset.x)}" y="${n(labelOffset.y)}" as="offset"/>\n`
  }
  // labelX (relative -1..1 along the edge: -1 source, 0 center, +1 target) pins the
  // label toward the arrowhead. y=0 keeps it on the line; the style handles anchoring.
  const geoAttrs = labelX != null ? ` x="${n(labelX)}" y="0"` : ''
  return (
    `        <mxCell id="${esc(id)}" value="${value}" style="${style}" edge="1" parent="1">\n` +
    `          <mxGeometry${geoAttrs} relative="1" as="geometry">\n${inner}          </mxGeometry>\n` +
    `        </mxCell>\n`
  )
}

/**
 * Wrap a body of `<mxCell>`s in the full uncompressed `<mxfile>` envelope, embedding the raw
 * spec YAML as base64 `data-spec` so the file is self-describing for the reverse pull.
 * @param {{ id?: string, name?: string, specYaml?: string, width: number, height: number, body: string }} args
 * @returns {string} the full `<mxfile>` XML
 */
export function mxfile({ id, name, specYaml, width, height, body }) {
  const dataSpec = specYaml
    ? ` data-spec="${Buffer.from(specYaml, 'utf8').toString('base64')}" data-spec-format="drawio-diagrams/v1"`
    : ''
  const diagramName = esc(name ?? id ?? 'diagram')
  const diagramId = esc(id ?? 'diagram')
  return (
    `<mxfile>\n` +
    `  <diagram id="${diagramId}" name="${diagramName}"${dataSpec}>\n` +
    `    <mxGraphModel dx="${n(width)}" dy="${n(height)}" grid="0" gridSize="10" ` +
    `guides="1" tooltips="1" connect="1" arrows="1" fold="1" page="1" pageScale="1" ` +
    `pageWidth="${n(width)}" pageHeight="${n(height)}" math="0" shadow="0">\n` +
    `      <root>\n` +
    `        <mxCell id="0"/>\n` +
    `        <mxCell id="1" parent="0"/>\n` +
    body +
    `      </root>\n` +
    `    </mxGraphModel>\n` +
    `  </diagram>\n` +
    `</mxfile>\n`
  )
}
