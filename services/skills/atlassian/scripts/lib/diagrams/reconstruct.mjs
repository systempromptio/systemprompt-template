/**
 * Shared diagram reconstruction (Confluence -> markdown).
 *
 * Both the `pull-diagrams` command and the full markdown export need the exact
 * same move: from a page's attachment list, find the generated diagrams (a
 * `<id>.drawio` with an optional sibling `<id>.png`), download both into
 * `assetsDir` under their CLEAN slug names, reconstruct the
 * ```drawio:<type>:<id> authoring block via the diagrams-side `reverse.mjs`
 * (it decodes the embedded `data-spec`), and splice the block + image into the
 * doc markdown — replacing an existing block or a bare exported image in place.
 *
 * The download + reverse invocation are injected so each caller owns its auth,
 * paths, and reverse-CLI location; the collection + splicing stays reusable and
 * unit-testable. Naming stays clean-slug on purpose: it keeps the `<id>.png`
 * paired with its `<id>.drawio` sibling and lets the hash-gated attachment sync
 * match the attachment title on re-publish (no duplicate attachment).
 */
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync } from 'node:fs'
import { join } from 'node:path'
import { collectDrawioAttachments, upsertDiagramBlock } from './pull.mjs'

/**
 * @param {object} o
 * @param {string} o.md                              current doc markdown
 * @param {string} o.pageId
 * @param {Array<{ title: string }>} o.attachments   page attachment objects (v2 list results)
 * @param {string} o.assetsDir                       absolute dir to write `<id>.drawio` / `<id>.png`
 * @param {string} o.imageRelPrefix                  markdown-relative image prefix (no trailing slash), e.g. `./assets` or `../../assets/<pageId>`
 * @param {(pageId: string, att: object, destPath: string) => Promise<any>} o.downloadAttachment
 * @param {string} o.reverseCli                      absolute path to the diagrams `reverse.mjs`
 * @param {boolean} [o.skipExistingPng]              skip re-downloading a `<id>.png` that already exists on disk
 * @returns {Promise<{ md: string, pulled: string[], diagramPngTitles: Set<string> }>}
 */
export async function reconstructDiagramsInDoc({
  md,
  pageId,
  attachments,
  assetsDir,
  imageRelPrefix,
  downloadAttachment,
  reverseCli,
  skipExistingPng = false,
}) {
  const pairs = collectDrawioAttachments(attachments)
  const pulled = []
  const diagramPngTitles = new Set()
  if (!pairs.length) return { md, pulled, diagramPngTitles }

  mkdirSync(assetsDir, { recursive: true })
  const rel = imageRelPrefix.replace(/\/+$/, '')
  let out = md

  for (const { id, drawio, png } of pairs) {
    const drawioPath = join(assetsDir, `${id}.drawio`)
    await downloadAttachment(pageId, drawio, drawioPath)

    if (png) {
      const pngPath = join(assetsDir, `${id}.png`)
      if (!(skipExistingPng && existsSync(pngPath))) {
        await downloadAttachment(pageId, png, pngPath)
      }
      diagramPngTitles.add(png.title)
    }

    const manifest = JSON.parse(
      execFileSync('node', [reverseCli, '--drawio', drawioPath], { encoding: 'utf8' }),
    )
    const imageMarkdown = `![${manifest.title}](${rel}/${manifest.png})`
    out = upsertDiagramBlock(out, { id, block: manifest.block, imageMarkdown })
    pulled.push(id)
  }

  return { md: out, pulled, diagramPngTitles }
}
