/**
 * Confluence HTML (export_view) → Markdown pipeline, shared by the bulk exporter
 * and the export validator. Also home to `makeStorageTurndown`, the STORAGE-body
 * converter the type-aware reverse pull (`storageToDoc`) uses. The Turndown
 * configuration used to be copy-pasted in both scripts; this is the one copy.
 *
 * Responsibility: pure HTML/markdown transforms + export-header parsing. No
 *   Confluence API calls and no filesystem access — callers pass in `baseUrl`
 *   where a transform needs it (so this module stays unit-testable offline).
 * Edit here when: the export markdown conventions (turndown opts, header shape,
 *   URL absolutization, non-content stripping) change.
 */

import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const TurndownService = require('turndown')
const { highlightedCodeBlock, strikethrough, taskListItems, tables } = require('turndown-plugin-gfm')

/** A Turndown service configured for the export markdown conventions. */
export function makeTurndown() {
  const turndown = new TurndownService({
    headingStyle: 'atx',
    hr: '---',
    bulletListMarker: '-',
    codeBlockStyle: 'fenced',
    emDelimiter: '_',
  })
  turndown.use([highlightedCodeBlock, strikethrough, taskListItems])
  // Tables are kept as raw HTML (GFM tables lose colspans/rich cells).
  turndown.keep(['table', 'thead', 'tbody', 'tfoot', 'tr', 'th', 'td', 'colgroup', 'col'])
  return turndown
}

// ── Storage-format body converter (the inverse of md-to-storage) ────────────────
//
// The typed reverse pull converts a page's STORAGE body (not export_view) back to
// canonical markdown. Storage carries classic `<ac:...>`/`<ri:...>` macros rather
// than rendered HTML, and the authored body tables are plain `<table>` (no
// `confluenceTable` chrome), so unlike `makeTurndown` this variant CONVERTS tables
// to GFM pipe tables and adds rules that invert md-to-storage's macros:
//   `<ac:structured-macro ac:name="code">`  -> fenced code block
//   `<ac:structured-macro ac:name="status">`-> `word` (backtick badge)
//   any other `<ac:structured-macro>`        -> dropped (toc/expand/change-history)
//   `<ac:link><ri:user account-id>`          -> resolved display name (via mentionNames)
//   `<ac:link ac:anchor="slug">`             -> `[text](#slug)` in-page link
// `<ac:image>` is handled BEFORE turndown (see storage-to-doc): it carries no text
// so turndown's blankRule would drop it, and its `<ri:attachment>` child is not a
// void element — so the parser rewrites it to a real `<img>` first.

const acParam = (node, name) => {
  for (const p of Array.from(node.getElementsByTagName('ac:parameter'))) {
    if (p.getAttribute('ac:name') === name) return p.textContent || ''
  }
  return ''
}

const acPlainTextBody = (node) => {
  const b = node.getElementsByTagName('ac:plain-text-body')[0]
  return b ? b.textContent || '' : ''
}

const acPlainTextLinkBody = (node) => {
  const b = node.getElementsByTagName('ac:plain-text-link-body')[0]
  return b ? b.textContent || '' : ''
}

/**
 * Turndown configured for Confluence STORAGE-format body conversion.
 *
 * @param {object} [o]
 * @param {Record<string,string>} [o.mentionNames]  accountId -> display name for `<ri:user>`
 */
// A header cell is bold in storage (we emit it, the native editor emits it), but
// the canonical markdown writes header text plain — unwrap so a round trip is a no-op.
function unboldHeader(content, node) {
  if (node.nodeName.toLowerCase() !== 'th') return content
  const m = /^\s*\*\*([\s\S]+)\*\*\s*$/.exec(content)
  return m && !m[1].includes('**') ? m[1] : content
}

