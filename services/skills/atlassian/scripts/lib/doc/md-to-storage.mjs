/**
 * Pure markdown → Confluence storage XHTML converter.
 * No API calls, no doc-type knowledge, no side effects.
 *
 * export function mdToStorage(md, opts?) → { body: string, assets: string[] }
 *
 * opts:
 *   mentionMap   – { 'Full Name': 'accountId', … }  renders @mentions
 *   badgeMap     – { 'draft': 'Grey', … }  maps backtick-wrapped text to Confluence status macros
 *   dropFirstH1  – boolean (default true)  drop the leading h1 (it is the page title)
 *   thematicBreak – boolean (default false) emit <hr/> for `---` lines instead of dropping them
 *   blankParagraphs – boolean (default false) materialize authored vertical gaps: a
 *     run of K blank lines between two blocks emits (K-1) empty <p/> paragraphs
 *     (a single blank line is normal block separation and adds no gap).
 *   anchorTargets – Set of heading slugs that get an anchor macro, for callers that
 *     convert a document one part at a time (a link and the heading it targets then
 *     sit in different calls). Defaults to the targets found in `md`.
 */

// Escaping is centralized in xhtml.mjs (the ONE storage-escaping choke point):
// esc/escAttr only escape markup characters, so text reaches the page verbatim.
import { escHtml as esc, escAttr, escapeRegExp as escapeRe } from '../util/xhtml.mjs'

// Body images render as a capped thumbnail (~12 text lines tall), scaled
// proportionally — clicking opens the full-size preview in Confluence. Keeps a
// full-width diagram from dominating the page.
const IMAGE_MAX_HEIGHT_PX = 240

// Ordinal-column table sizing. Confluence Cloud only honours a <colgroup> when
// EVERY <col> carries an explicit width — a single bare <col/> makes it ignore
// the whole group and fall back to equal columns (the px values are treated as
// PROPORTIONS, not absolute widths). So we pin the ordinal (row-number) first
// column narrow and give every content column an equal, larger share.
const ORDINAL_COL_WIDTH_PX = 22
const TOKEN_COL_WIDTH_PX = 48
const CONTENT_COL_WIDTH_PX = 240

function statusMacro(title, colour) {
  return (
    `<ac:structured-macro ac:name="status">` +
    `<ac:parameter ac:name="colour">${colour}</ac:parameter>` +
    `<ac:parameter ac:name="title">${esc(title)}</ac:parameter>` +
    `</ac:structured-macro>`
  )
}

function mentionMacro(accountId) {
  return `<ac:link><ri:user ri:account-id="${escAttr(accountId)}"/></ac:link>`
}

function codeMacro(code, language) {
  const lang = language ? `<ac:parameter ac:name="language">${escAttr(language)}</ac:parameter>` : ''
  return (
    `<ac:structured-macro ac:name="code">` +
    lang +
    `<ac:plain-text-body><![CDATA[${code}]]></ac:plain-text-body>` +
    `</ac:structured-macro>`
  )
}

// In-page anchors. Confluence derives a heading's own anchor id from its rendered
// text, which is unstable for our headings (inline code, brackets, punctuation), so
// a referenced heading carries an explicit anchor macro instead and links target
// that fixed slug rather than the heading text.
function anchorMacro(name) {
  return (
    `<ac:structured-macro ac:name="anchor">` +
    `<ac:parameter ac:name="">${esc(name)}</ac:parameter>` +
    `</ac:structured-macro>`
  )
}

function anchorLink(anchor, text) {
  return (
    `<ac:link ac:anchor="${escAttr(anchor)}">` +
    `<ac:plain-text-link-body><![CDATA[${text}]]></ac:plain-text-link-body>` +
    `</ac:link>`
  )
}

