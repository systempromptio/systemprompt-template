/**
 * Typed reverse-pull core — one page's STORAGE → canonical authored markdown,
 * with its non-diagram body images downloaded alongside.
 *
 * This is the piece shared by the two callers of the typed pull:
 *   - `confluence.mjs get-page --type=<t>` (single-page baseline / re-sync), and
 *   - `export-confluence-to-markdown.mjs --type=<t>` (bulk export).
 * It wraps `doc/storage-to-doc.storageToDoc` (the type-free STORAGE→markdown
 * driver) and adds the one bit of I/O both callers need identically: fetch the
 * body's referenced attachment images into the page's assets dir (skipping the
 * generated-diagram PNGs, which `diagrams/reconstruct.mjs` handles under their
 * clean slug names).
 *
 * Diagram reconstruction is deliberately NOT done here: it is a shared step that
 * both callers already run over the FINAL markdown via `diagrams/reconstruct.mjs`
 * (the generic export path needs it too), so it stays one call up.
 *
 * Boundary: the actual Confluence I/O (`downloadAttachment`, `resolveAccountId`)
 *   is injected so this module stays unit-testable offline, mirroring
 *   `diagrams/reconstruct.mjs`.
 */

import { mkdir } from 'node:fs/promises'
import { join } from 'node:path'
import { withConcurrency, fileExists, sanitizeFileName } from '../util/node-io.mjs'
import { collectDrawioAttachments } from '../diagrams/pull.mjs'
import { storageToDoc } from './storage-to-doc.mjs'

const IMAGE_CONCURRENCY = 10

// Download the non-diagram images a typed body references (by attachment title)
// into `assetsDir`. Diagram PNGs are skipped (reconstruct.mjs fetches those under
// their clean slug); everything else (screenshots, logos) is fetched here so the
// local `![](<rel>/<file>)` links resolve.
async function downloadBodyImages({ pageId, filenames, attachments, assetsDir, skipTitles, downloadAttachment }) {
  const byTitle = new Map(attachments.map((a) => [a.title, a]))
  const wanted = [...new Set(filenames)].filter((f) => f && !skipTitles.has(f))
  if (!wanted.length) return
  await mkdir(assetsDir, { recursive: true })
  await withConcurrency(
    wanted.map((fileName) => async () => {
      const att = byTitle.get(fileName)
      if (!att) return
      const destPath = join(assetsDir, sanitizeFileName(fileName))
      if (await fileExists(destPath)) return
      try {
        await downloadAttachment(pageId, att, destPath)
      } catch (err) {
        process.stdout.write(
          `WARN: failed to download image ${fileName} for ${pageId}: ${err instanceof Error ? err.message : String(err)}\n`,
        )
      }
    }),
    IMAGE_CONCURRENCY,
  )
}

/**
 * Reverse one page's STORAGE into canonical markdown and download its body
 * images. The returned markdown still carries the `# Title` H1 that
 * `serializeDoc` emits and has NOT had diagrams reconstructed yet — the caller
 * runs `reconstructDiagramsInDoc` over the final markdown.
 *
 * @param {object} o
 * @param {'fsd'|'isd'|string} o.type
 * @param {string} o.pageId
 * @param {string} o.storageXhtml                 the page's `body.storage.value`
 * @param {string} [o.title]                      page title (storage has none)
 * @param {string} o.imageRelPrefix               markdown-relative image prefix (no trailing slash), e.g. `./assets`
 * @param {string} o.assetsDir                    absolute dir to download body images into
 * @param {Array<{ title: string }>} [o.attachments]  page attachment objects (v2 list results)
 * @param {(pageId: string, att: object, destPath: string) => Promise<any>} o.downloadAttachment
 * @param {(accountId: string) => Promise<string>|string} [o.resolveAccountId]
 * @returns {Promise<{ markdown: string, imageFilenames: string[], diagramPngTitles: Set<string> }>}
 * @throws {NotDocTypeError} propagated from `storageToDoc` for per-page fallback
 */
export async function pullTypedMarkdown({
  type,
  pageId,
  storageXhtml,
  title = '',
  imageRelPrefix,
  assetsDir,
  attachments = [],
  downloadAttachment,
  resolveAccountId = null,
}) {
  const { markdown, imageFilenames } = await storageToDoc({
    type,
    storageXhtml,
    title,
    imageRelPrefix,
    resolveAccountId,
  })

  const diagramPngTitles = new Set(
    collectDrawioAttachments(attachments)
      .filter((p) => p.png)
      .map((p) => p.png.title),
  )

  await downloadBodyImages({
    pageId,
    filenames: imageFilenames,
    attachments,
    assetsDir,
    skipTitles: diagramPngTitles,
    downloadAttachment,
  })

  return { markdown, imageFilenames, diagramPngTitles }
}