export function makeStorageTurndown({ mentionNames = {} } = {}) {
  const turndown = new TurndownService({
    headingStyle: 'atx',
    hr: '---',
    bulletListMarker: '-',
    codeBlockStyle: 'fenced',
    emDelimiter: '_',
  })
  turndown.use([tables, strikethrough, taskListItems])

  // Confluence wraps every table-cell's content in a block `<p>` (and the native
  // editor adds more), which Turndown renders as `\n\n…\n\n` INSIDE the cell —
  // shattering the one-row-per-line GFM table into ragged multi-line pipes. Flatten
  // each cell to a single inline line (block breaks → space, runs collapsed, pipes
  // escaped). Added after the tables plugin so it overrides its `tableCell` rule;
  // the plugin's heading-separator row builds its own cells, so it is untouched.
  turndown.addRule('storageTableCell', {
    filter: ['th', 'td'],
    replacement: (content, node) => {
      const index = Array.prototype.indexOf.call(node.parentNode.childNodes, node)
      const prefix = index === 0 ? '| ' : ' '
      const inline = unboldHeader(content, node)
        .replace(/\r?\n+/g, ' ')
        .replace(/[ \t]+/g, ' ')
        .trim()
        // Turndown escapes markdown punctuation that means nothing in a cell: a
        // bracket opens no link without a following `(`, a leading hyphen opens no
        // list. Left in, the backslashes publish verbatim on the next round trip.
        .replace(/\\([[\]])/g, '$1')
        .replace(/^\\-/, '-')
        .replace(/\|/g, '\\|')
      return `${prefix}${inline} |`
    },
  })

  // Turndown's default break is `  \n`, which the cell flattener above collapses
  // back into plain spaces. Keep the authored `<br>` verbatim so multi-line cells
  // survive the round trip.
  turndown.addRule('storageLineBreak', {
    filter: 'br',
    replacement: () => '<br>',
  })

  // Compact list items: Turndown pads the marker to 4 chars (`-   `, `1.  `) to
  // align mixed lists; authored markdown uses a single space (`- `, `1. `). Match
  // the authored convention so the canonical body round-trips cleanly.
  turndown.addRule('compactListItem', {
    filter: 'li',
    replacement: (content, node, options) => {
      // Confluence wraps each <li>'s content in a block <p>, so `content` arrives as
      // `\n\ntext\n\n`. Strip ALL surrounding newlines (not just down to one — a
      // dangling `\n` gets indented to a `  ` line, littering the list with trailing-
      // whitespace blank lines), indent genuine continuations, then guarantee no line
      // keeps trailing whitespace.
      const body = content
        .replace(/^\n+/, '')
        .replace(/\n+$/, '')
        .replace(/\n/gm, '\n  ')
        .replace(/[ \t]+$/gm, '')
      let prefix = `${options.bulletListMarker} `
      const parent = node.parentNode
      if (parent && parent.nodeName === 'OL') {
        const start = parent.getAttribute('start')
        const index = Array.prototype.indexOf.call(parent.children, node)
        prefix = `${start ? Number(start) + index : index + 1}. `
      }
      return prefix + body + (node.nextSibling ? '\n' : '')
    },
  })

  // One rule for every `<ac:structured-macro>` (code → fence, status → backtick,
  // everything else dropped). A single rule avoids Turndown's addRule precedence
  // (later-added rules win), which would otherwise let a generic "drop" rule mask
  // the specific code/status handling.
  turndown.addRule('acStructuredMacro', {
    filter: (node) => node.nodeName.toLowerCase() === 'ac:structured-macro',
    replacement: (_c, node) => {
      const name = node.getAttribute('ac:name')
      if (name === 'code') {
        const lang = acParam(node, 'language')
        const body = acPlainTextBody(node).replace(/\n+$/, '')
        return `\n\n\`\`\`${lang}\n${body}\n\`\`\`\n\n`
      }
      if (name === 'status') return `\`${acParam(node, 'title')}\``
      return ''
    },
  })

  // <ac:link><ri:user account-id>: resolve to the display name; an ac:anchor link
  // becomes an in-page markdown link (the anchor macro that marks its target
  // heading is re-emitted on publish, so it is dropped here); a plain <ac:link>
  // (page link) keeps its inner text.
  turndown.addRule('acLink', {
    filter: (node) => node.nodeName.toLowerCase() === 'ac:link',
    replacement: (content, node) => {
      const user = node.getElementsByTagName('ri:user')[0]
      if (user) {
        const id = user.getAttribute('ri:account-id') || ''
        return mentionNames[id] || ''
      }
      const anchor = node.getAttribute('ac:anchor')
      if (anchor) return `[${(acPlainTextLinkBody(node) || content).trim()}](#${anchor})`
      return content
    },
  })

  return turndown
}

/** Drop <head>/<style>/<script> blocks that carry no page content. */
export function stripNonContentHtml(html) {
  return html
    .replace(/<head\b[^>]*>[\s\S]*?<\/head>/gi, '')
    .replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, '')
    .replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, '')
}

/** Rewrite root-relative href/src (`/wiki/...`) to absolute using `baseUrl`. */
export function absolutizeRootRelativeUrls(html, baseUrl) {
  return html
    .replaceAll('href="/', `href="${baseUrl}/`)
    .replaceAll("href='/", `href='${baseUrl}/`)
    .replaceAll('src="/', `src="${baseUrl}/`)
    .replaceAll("src='/", `src='${baseUrl}/`)
}

/**
 * Parse the export markdown header block (produced by the bulk exporter):
 *
 *   # <Title>
 *   - Confluence: <url>
 *   - Page ID: <id>
 *   - Version: <n>
 *   - Updated: <iso>
 *
 * Returns { pageId, version, title, body } — `body` is everything after the
 * header block (trimmed). Callers that only need the metadata ignore `body`.
 */
export function parseExportHeader(markdown, { maxLines = 45 } = {}) {
  const lines = markdown.split('\n').slice(0, maxLines)
  let pageId = null
  let version = null
  let title = null
  let metadataEndIndex = 0

  const first = lines[0] || ''
  if (first.startsWith('# ')) title = first.slice(2).trim()

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const idMatch = line.match(/^- Page ID:\s*(\d+)\s*$/)
    if (idMatch) pageId = idMatch[1]
    const versionMatch = line.match(/^- Version:\s*(\d+)\s*$/)
    if (versionMatch) version = parseInt(versionMatch[1], 10)
    if (/^- Updated:/.test(line)) metadataEndIndex = i + 1
  }

  const rel = lines.slice(metadataEndIndex).findIndex((l) => l.trim() !== '')
  const bodyStartIndex = metadataEndIndex + (rel >= 0 ? rel : 0)
  const body = lines.slice(bodyStartIndex).join('\n').trim()

  return { pageId, version, title, body }
}