/** GitHub-style heading slug: the `#target` an author writes in a markdown link. */
export function headingSlug(text) {
  return String(text == null ? '' : text)
    .replace(/`/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

/** Slugs referenced by an in-page markdown link (`[text](#slug)`) anywhere in `md`. */
export function collectAnchorTargets(md) {
  const targets = new Set()
  for (const m of String(md).matchAll(/\[[^\]]+\]\(#([^)]*)\)/g)) {
    const slug = headingSlug(m[1])
    if (slug) targets.add(slug)
  }
  return targets
}

/**
 * Build the inline renderer once per mdToStorage call (captures mentionMap/badgeMap by closure).
 *
 * Exported because the chrome templates render authored cells (the header card,
 * Reference Materials) through the SAME renderer as the body — one implementation
 * of "inline markdown → storage", so the two can never drift apart.
 */
export function makeInlineRenderer(mentionMap, badgeMap) {
  const mentionNames = Object.keys(mentionMap)
  const mentionRe = mentionNames.length
    ? new RegExp('(' + mentionNames.map(escapeRe).join('|') + ')')
    : null

  // Token regex: image, link, bold, inline-code, italic, line break
  const tokenRe =
    /(!\[[^\]]*\]\([^)]*\))|(\[[^\]]+\]\([^)]*\))|(\*\*[^*]+\*\*)|(`[^`]+`)|(_[^_]+_)|(<br\s*\/?>)/g

  function renderTokens(text) {
    let out = ''
    let last = 0
    let m
    tokenRe.lastIndex = 0
    while ((m = tokenRe.exec(text)) !== null) {
      out += esc(text.slice(last, m.index))
      const tok = m[0]
      if (m[1]) {
        // image  ![alt](path)
        const im = /^!\[([^\]]*)\]\(([^)]*)\)$/.exec(tok)
        out += `<ac:image ac:alt="${escAttr(im[1])}" ac:height="${IMAGE_MAX_HEIGHT_PX}"><ri:attachment ri:filename="${escAttr(im[2].split('/').pop())}"/></ac:image>`
      } else if (m[2]) {
        // link  [text](url) — `#slug` is an in-page anchor, anything else a plain href
        const lm = /^\[([^\]]+)\]\(([^)]*)\)$/.exec(tok)
        out += lm[2].startsWith('#')
          ? anchorLink(headingSlug(lm[2].slice(1)), lm[1])
          : `<a href="${escAttr(lm[2])}">${esc(lm[1])}</a>`
      } else if (m[3]) {
        // **bold**
        out += `<strong>${esc(tok.slice(2, -2))}</strong>`
      } else if (m[4]) {
        // `code` — may render as a status badge
        const inner = tok.slice(1, -1)
        const key = inner.toLowerCase()
        if (badgeMap[key]) {
          out += statusMacro(inner, badgeMap[key])
        } else {
          out += `<code>${esc(inner)}</code>`
        }
      } else if (m[5]) {
        // _italic_
        out += `<em>${esc(tok.slice(1, -1))}</em>`
      } else if (m[6]) {
        // <br> — the only inline HTML we honour, for multi-line table cells
        out += '<br />'
      }
      last = tokenRe.lastIndex
    }
    return out + esc(text.slice(last))
  }

  function inline(text) {
    if (!mentionRe) return renderTokens(text)
    return text
      .split(mentionRe)
      .map((part) => (mentionMap[part] ? mentionMacro(mentionMap[part]) : renderTokens(part)))
      .join('')
  }

  return inline
}

// ─── Block parser ─────────────────────────────────────────────────────────────

const LIST_RE = /^(\s*)([-*]|\d+\.)\s+(.*)$/

function renderListItems(items, start) {
  const base = items[start].indent
  const ordered = items[start].ordered
  let html = ordered ? '<ol>' : '<ul>'
  let k = start
  while (k < items.length && items[k].indent === base) {
    let li = items[k].text // already inline-rendered by caller
    const next = k + 1
    if (next < items.length && items[next].indent > base) {
      const child = renderListItems(items, next)
      li += child.html
      k = child.next
    } else {
      k = next
    }
    html += `<li>${li}</li>`
  }
  return { html: html + (ordered ? '</ol>' : '</ul>'), next: k }
}

const SEPARATOR_CELL_RE = /^:?-+:?$/

