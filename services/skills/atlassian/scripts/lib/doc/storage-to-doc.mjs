/**
 * Typed reverse pull — the inverse of the publish pipeline: Confluence page
 * STORAGE (XHTML with `<ac:…>`/`<ri:…>` macros) back to canonical authored
 * markdown. It reads the few special elements structurally (the header card +
 * approval groups and the Reference Materials table) into the generic model, then
 * takes the body SUBTRACTIVELY — everything between the header-card table and the
 * script-owned footer, minus the wiki-only chrome (the Content Properties
 * (`details`) wrapper around the card table, `<ac:layout>` wrappers, TOC macro,
 * status-badge legend, the injected `<hr/>`/`<p/>` dividers). Anchoring on
 * those stable elements (not the template's `<hr/>`/attribute-free `<h2>`, which
 * the native Confluence editor strips/rewrites) means hand-edited pages still
 * reverse-pull their full body. The result serializes via the already-built
 * `serializeDoc` — so it is authored-style markdown, not a render dump.
 *
 * Boundary: this module is the ONLY reverse engine. `parseStorageToModel` is the
 *   generic, doc-type-agnostic parser (driven by a `cardHeading` from the typed
 *   layer); `storageToDoc` is the async CLI-facing driver that resolves the type,
 *   optionally resolves `<ri:user>` account-ids to display names, and serializes.
 * Edit here when: the reverse of a publish transform changes. Type-SPECIFIC
 *   vocabulary stays in doc/types/; generic serialization in doc/model.mjs.
 */

import { createRequire } from 'node:module'
import { serializeDoc } from './model.mjs'
import { getDocType } from './types/index.mjs'
import { makeStorageTurndown } from '../atlassian/html-to-markdown.mjs'
import { escHtml } from '../util/xhtml.mjs'

const require = createRequire(import.meta.url)
const domino = require('@mixmark-io/domino')

/** Thrown when a page's storage does not match the requested doc type (missing
 * card heading), so the export CLI can fall back to the generic dump. */
export class NotDocTypeError extends Error {
  constructor(message) {
    super(message)
    this.name = 'NotDocTypeError'
  }
}

const norm = (s) => String(s == null ? '' : s).replace(/\s+/g, ' ').trim()

const acParam = (node, name) => {
  for (const p of Array.from(node.getElementsByTagName('ac:parameter'))) {
    if (p.getAttribute('ac:name') === name) return p.textContent || ''
  }
  return ''
}

// A chrome cell (header card / roster, and the Reference Materials Material and
// Notes columns) back to the inline markdown it was authored as, flattened to a
// single line — the inverse of the templates' md() global, and the reason both
// directions stay symmetric: a cell rendered as a link or `code` pulls back as
// one. Mentions are handled by the converter's own <ri:user> rule.
//
// A status badge is the one exception: the card carries it as a bare word, where
// the body converter would emit a backtick badge.
//
// Markdown escaping is switched OFF. These cells are short single-line values
// whose plain text (`utag_data`, `a_b`) has to survive verbatim, and the forward
// inline renderer does not interpret backslash escapes — emitting them would put
// literal backslashes on the page.
const makeCellReader = (mentionNames) => {
  const toMarkdown = makeStorageTurndown({ mentionNames })
  toMarkdown.escape = (s) => s
  return (td) => {
    if (!td) return ''
    const status = Array.from(td.getElementsByTagName('ac:structured-macro')).find(
      (m) => m.getAttribute('ac:name') === 'status',
    )
    if (status) return acParam(status, 'title').trim()

    // A mention is <ac:link><ri:user/></ac:link> — no text of its own, so Turndown
    // classifies it as a blank node and drops it before any rule runs. Resolve it
    // to its display name up front (on a clone, so the parsed document is left
    // intact) and let the converter see plain text.
    const cell = td.cloneNode(true)
    for (const link of Array.from(cell.getElementsByTagName('ac:link'))) {
      const user = link.getElementsByTagName('ri:user')[0]
      if (!user) continue
      const name = mentionNames[user.getAttribute('ri:account-id') || ''] || ''
      link.parentNode.replaceChild(cell.ownerDocument.createTextNode(name), link)
    }
    return norm(toMarkdown.turndown(cell.innerHTML))
  }
}

