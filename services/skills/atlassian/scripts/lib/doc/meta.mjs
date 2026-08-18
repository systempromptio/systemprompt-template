/**
 * doc-meta.mjs — script-owned stamping of the Confluence metadata header in an
 * FSD/ISD working-copy markdown file.
 *
 * The canonical header is two list lines placed right after the document H1:
 *
 *   # <WBS> - <Feature Name>
 *
 *   - Confluence: <URL>
 *   - Page ID: <ID>
 *
 * The SAME format is written on first publish (publish.mjs) and on a later pull
 * (confluence.mjs get-page --into <path>), so the working copy always records the
 * live page identity in one place. The functional-artifacts script reads
 * `- Confluence:` to build the stories' acceptance-criteria references + References
 * (a local FSD reference when the doc is unpublished).
 */

import { readFileSync, writeFileSync } from 'node:fs'

// Read the { url, pageId } currently stamped in a working-copy markdown string.
// Missing fields come back as ''. Only the header zone (before the first H2) is
// scanned so body content that happens to look like a meta line is ignored.
export function readDocMeta(md) {
  const lines = md.split(/\r?\n/)
  const firstH2 = lines.findIndex((l) => /^##\s+/.test(l))
  const zoneEnd = firstH2 === -1 ? lines.length : firstH2
  let url = ''
  let pageId = ''
  for (let i = 0; i < zoneEnd; i++) {
    const conf = /^-\s*Confluence:\s*(.*)$/i.exec(lines[i])
    if (conf && !url) url = conf[1].trim()
    const pid = /^-\s*Page ID:\s*(.*)$/i.exec(lines[i])
    if (pid && !pageId) pageId = pid[1].trim()
  }
  return { url, pageId }
}

// Insert or update the `- Confluence:` / `- Page ID:` header lines in a working
// copy at `mdPath`. Idempotent: existing header-zone meta lines are replaced in
// place (never duplicated), whether this is the first publish or a re-pull.
// Returns { path, changed }.
export function stampDocMeta(mdPath, { url, pageId } = {}) {
  const block = []
  if (url) block.push(`- Confluence: ${url}`)
  if (pageId != null && String(pageId) !== '') block.push(`- Page ID: ${pageId}`)
  if (!block.length) return { path: mdPath, changed: false }

  const raw = readFileSync(mdPath, 'utf8')
  const eol = raw.includes('\r\n') ? '\r\n' : '\n'
  let lines = raw.split(/\r?\n/)

  // Header zone = everything before the first H2 (or the whole file when there is
  // none, e.g. a pulled storage body without markdown headings).
  const firstH2 = lines.findIndex((l) => /^##\s+/.test(l))
  const zoneEnd = firstH2 === -1 ? lines.length : firstH2
  const isMeta = (l) => /^-\s*Confluence:/i.test(l) || /^-\s*Page ID:/i.test(l)

  // Drop any existing meta lines (and blank lines left immediately around them)
  // within the header zone.
  const kept = []
  for (let i = 0; i < lines.length; i++) {
    if (i < zoneEnd && isMeta(lines[i])) continue
    kept.push(lines[i])
  }
  lines = kept

  // Insert after the H1 (with one blank line on each side); prepend if no H1.
  const h1Idx = lines.findIndex((l) => /^#\s+/.test(l))
  if (h1Idx === -1) {
    lines.splice(0, 0, ...block, '')
  } else {
    // Normalize: strip blank lines right after the H1, then insert a clean block.
    let after = h1Idx + 1
    while (after < lines.length && lines[after].trim() === '') lines.splice(after, 1)
    lines.splice(after, 0, '', ...block, '')
  }

  let out = lines.join(eol).replace(new RegExp(`(?:${eol}){3,}`, 'g'), eol + eol)
  if (!out.endsWith(eol)) out += eol
  writeFileSync(mdPath, out, 'utf8')
  return { path: mdPath, changed: raw !== out }
}