/**
 * Cells of one pipe-table row. GFM writes a literal pipe inside a cell as `\|`, so
 * splitting on every pipe would break the cell into phantom columns and leave the
 * backslash stranded in the output (the reverse converter already emits `\|`).
 */
function splitRow(line) {
  return line
    .trim()
    .split(/(?<!\\)\|/)
    .slice(1, -1)
    .map((c) => c.trim().replace(/\\\|/g, '|'))
}

// Confluence renders a bare <th> in the page font; its native editor bolds header
// text itself, so we emit <strong> too and a hand-edited table stays consistent.
function boldHeader(html) {
  if (!html.trim() || /^<strong>[\s\S]*<\/strong>$/.test(html)) return html
  return `<strong>${html}</strong>`
}

function renderTable(rows, inline) {
  const ncols = Math.max(...rows.map((r) => r.length))
  const firstCell = (r) => String((r && r[0]) || '').trim()
  const header0 = firstCell(rows[0])
  const dataRows = rows.slice(1)
  // Ordinal index column: a "#"/"Q#"/"No." style header, or every data row's
  // first cell being a bare integer (a plain row number).
  const ordinal =
    /#$/.test(header0) ||
    /^(no\.?|№|item|idx|index)$/i.test(header0) ||
    (dataRows.length > 0 && dataRows.every((r) => /^\d+$/.test(firstCell(r))))

  let table = '<table>'
  if (ordinal) {
    // A prefixed ordinal ("Q#") holds a token like `Q1`, not a bare digit.
    const width = header0 === '#' || !/#$/.test(header0) ? ORDINAL_COL_WIDTH_PX : TOKEN_COL_WIDTH_PX
    table += `<colgroup><col style="width: ${width}.0px;"/>`
    for (let c = 1; c < ncols; c += 1) table += `<col style="width: ${CONTENT_COL_WIDTH_PX}.0px;"/>`
    table += '</colgroup>'
  }
  table += '<tbody>'
  rows.forEach((cells, r) => {
    const tag = r === 0 ? 'th' : 'td'
    // A divider/subheader row inside an ordinal table (only the first cell
    // has content) spans the full width instead of squeezing into the pinned
    // ordinal column.
    if (ordinal && r > 0 && ncols > 1 && firstCell(cells) && cells.slice(1).every((c) => !String(c || '').trim())) {
      table += `<tr><td colspan="${ncols}">${inline(cells[0])}</td></tr>`
      return
    }
    table += '<tr>' + cells.map((c) => `<${tag}>${r === 0 ? boldHeader(inline(c)) : inline(c)}</${tag}>`).join('') + '</tr>'
  })
  return table + '</tbody></table>'
}

