/**
 * Forward renderer: canonical document model → Confluence storage XHTML.
 *
 * renderDoc(model, opts?) → string
 *
 * The chrome (the header card, Approvals, Reference Materials, optional TOC, status
 * lozenges, @mentions) is produced by the Nunjucks templates in
 * templates/confluence/, rendered linearly to mirror the canonical markdown. The
 * requirement body is converted to storage by the existing mdToStorage() and
 * injected into the template's `body` block. Output is classic <ac:...> storage —
 * the only representation the publish transport sends.
 *
 * People are plain names in the model; @mentions are resolved via the
 * `mentionMap` option ({ 'Full Name': 'accountId' }). Without a match the name is
 * rendered as plain text (e.g. dry render before the publisher looks ids up).
 *
 * The Nunjucks environment runs with autoescape OFF; templates escape every
 * dynamic value with the `x` (text) / `xa` (attribute) filters.
 */

import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'
import { mdToStorage, collectAnchorTargets, makeInlineRenderer } from './md-to-storage.mjs'
import { getDocType } from './types/index.mjs'
import { escHtml, escAttr } from '../util/xhtml.mjs'
import { statusColour, isStatus, deriveClientName } from './status-vocab.mjs'

const require = createRequire(import.meta.url)
const nunjucks = require('nunjucks')

const TEMPLATES_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..', 'templates', 'confluence')

// Escaping (util/xhtml.mjs) and the status word/colour vocabulary (doc/status-vocab.mjs)
// are shared with doc/md-to-storage and doc/profiles so the header renderer, the
// body converter, and the badge maps can never drift apart.

// ─── Defaults ──────────────────────────────────────────────────────────────────

// Equal thirds: the header table is a Content Properties source, so its columns
// are key | value | note — none of them is the narrow "role" column it once was.
const DEFAULTS = {
  includeToc: false,
  tocMin: 2,
  tocMax: 3,
  headerBg: '#f4f5f7',
  cardWidths: [109, 109, 109],
}

// Build the Nunjucks environment once per mentionMap identity; a fresh env per
// render keeps the md global scoped to that document's resolved names.
//
// `md` renders an authored cell's inline markdown (links, `code`, **bold**,
// @mentions) exactly as the body converter does, and escapes its own text — so a
// cell passed through it must NOT also go through the `x` filter.
function buildEnv(mentionMap, badgeMap, clientName) {
  const env = new nunjucks.Environment(new nunjucks.FileSystemLoader(TEMPLATES_DIR, { noCache: true }), {
    autoescape: false,
    trimBlocks: false,
    lstripBlocks: false,
    throwOnUndefined: false,
  })
  env.addFilter('x', escHtml)
  env.addFilter('xa', escAttr)
  // Bound to this document's client so "<Client> review" lozenges and colours
  // without the client's name ever being written into the library.
  env.addGlobal('statusColor', (text) => statusColour(text, clientName))
  env.addGlobal('isStatus', (text) => isStatus(text, clientName))
  env.addGlobal('md', makeInlineRenderer(mentionMap, badgeMap))
  return env
}

// ─── Body section layout ────────────────────────────────────────────────────
//
// Section spacing is NOT taken from the authored markdown: we strip every blank
// line and `---` divider from the body, then split it into H2/H3 parts and let
// the template apply the deterministic layout rules (divider before each H2,
// gap before each H3). Fenced code blocks are preserved verbatim.

function stripBodyLayout(body) {
  const out = []
  let inFence = false
  for (const line of String(body || '').split(/\r?\n/)) {
    const t = line.trim()
    if (t.startsWith('```') || t.startsWith('~~~')) { inFence = !inFence; out.push(line); continue }
    if (inFence) { out.push(line); continue }
    if (t === '' || /^-{3,}$/.test(t)) continue
    out.push(line)
  }
  return out.join('\n')
}