// Invert model.mjs parseLinkCell for a storage reference "Link" cell:
//   <a href>text</a>                        → labeled link
//   <a href data-card-appearance>url</a>     → inline card (bare url)
//   plain text                               → text only
const refLinkCell = (td) => {
  const a = td ? td.getElementsByTagName('a')[0] : null
  if (a) {
    const href = a.getAttribute('href') || ''
    if (a.hasAttribute('data-card-appearance')) return { href, text: '', card: true }
    return { href, text: norm(a.textContent), card: false }
  }
  return { href: '', text: norm(td && td.textContent), card: false }
}

// ── Body boundaries (raw-string, chrome-anchored) ──────────────────────────────

// Where the body BEGINS: right after the header-card table (the type's detection
// anchor, always present). Found by the card-heading text, then the first
// </table> that closes the card table (it carries no nested tables), plus the
// Content Properties (`details`) wrapper's closing tags when present — the forward
// renderer wraps the card table in that macro, and leaving its close behind would
// lead the body with stray macro markup. Falls back to the legacy first-`<hr/>`
// anchor only if the heading text is not in the raw string (it always is once the
// DOM found the card, so this is defensive).
function bodyStartIndex(xhtml, cardHeading) {
  const at = xhtml.toLowerCase().indexOf(String(cardHeading || '').toLowerCase())
  if (at >= 0) {
    const close = xhtml.slice(at).match(/<\/table>(\s*<\/ac:rich-text-body>\s*<\/ac:structured-macro>)?/i)
    if (close) return at + close.index + close[0].length
  }
  const hr = xhtml.match(/<hr\s*\/>/i)
  return hr ? hr.index + hr[0].length : 0
}

// Where the body ENDS: the first script-owned footer heading (Document Change Log
// or Linked Jira Tickets), attribute-tolerant. undefined (slice to EOF) when
// neither is present.
function footerStartIndex(xhtml) {
  let earliest = -1
  for (const re of [
    /<h2[^>]*>\s*document change log\s*<\/h2>/i,
    /<h2[^>]*>\s*linked jira tickets\s*<\/h2>/i,
  ]) {
    const at = xhtml.search(re)
    if (at >= 0 && (earliest < 0 || at < earliest)) earliest = at
  }
  return earliest < 0 ? undefined : earliest
}

/**
 * Generic (doc-type-agnostic) storage → model parser.
 *
 * @param {string} storageXhtml               a page's `body.storage.value`
 * @param {object} o
 * @param {string} o.cardHeading              this type's header-card heading (the detection anchor)
 * @param {Record<string,string>} [o.mentionNames]  accountId → display name for `<ri:user>`
 * @param {string} [o.imageRelPrefix]         relative dir for body `<ac:image>` (no trailing slash)
 * @returns {{ model: { header: object, references: object[], body: string }, imageFilenames: string[] }}
 * @throws {NotDocTypeError} when no header table with `cardHeading` is present
 */
