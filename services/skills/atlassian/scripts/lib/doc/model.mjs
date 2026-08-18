/**
 * Generic, doc-type-AGNOSTIC document model — the markdown-native single source
 * of truth for any templated Confluence page.
 *
 * The canonical document is plain, human-readable Markdown that visually mirrors
 * the page. There is NO YAML front-matter and NO systemic metadata:
 *
 *   # <Page Title>                       ← h1 = page title
 *
 *   ## <Card Heading>                    ← header container; opens the combined
 *                                          chrome table (2-col label | value rows)
 *   ### <Group>                          ← H3 subsection → another merged section
 *                                          row + roster rows
 *   ## Reference Materials               ← separate 3-col table (Material|Link|Notes)
 *
 *   ## <body sections…>                  ← body: verbatim markdown from the first
 *                                          body-section H2 to EOF
 *
 * The header is ONE generic container: the card H2 opens it and each following
 * H3 becomes a grey merged section-header row in a single rendered table, with
 * the rows beneath them as the white cells. The renderer walks
 * `header.sections` without hard-coding field or group names.
 *
 * This module knows nothing about FSD vs ISD: which H2 opens the card and which
 * H2 begins the body are passed in as `bodySections` (the typed layer in
 * doc/types/ supplies them). It parses markdown into a structured model,
 * serializes a model back to markdown, and exposes a body outline.
 *
 * API:
 *   parseDoc(md, { bodySections }) → { title, header:{sections}, references, body }
 *   serializeDoc(model)            → string  (canonical markdown, round-trip stable)
 *   parseBody(body)                → { sections }   (H2/H3 outline, fence-aware)
 *   collectHeadings(md)            → [{ level, title, i }]  (fence-aware)
 *   stripMention(value)            → string  (drops a leading "@")
 */

// ─── Regexes ─────────────────────────────────────────────────────────────────

const HEADING_RE = /^(#{1,6})\s+(.*\S)\s*$/
const LINK_RE = /^\[([^\]]+)\]\(([^)]+)\)$/
const BARE_URL_RE = /^<?(https?:\/\/[^\s>]+)>?$/

// ─── Heading collection ────────────────────────────────────────────────────────

/** All headings (fence-aware), as { level, title, i }. */
export function collectHeadings(md) {
  const lines = String(md == null ? '' : md).split(/\r?\n/)
  const headings = []
  let inFence = false
  for (let i = 0; i < lines.length; i++) {
    const t = lines[i].trim()
    if (t.startsWith('```') || t.startsWith('~~~')) { inFence = !inFence; continue }
    if (inFence) continue
    const hm = HEADING_RE.exec(lines[i])
    if (hm) headings.push({ level: hm[1].length, title: hm[2].trim(), i })
  }
  return headings
}

// ─── Parse ───────────────────────────────────────────────────────────────────

/**
 * Parse a canonical markdown document into the generic structured model.
 *
 * `bodySections` are the H2 titles (case-insensitive) that begin the document
 * body — everything from the FIRST of them to EOF is body; everything above is
 * chrome. The typed layer (doc/types/) supplies this vocabulary.
 */
export function parseDoc(md, { bodySections = [] } = {}) {
  if (typeof md !== 'string') throw new TypeError('parseDoc expects a string')
  const lines = md.split(/\r?\n/)

  // Locate the h1 title and the first body-section heading (case-insensitive).
  let titleIndex = -1
  let bodyStart = lines.length
  let inFence = false
  const bodySet = new Set(bodySections.map((s) => s.toLowerCase()))

  for (let i = 0; i < lines.length; i++) {
    const t = lines[i].trim()
    if (t.startsWith('```') || t.startsWith('~~~')) { inFence = !inFence; continue }
    if (inFence) continue
    const hm = HEADING_RE.exec(lines[i])
    if (!hm) continue
    const level = hm[1].length
    const title = hm[2].trim()
    if (level === 1 && titleIndex === -1) titleIndex = i
    if (level === 2 && bodySet.has(title.toLowerCase())) { bodyStart = i; break }
  }

  // A divider authored directly above the first body section (through blank
  // lines) is the chrome→body separator; pull it into the body so it renders as
  // an <hr/> instead of being dropped with the chrome.
  let bodyFrom = bodyStart
  let k = bodyStart - 1
  while (k >= 0 && lines[k].trim() === '') k -= 1
  if (k >= 0 && /^-{3,}$/.test(lines[k].trim())) bodyFrom = k

  const title = titleIndex >= 0 ? HEADING_RE.exec(lines[titleIndex])[2].trim() : ''
  const chromeLines = lines.slice(titleIndex >= 0 ? titleIndex + 1 : 0, bodyFrom)
  const body = lines.slice(bodyFrom).join('\n').replace(/^\s+/, '').replace(/\s+$/, '')

  const { sections, references } = parseChrome(chromeLines)

  return { title, header: { sections }, references, body }
}

/**
 * Parse the chrome region (between the title and the body) into structured data.
 *
 * The first heading (the card H2) opens the header container and every following
 * heading starts another section — until "## Reference Materials", which is a
 * separate typed table (real Material/Link/Notes headers) and is NOT part of the
 * header.
 *
 * Returns generic `sections` (each { label, rows } — rows are arrays of raw cell
 * strings; sections[0] is the header card) plus typed `references`. The empty
 * placeholder header row that GFM requires (all-blank first row) is dropped.
 */
