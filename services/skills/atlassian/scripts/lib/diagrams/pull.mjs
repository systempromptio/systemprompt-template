/**
 * Reverse (pull) delta for diagrams — the mirror of `diagrams/publish.mjs`.
 *
 * Diagrams are identified on a Confluence page by their durable signal: a
 * `<id>.drawio` attachment (optionally paired with a `<id>.png`). This module
 * is pure/string-only — the Confluence I/O (list + download) and the diagrams
 * `reverse.mjs` call live in the `pull-diagrams` command; here we only pick the
 * diagram attachments and splice reconstructed blocks into the doc markdown.
 */

import { escapeRegExp } from '../util/xhtml.mjs'

/**
 * From a page's attachment list, pick the `.drawio` diagrams and pair each with
 * its sibling `.png` (same basename), if present.
 *
 * @param {Array<{ title: string }>} attachments
 * @returns {Array<{ id: string, drawio: object, png: object|null }>}
 */
export function collectDrawioAttachments(attachments) {
  const byTitle = new Map((attachments || []).map((a) => [a.title, a]))
  const out = []
  for (const att of attachments || []) {
    if (typeof att.title !== 'string' || !att.title.endsWith('.drawio')) continue
    const id = att.title.slice(0, -'.drawio'.length)
    out.push({ id, drawio: att, png: byTitle.get(`${id}.png`) || null })
  }
  return out
}

/**
 * Insert or replace a diagram's authoring block (and its immediately-following
 * image) in the doc markdown, keyed by `id`. Resolution order:
 *   1. an existing block for the same id -> replaced in place (idempotent re-pull);
 *   2. else a bare `<id>.png` image with no block yet (the fresh-export case,
 *      where the page body only carried the flat image) -> replaced in place so
 *      the diagram lands where it belongs, not appended at the end;
 *   3. else the block + image are appended at the end.
 *
 * @param {string} md
 * @param {{ id: string, block: string, imageMarkdown: string }} entry
 * @returns {string}
 */
export function upsertDiagramBlock(md, { id, block, imageMarkdown }) {
  const idRe = escapeRegExp(id)
  const fence = '```drawio:[a-z0-9][a-z0-9-]*:' + idRe + '\\n[\\s\\S]*?\\n```'
  const trailingImage = '(?:\\s*!\\[[^\\]]*\\]\\([^)]*' + idRe + '\\.png[^)]*\\))?'
  const re = new RegExp(fence + trailingImage)
  const replacement = `${block}\n\n${imageMarkdown}`

  if (re.test(md)) return md.replace(re, replacement)

  // No block yet: a fresh export only has the flat `<id>.png` image. Require a
  // `/` before the id so `flow.png` never matches a sibling like `myflow.png`.
  const imageOnly = new RegExp('!\\[[^\\]]*\\]\\([^)]*/' + idRe + '\\.png[^)]*\\)')
  if (imageOnly.test(md)) return md.replace(imageOnly, replacement)

  const trimmed = md.replace(/\s*$/, '')
  const prefix = trimmed.length ? `${trimmed}\n\n` : ''
  return `${prefix}${replacement}\n`
}