export function parseStorageToModel(storageXhtml, { cardHeading, mentionNames = {}, imageRelPrefix = './assets' } = {}) {
  const xhtml = String(storageXhtml == null ? '' : storageXhtml)
  const doc = domino.createWindow(xhtml).document
  const cellValue = makeCellReader(mentionNames)

  // ── Header: the combined card + approval-group table (identified by its card
  // heading). Its absence is the strongest signal this page is not this type.
  const tables = Array.from(doc.getElementsByTagName('table'))
  const wanted = norm(cardHeading)
  const headerTable = tables.find((t) =>
    Array.from(t.getElementsByTagName('th')).some((th) => norm(th.textContent) === wanted),
  )
  if (!headerTable) {
    throw new NotDocTypeError(`storage does not contain the "${cardHeading}" header card`)
  }

  const sections = []
  let current = null
  for (const tr of Array.from(headerTable.getElementsByTagName('tr'))) {
    const th = tr.getElementsByTagName('th')[0]
    if (th) {
      current = { label: norm(th.textContent), rows: [] }
      sections.push(current)
      continue
    }
    if (!current) continue
    const tds = Array.from(tr.getElementsByTagName('td'))
    if (tds.length) current.rows.push(tds.map((td) => cellValue(td)))
  }

  // ── Reference Materials: the 3-col table following that H2.
  const references = []
  const refH2 = Array.from(doc.getElementsByTagName('h2')).find((h) =>
    /reference materials?/i.test(norm(h.textContent)),
  )
  if (refH2) {
    let el = refH2.nextElementSibling
    while (el && el.nodeName.toLowerCase() !== 'table') el = el.nextElementSibling
    if (el) {
      for (const tr of Array.from(el.getElementsByTagName('tr'))) {
        if (tr.getElementsByTagName('th')[0]) continue // column-header row
        const tds = Array.from(tr.getElementsByTagName('td'))
        const material = cellValue(tds[0])
        if (!material) continue
        references.push({ material, ...refLinkCell(tds[1]), notes: cellValue(tds[2]) })
      }
    }
  }

  // ── Body: everything BETWEEN the chrome (the header-card table + the Reference
  // Materials table) and the script-owned footer — a SUBTRACTIVE cut. Only a
  // handful of elements are special (parsed above); the rest is ordinary markdown
  // the converter handles generically, so we anchor on those STABLE elements and
  // the footer heading text, never on the template's `<hr/>` divider or an
  // attribute-free `<h2>`. The native Confluence editor removes `<hr/>` and stamps
  // `local-id` on every heading, so the old positional slice returned an empty
  // body for any hand-edited page; anchoring on content survives that.
  //
  // Runs on the raw storage string (not the domino DOM) so a code macro's CDATA
  // survives — the chrome above was already read structurally from the DOM.
  let bodyHtml = xhtml.slice(bodyStartIndex(xhtml, cardHeading), footerStartIndex(xhtml))

  bodyHtml = bodyHtml
    // Drop the Reference Materials block when it falls inside the slice (chrome,
    // already captured): its H2 through the first following </table>.
    .replace(/<h2[^>]*>\s*reference materials?\s*<\/h2>[\s\S]*?<\/table>/i, '')
    // Unwrap Confluence layout wrappers (presentational only) so the body is linear.
    .replace(/<\/?ac:layout(?:-section|-cell)?\b[^>]*>/gi, '')
    // Drop leftover chrome that can lead the body when references are absent: the
    // status-badge legend paragraph and a stray TOC macro.
    .replace(/<p>(?:(?!<\/p>)[\s\S])*?Status badge values[\s\S]*?<\/p>/gi, '')
    .replace(/<ac:structured-macro\b[^>]*ac:name="toc"[\s\S]*?<\/ac:structured-macro>/gi, '')
    // Drop the column widths we pin on ordinal tables: they carry no authored
    // content, and turndown's table rule only accepts a heading row when <tbody>
    // leads the table — a preceding <colgroup> makes it emit raw HTML instead.
    .replace(/<colgroup>[\s\S]*?<\/colgroup>/gi, '')
    // Remove the template's structural dividers (authored bodies never emit these).
    .replace(/<hr\s*\/>/gi, '\n')
    .replace(/<p\s*\/>/gi, '\n')
    .replace(/<p>\s*<\/p>/gi, '\n')

  const imageFilenames = [...bodyHtml.matchAll(/ri:filename="([^"]+)"/g)].map((m) => m[1])

  // A code macro's body is a CDATA section, which the HTML parser turndown uses
  // turns into a bogus comment (CDATA is XML-only) — so its text is unreachable
  // via the DOM. Rewrite the whole macro to `<pre><code class="language-…">` (the
  // CDATA extracted and HTML-escaped) so turndown's fenced-code rule emits it.
  bodyHtml = bodyHtml.replace(
    /<ac:structured-macro\b[^>]*ac:name="code"[\s\S]*?<\/ac:structured-macro>/gi,
    (macro) => {
      const lang = /<ac:parameter ac:name="language">([^<]*)<\/ac:parameter>/.exec(macro)
      const cdata = /<!\[CDATA\[([\s\S]*?)\]\]>/.exec(macro)
      const cls = lang && lang[1] ? ` class="language-${lang[1]}"` : ''
      return `<pre><code${cls}>${escHtml(cdata ? cdata[1] : '')}</code></pre>`
    },
  )

  // An anchor link's `<ac:plain-text-link-body>` hits the same CDATA blind spot:
  // its text is unreachable, so the whole `<ac:link>` reads as blank and turndown
  // drops the link. Unwrap the body to escaped text for the acLink rule to pick up.
  bodyHtml = bodyHtml.replace(
    /<ac:plain-text-link-body>\s*<!\[CDATA\[([\s\S]*?)\]\]>\s*<\/ac:plain-text-link-body>/gi,
    (_macro, text) => escHtml(text),
  )

  // `<ac:image>` carries no text and its `<ri:attachment>` child is not a void
  // element, so Turndown's blankRule would drop it. Rewrite to a real <img> (a
  // recognized void element) up front, mapping the attachment to its local path.
  const rel = String(imageRelPrefix).replace(/\/+$/, '')
  bodyHtml = bodyHtml.replace(
    /<ac:image\b([^>]*)>\s*<ri:attachment\s+ri:filename="([^"]+)"[^>]*\/?>\s*<\/ac:image>/gi,
    (_m, attrs, file) => {
      const alt = /ac:alt="([^"]*)"/.exec(attrs)
      return `<img alt="${alt ? alt[1] : ''}" src="${rel}/${file}"/>`
    },
  )

  const body = makeStorageTurndown({ mentionNames }).turndown(bodyHtml).trim()

  return { model: { header: { sections }, references, body }, imageFilenames }
}