function parseChrome(lines) {
  const blocks = [] // { kind: 'heading'|'table', level?, title?, rows? }
  let inFence = false

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const t = line.trim()
    if (t.startsWith('```') || t.startsWith('~~~')) { inFence = !inFence; continue }
    if (inFence) continue

    const hm = HEADING_RE.exec(line)
    if (hm) {
      blocks.push({ kind: 'heading', level: hm[1].length, title: hm[2].trim() })
      continue
    }
    if (t.startsWith('|')) {
      const rows = []
      while (i < lines.length && lines[i].trim().startsWith('|')) {
        const cells = splitRow(lines[i])
        const isSep = cells.length && cells.every((c) => /^:?-+:?$/.test(c.trim()))
        if (!isSep) rows.push(cells)
        i++
      }
      i-- // step back; outer loop will advance
      blocks.push({ kind: 'table', rows })
    }
  }

  const sections = [] // generic header sections; sections[0] = the header card
  const references = []
  let target = null // 'header' | 'references'
  let current = null

  for (const b of blocks) {
    if (b.kind === 'heading') {
      if (/reference materials?/.test(b.title.toLowerCase())) {
        target = 'references'
        current = null
      } else {
        target = 'header'
        current = { label: b.title, rows: [] }
        sections.push(current)
      }
      continue
    }
    // table rows: drop the blank placeholder header row (all cells empty)
    const rows = b.rows.filter((r, idx) => !(idx === 0 && r.every((c) => c === '')))
    if (target === 'references') references.push(...parseReferenceRows(rows))
    else if (target === 'header' && current) current.rows.push(...rows)
  }

  return { sections, references }
}

// ─── Row parsers ───────────────────────────────────────────────────────────────

function splitRow(line) {
  return line.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map((c) => c.trim())
}

function parseReferenceRows(rows) {
  const out = []
  rows.forEach((cells, r) => {
    const [material = '', linkCell = '', notes = ''] = cells
    if (r === 0 && /material/.test(material.toLowerCase())) return // header
    if (!material) return
    out.push({ material: material.trim(), ...parseLinkCell(linkCell.trim()), notes: notes.trim() })
  })
  return out
}

// A reference "link" cell: labeled link → normal link; bare URL → inline card;
// "-" or plain text → text only.
function parseLinkCell(cell) {
  const lm = LINK_RE.exec(cell)
  if (lm) return { href: lm[2].trim(), text: lm[1].trim(), card: false }
  const um = BARE_URL_RE.exec(cell)
  if (um) return { href: um[1].trim(), text: '', card: true }
  return { href: '', text: cell, card: false }
}

/** Drop a leading "@" from a written name (people are plain names in the model). */
export function stripMention(v) {
  return String(v || '').replace(/^@/, '').trim()
}

// ─── Serialize ─────────────────────────────────────────────────────────────────

/**
 * Reassemble a model into canonical markdown (round-trip stable with parseDoc).
 *
 * The markdown carries only the heading hierarchy — no dividers, no layout. All
 * spacing/dividers are owned by the Confluence renderer, so any authored `---`
 * is dropped here:
 *   - the card H2 container is followed directly by its table.
 *   - every later header section is an "### " H3 followed directly by its table.
 *   - each chrome table opens with a blank placeholder header row + separator.
 *   - "## Reference Materials" is a separate chrome H2 with real column headers.
 */
export function serializeDoc(model) {
  const out = []
  out.push(`# ${model.title || ''}`, '')

  const sections = model.header?.sections || []
  sections.forEach((s, idx) => {
    if (idx === 0) out.push(`## ${s.label}`)
    else {
      if (idx >= 2) out.push('') // blank between H3s; first H3 has none
      out.push(`### ${s.label}`)
    }
    const cols = Math.max(2, ...s.rows.map((r) => r.length))
    out.push(emptyHeaderRow(cols), sepRow(cols))
    for (const r of s.rows) out.push(rowMd(r, cols))
  })

  if (model.references?.length) {
    out.push('') // separate chrome H2
    out.push('## Reference Materials')
    out.push('| Material | Link / reference | Notes |', '| --- | --- | --- |')
    for (const r of model.references) {
      out.push(`| ${r.material} | ${refLinkToMd(r)} | ${r.notes || ''} |`)
    }
  }

  // Drop any authored chrome→body divider: layout is renderer-owned, not markdown.
  const body = String(model.body || '')
    .replace(/^\s*(?:-{3,}\s*)?/, '')
    .replace(/\s+$/, '')
  if (body) out.push('', body)

  return out.join('\n').replace(/\s+$/, '') + '\n'
}

// A header section's table always opens with a blank placeholder header row (the
// column labels are redundant with the merged section title on the wiki).
const emptyHeaderRow = (cols) => `| ${Array(cols).fill('').join(' | ')} |`
const sepRow = (cols) => `| ${Array(cols).fill('---').join(' | ')} |`
const rowMd = (row, cols) => {
  const cells = row.slice()
  while (cells.length < cols) cells.push('')
  return `| ${cells.map((c) => c ?? '').join(' | ')} |`
}

function refLinkToMd(r) {
  if (r.text && r.href) return `[${r.text}](${r.href})`
  if (r.href) return r.href
  return r.text || '-'
}

// ─── Body outline ────────────────────────────────────────────────────────────

/**
 * Fence-aware H2/H3 outline of the body: [{ level, title, line }]. Requirement
 * code extraction is a TYPED concern and lives in doc/types/ (parseRequirements),
 * not here — this stays a plain structural outline usable by any doc type.
 */
export function parseBody(body) {
  const lines = String(body || '').split(/\r?\n/)
  const sections = []
  let inFence = false

  for (let n = 0; n < lines.length; n++) {
    const line = lines[n]
    const t = line.trim()
    if (t.startsWith('```') || t.startsWith('~~~')) { inFence = !inFence; continue }
    if (inFence) continue

    const hm = /^(#{2,6})\s+(.*\S)\s*$/.exec(line)
    if (!hm) continue
    sections.push({ level: hm[1].length, title: hm[2].trim(), line: n })
  }
  return { sections }
}