// Split the (stripped) body into ordered parts, each starting at an H2/H3 heading
// and running until the next one. `level` is 2 or 3 (0 for any pre-heading lead).
// Each part's markdown is converted to storage independently so the template can
// join them with its own dividers/gaps.
function splitBodyParts(body, convert) {
  const parts = []
  let cur = null
  let inFence = false
  const flush = () => { if (cur) { parts.push({ level: cur.level, storage: convert(cur.md.join('\n')) }) } }
  for (const line of String(body || '').split(/\r?\n/)) {
    const t = line.trim()
    if (t.startsWith('```') || t.startsWith('~~~')) inFence = !inFence
    const hm = inFence ? null : /^(#{2,3})\s+/.exec(t)
    if (hm) { flush(); cur = { level: hm[1].length, md: [line] } }
    else { if (!cur) cur = { level: 0, md: [] }; cur.md.push(line) }
  }
  flush()
  return parts.filter((p) => p.storage && p.storage.trim())
}

// ─── Render ──────────────────────────────────────────────────────────────────

/**
 * Render a canonical model to Confluence storage XHTML.
 *
 * opts:
 *   type        – 'fsd' | 'isd' (default 'fsd') → picks the template
 *   mentionMap  – { 'Full Name': 'accountId' } for @mention resolution
 *   badgeMap    – { 'status text': 'Colour' } for backtick badges inside the body
 *   validate    – throw on validation errors (default true)
 *   includeToc, tocMin, tocMax, headerBg – chrome overrides
 */
export function renderDoc(model, opts = {}) {
  const type = (opts.type || 'fsd').toLowerCase()
  const docType = getDocType(type)

  if (opts.validate !== false) {
    const v = docType.validate(model)
    if (!v.ok) throw new Error(`Document is invalid:\n  - ${v.errors.join('\n  - ')}`)
  }

  // Convert one body part's markdown to storage. Body layout (blank lines and
  // `---` dividers) is stripped beforehand (stripBodyLayout) and re-applied by
  // the template's fixed rules, so this conversion deliberately emits NO
  // thematic breaks and NO blank paragraphs (thematicBreak/blankParagraphs off).
  // Anchor targets are collected from the WHOLE body first: the conversion runs one
  // heading-section at a time, so a link and the heading it points at almost always
  // land in different parts.
  const anchorTargets = collectAnchorTargets(model.body || '')
  const convert = (md) =>
    mdToStorage(md, {
      mentionMap: opts.mentionMap || {},
      badgeMap: opts.badgeMap || {},
      thematicBreak: false,
      blankParagraphs: false,
      dropFirstH1: false,
      anchorTargets,
    }).body

  const bodyParts = splitBodyParts(stripBodyLayout(model.body || ''), convert)

  const clientName = deriveClientName(model)
  const env = buildEnv(opts.mentionMap || {}, opts.badgeMap || {}, clientName)

  const context = {
    ...DEFAULTS,
    ...pickOverrides(opts),
    model,
    bodyParts,
    propertiesId: docType.propertiesId,
    clientName,
    jiraLinksJql: jiraLinksJql(opts),
    jiraCloudId: opts.jiraCloudId || '',
  }

  return env.render(`${type}.njk`, context).trim() + '\n'
}

// JQL for the "Linked Jira Tickets" macro: the Stories that carry a Confluence
// remote link to THIS page (globalId `appId=<...>&pageId=<...>`). Rendered only
// when both the page id and the Confluence app id are known; otherwise the
// section is omitted (e.g. a brand-new page before its id exists, or when
// CONFLUENCE_APP_ID is not configured).
function jiraLinksJql(opts) {
  const pageId = String(opts.pageId || '').trim()
  const appId = String(opts.jiraAppId || '').trim()
  if (!pageId || !appId) return ''
  return `issue in issuesWithRemoteLinksByGlobalId("appId=${appId}&pageId=${pageId}")`
}

function pickOverrides(opts) {
  const out = {}
  for (const k of ['includeToc', 'tocMin', 'tocMax', 'headerBg', 'cardWidths']) {
    if (opts[k] !== undefined) out[k] = opts[k]
  }
  return out
}