/**
 * CLI-facing driver: storage → canonical markdown for a given doc type.
 *
 * Keeps the reverse type-free the way `publishDoc` keeps the forward path:
 * resolve the type via the registry, optionally resolve `<ri:user>` account-ids
 * to display names (the publish path turned names INTO ids), then serialize.
 *
 * @param {object} o
 * @param {'fsd'|'isd'|string} o.type
 * @param {string} o.storageXhtml
 * @param {string} [o.title]                       page title (storage has none)
 * @param {string} [o.imageRelPrefix]              relative dir for body images
 * @param {(accountId: string) => Promise<string>|string} [o.resolveAccountId]
 * @returns {Promise<{ markdown: string, imageFilenames: string[] }>}
 * @throws {NotDocTypeError} propagated from the parser for per-page fallback
 */
export async function storageToDoc({ type, storageXhtml, title = '', imageRelPrefix = './assets', resolveAccountId = null } = {}) {
  const dt = getDocType(type)

  const mentionNames = {}
  if (typeof resolveAccountId === 'function') {
    const ids = new Set([...String(storageXhtml || '').matchAll(/ri:account-id="([^"]+)"/g)].map((m) => m[1]))
    for (const id of ids) {
      try {
        mentionNames[id] = await resolveAccountId(id)
      } catch {
        mentionNames[id] = ''
      }
    }
  }

  const { model, imageFilenames } = dt.parseStorage(storageXhtml, { mentionNames, imageRelPrefix })
  model.title = title || model.title || ''
  return { markdown: serializeDoc(model), imageFilenames }
}
