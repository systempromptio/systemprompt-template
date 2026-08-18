/**
 * Diagram-specific publish delta — layered on top of the agnostic hash-gated
 * `attachment-sync.mjs`.
 *
 * A published markdown image is treated as a generated diagram iff a sibling
 * `<slug>.drawio` sits next to the referenced `<slug>.png` on disk. For such an
 * image we additionally upload the `.drawio` companion (downloadable source;
 * this is what powers the Confluence -> markdown round-trip in
 * `diagrams/reconstruct.mjs`, used by both `pull-diagrams` and the export).
 *
 * DURABLE SIGNAL: the authoritative marker that an image is a generated diagram
 * is the sibling ATTACHMENT — a `<slug>.png` whose page also carries a
 * `<slug>.drawio` of the same basename. (An earlier design annotated the storage
 * body with a `<!-- diagram:<id> -->` anchor, but Confluence's sanitizer strips
 * HTML comments on save, so that signal does not persist and was dropped.) The
 * reverse round-trip (`collectDrawioAttachments`) keys off this sibling attachment.
 *
 * The `attachment-sync.mjs` layer stays fully agnostic; this module only decides
 * WHICH files are diagrams so their `.drawio` companions get uploaded.
 */
import { existsSync } from 'node:fs'
import { resolve, parse as parsePath, format as formatPath } from 'node:path'

/** Absolute path of the `.drawio` companion that would sit next to an image. */
export function companionFor(absImagePath) {
  const p = parsePath(absImagePath)
  return formatPath({ dir: p.dir, name: p.name, ext: '.drawio' })
}

/**
 * Resolve the referenced images against the markdown dir and collect the sibling
 * `.drawio` companions of the ones that are generated diagrams.
 *
 * @param {object} o
 * @param {string} o.mdDir         directory of the markdown file
 * @param {string[]} o.assetPaths  relative `![](...)` paths from the markdown
 * @returns {{ images: string[], companions: string[] }}
 */
export function collectDiagrams({ mdDir, assetPaths }) {
  const images = []
  const companions = []
  const seenCompanion = new Set()

  for (const rel of assetPaths) {
    const abs = resolve(mdDir, rel)
    images.push(abs)

    const drawio = companionFor(abs)
    if (existsSync(drawio) && !seenCompanion.has(drawio)) {
      seenCompanion.add(drawio)
      companions.push(drawio)
    }
  }

  return { images, companions }
}
