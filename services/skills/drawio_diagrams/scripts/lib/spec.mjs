/**
 * Shared, type-agnostic spec loading + validation used by generate.mjs and validate.mjs.
 *
 * Responsibility: parse YAML, extract diagram blocks from markdown, reconstruct a spec/block
 *   from a generated `.drawio` (the reverse pull), and validate the common envelope (`type`,
 *   `id`) before delegating to the registered type's own validator. This is the type-agnostic
 *   seam: new types plug in via the registry and need no change here.
 * Inputs/Outputs: strings/objects in, parsed specs / error strings / slugs out.
 * Edit here when: you change the common envelope or the markdown block syntax.
 * Do NOT: add type-specific rules here — those belong in `types/<type>/validate`.
 */
import YAML from 'yaml'
import { getType, listTypes } from '../types/index.mjs'

export function parseYaml(text) {
  return YAML.parse(text)
}

/**
 * Extract ```drawio:<type>:<id> fenced blocks from a markdown document.
 *
 * The info string is authoritative for the diagram `type` and its unique `id`;
 * the block body is the scenario only (the type-specific fields, e.g. a
 * sequence's `participants`/`messages`/`notes`). `type`/`id` are NOT repeated in
 * the body — the caller injects them from the header.
 *
 * @param {string} markdown
 * @returns {Array<{ type: string, id: string, body: string }>}
 */
export function extractDrawioBlocks(markdown) {
  const blocks = []
  const re = /```drawio:([a-z0-9][a-z0-9-]*):([^\n`]+)\n([\s\S]*?)\n```/g
  let m
  while ((m = re.exec(markdown))) {
    blocks.push({ type: m[1].trim(), id: m[2].trim(), body: m[3] })
  }
  return blocks
}

/**
 * Reverse of the `data-spec` embedding done by `emit`: decode the full spec
 * that a generated `.drawio` carries as base64 YAML on its `<diagram>` element.
 *
 * The `.drawio` is self-describing (`data-spec` + `data-spec-format`), so the
 * reverse pull reconstructs the authoring block without parsing draw.io
 * geometry. Only the versioned `drawio-diagrams/v1` format is understood.
 *
 * @param {string} drawioXml  contents of a generated `.drawio` file
 * @returns {import('./types.mjs').Spec}
 */
export function readEmbeddedSpec(drawioXml) {
  const fmt = /data-spec-format="([^"]+)"/.exec(drawioXml)?.[1]
  if (fmt !== 'drawio-diagrams/v1') {
    throw new Error(`unsupported or missing data-spec-format: ${fmt || 'none'}`)
  }
  const b64 = /data-spec="([^"]+)"/.exec(drawioXml)?.[1]
  if (!b64) throw new Error('no data-spec found on .drawio')
  return YAML.parse(Buffer.from(b64, 'base64').toString('utf8'))
}

/**
 * Serialize a body to compact authoring YAML: leaf mappings (all-scalar, i.e. a
 * single `participants`/`messages`/`notes` item) render inline as `{ k: v, … }`
 * on one line, while the outer structure (keys whose values are sequences) stays
 * block. `lineWidth: 0` disables wrapping so a long inline item never folds, and
 * the serializer auto-quotes any scalar that needs it (e.g. a `text` with a
 * comma) — so generated output is always safe regardless of value content.
 *
 * @param {Record<string, unknown>} body
 * @returns {string}
 */
function bodyToCompactYaml(body) {
  const doc = new YAML.Document(body)
  YAML.visit(doc, {
    Map(_, node) {
      if (node.items.every((it) => YAML.isScalar(it.value))) node.flow = true
    },
  })
  return doc.toString({ lineWidth: 0 }).trimEnd()
}

/**
 * Serialize a spec back into a ```drawio:<type>:<id> authoring block. The header
 * carries `type`/`id` (authoritative), the body is the scenario only — the exact
 * inverse of `extractDrawioBlocks` + the generator's header injection. The body
 * is emitted in the compact one-item-per-line flow style (see `bodyToCompactYaml`).
 *
 * @param {import('./types.mjs').Spec} spec
 * @returns {string} the fenced block (no trailing newline)
 */
export function specToBlock(spec) {
  const { type, id, ...body } = spec
  const yaml = bodyToCompactYaml(body)
  return '```drawio:' + type + ':' + id + '\n' + yaml + '\n```'
}

/**
 * Validate the common envelope + delegate to the type-specific validator.
 * @param {import('./types.mjs').Spec} spec
 * @returns {string[]} error strings ([] = valid)
 */
export function validateSpec(spec) {
  if (!spec || typeof spec !== 'object' || Array.isArray(spec)) {
    return ['spec: must be a YAML mapping']
  }
  const errors = []
  if (!spec.type) {
    errors.push(`type: required (one of: ${listTypes().join(', ')})`)
  } else if (!getType(spec.type)) {
    errors.push(`type: unknown "${spec.type}" (known: ${listTypes().join(', ')})`)
  }
  if (!spec.id) errors.push('id: required (a stable slug used for filenames)')

  const t = spec.type && getType(spec.type)
  if (t) errors.push(...t.validate(spec))
  return errors
}

/**
 * Collect a spec's soft advisories (non-blocking): text-budget overflows and the like. These do
 * NOT gate file generation — they inform the author. Returns [] for an unknown/typeless spec.
 * @param {import('./types.mjs').Spec} spec
 * @returns {string[]} warning strings ([] = none)
 */
export function warnSpec(spec) {
  if (!spec || typeof spec !== 'object' || Array.isArray(spec)) return []
  const t = spec.type && getType(spec.type)
  return t && typeof t.warnings === 'function' ? t.warnings(spec) : []
}

/** Turn "Some Title" / "consent-manager" into a filesystem-safe slug. */
export function slugify(value) {
  return String(value ?? '')
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 80) || 'diagram'
}