export function mdToStorage(
  md,
  {
    mentionMap = {},
    badgeMap = {},
    dropFirstH1 = true,
    thematicBreak = false,
    blankParagraphs = false,
    anchorTargets = null,
  } = {},
) {
  const inline = makeInlineRenderer(mentionMap, badgeMap)
  const targets = anchorTargets || collectAnchorTargets(md)
  const assets = []
  const out = []

  const lines = md.split(/\r?\n/)
  let i = 0
  let droppedTitle = false

  // Authored vertical spacing. A run of K blank lines between two emitted blocks
  // materializes (K-1) empty paragraphs when blankParagraphs is on; leading and
  // trailing runs add nothing. `emit` flushes the pending gap before each block.
  let pendingBlanks = 0
  let emittedAny = false
  const emit = (html) => {
    if (blankParagraphs && emittedAny && pendingBlanks > 1) {
      for (let k = 0; k < pendingBlanks - 1; k += 1) out.push('<p />')
    }
    pendingBlanks = 0
    out.push(html)
    emittedAny = true
  }

  while (i < lines.length) {
    const line = lines[i]
    const t = line.trim()

    // blank
    if (!t) { pendingBlanks += 1; i += 1; continue }

    // HTML comment — dropped (the canonical doc has no front-matter; any authored
    // <!-- ... --> is an editorial note, not page content).
    if (t.startsWith('<!--')) {
      // consume until -->
      while (i < lines.length && !lines[i].includes('-->')) i += 1
      i += 1
      continue
    }

    // thematic break
    if (t === '---') {
      if (thematicBreak) emit('<hr/>')
      i += 1
      continue
    }

    // fenced code block  ``` [lang]
    if (t.startsWith('```')) {
      const lang = t.slice(3).trim()
      // Diagram spec fences (```drawio:<type>:<id>, ```drawio, ```diagram) are the
      // authoring source-of-truth, not page content — strip them so they never
      // reach the wiki. The generated ![](./assets/<id>.png) after the block still
      // becomes an ac:image, and the sibling .drawio publishes as a companion.
      const isSpecFence = lang === 'diagram' || lang === 'drawio' || lang.startsWith('drawio:')
      const codeLines = []
      i += 1
      while (i < lines.length && !lines[i].trim().startsWith('```')) {
        codeLines.push(lines[i])
        i += 1
      }
      i += 1 // consume closing ```
      if (!isSpecFence) emit(codeMacro(codeLines.join('\n'), lang))
      continue
    }

    // heading
    const hm = /^(#{1,6})\s+(.*)$/.exec(t)
    if (hm) {
      const level = hm[1].length
      if (level === 1 && dropFirstH1 && !droppedTitle) {
        droppedTitle = true
        i += 1
        continue
      }
      // The anchor macro sits INSIDE the heading: as a sibling block it would
      // render as an empty paragraph above it.
      const slug = headingSlug(hm[2])
      const anchor = targets.has(slug) ? anchorMacro(slug) : ''
      emit(`<h${level}>${anchor}${inline(hm[2])}</h${level}>`)
      i += 1
      continue
    }

    // pipe table — a run of pipe lines can hold SEVERAL tables, because the body
    // layout stripper removes the blank lines that separated them. A separator row
    // that is not the run's second line therefore marks a new table whose header is
    // the row just before it.
    if (t.startsWith('|')) {
      const tables = []
      let rows = []
      while (i < lines.length && lines[i].trim().startsWith('|')) {
        const cells = splitRow(lines[i])
        const isSeparator = cells.length && cells.every((c) => SEPARATOR_CELL_RE.test(c))
        if (!isSeparator) {
          rows.push(cells)
        } else if (rows.length > 1) {
          tables.push(rows.slice(0, -1))
          rows = rows.slice(-1)
        }
        i += 1
      }
      if (rows.length) tables.push(rows)
      for (const rowsOfTable of tables) emit(renderTable(rowsOfTable, inline))
      continue
    }

    // list (ordered or unordered)
    if (LIST_RE.test(line)) {
      const items = []
      while (i < lines.length && LIST_RE.test(lines[i])) {
        const mm = LIST_RE.exec(lines[i])
        const raw = mm[3]
        // track inline image assets referenced from list items (before escaping)
        const liImgRe = /!\[[^\]]*\]\(([^)]+)\)/g
        let im
        while ((im = liImgRe.exec(raw)) !== null) assets.push(im[1])
        items.push({ indent: mm[1].length, ordered: /\d/.test(mm[2]), text: inline(raw) })
        i += 1
      }
      emit(renderListItems(items, 0).html)
      continue
    }

    // paragraph — track inline image assets
    const imgRe = /!\[[^\]]*\]\(([^)]+)\)/g
    let m
    while ((m = imgRe.exec(t)) !== null) assets.push(m[1])

    emit(`<p>${inline(t)}</p>`)
    i += 1
  }

  return { body: out.join('\n'), assets }
}

/**
 * Collect asset paths from raw markdown (before converting), for image upload.
 * Returns relative paths as they appear in md src attributes.
 */
export function collectAssets(md) {
  const assets = []
  const re = /!\[[^\]]*\]\(([^)]+)\)/g
  let m
  while ((m = re.exec(md)) !== null) assets.push(m[1])
  return [...new Set(assets)]
}
