/**
 * Agnostic, hash-gated attachment sync (PUSH: local files -> Confluence).
 *
 * The gate is content-type agnostic: for ANY file (a PNG, an SVG, a `.drawio`
 * source, a PDF, ...) we store the sha256 of the local bytes in the attachment
 * `comment` as `sha256:<hex>`. On the next publish we compare the local hash to
 * the stored one and skip the upload when they match, so unchanged files keep
 * the exact same attachment (and link) across runs. Only changed bytes cause a
 * new attachment version.
 *
 * Public entry point: `syncAttachmentsToPage({ pageId, files })` — a batch push
 * of absolute file paths. This layer stays deliberately diagram-agnostic: the
 * "which files are generated diagrams (image + `.drawio` companion)" decision
 * lives in diagrams/publish.mjs, which feeds the resolved file list in here.
 */
import { createHash } from 'node:crypto'
import { readFileSync, existsSync } from 'node:fs'
import { basename } from 'node:path'
import {
  listAttachments,
  uploadAttachment,
  updateAttachment,
  deleteAttachment,
  readStoredComment,
} from '../atlassian/attachments.mjs'

export const HASH_PREFIX = 'sha256:'

export function sha256File(absPath) {
  return createHash('sha256').update(readFileSync(absPath)).digest('hex')
}

export function formatComment(hex) {
  return `${HASH_PREFIX}${hex}`
}

/** Extract the `sha256:<hex>` hash from an attachment comment, or null. */
export function parseHash(comment) {
  if (typeof comment !== 'string') return null
  const m = comment.match(/sha256:([0-9a-f]{64})/i)
  return m ? m[1].toLowerCase() : null
}

/**
 * Decide what to do with one file given the gate inputs. Pure and unit-tested.
 * - no attachment yet            -> 'upload'
 * - attachment exists, same hash -> 'skip'
 * - attachment exists, differs   -> 'update'   (also when hash is unreadable)
 */
export function decideAction({ exists, localHash, storedHash }) {
  if (!exists) return 'upload'
  if (storedHash && storedHash === localHash) return 'skip'
  return 'update'
}

/**
 * Decide which page attachments to prune after a push. Pure and unit-tested.
 * An attachment is pruned only when it is BOTH (a) managed — i.e. it carries our
 * `sha256:` gate comment, so we uploaded it — AND (b) no longer referenced (its
 * title is not in `keepTitles`, the set of files we just pushed). A rename is a
 * remove+add, so the old title falls out here and the new one uploads normally.
 * Unreferenced attachments we do NOT manage (client-uploaded images, inline-
 * comment media) are never deleted — they are returned as `warnings` to surface.
 *
 * @param {Array<{ id: string, title: string, managed: boolean }>} attachments
 * @param {Iterable<string>} keepTitles  basenames of the files just pushed
 * @returns {{ prune: Array<{id:string,title:string}>, warnings: string[] }}
 */
export function decidePrune({ attachments, keepTitles }) {
  const keep = new Set(keepTitles)
  const prune = []
  const warnings = []
  for (const att of attachments || []) {
    if (keep.has(att.title)) continue
    if (att.managed) prune.push({ id: att.id, title: att.title })
    else warnings.push(att.title)
  }
  return { prune, warnings }
}

/**
 * Hash-gate a single local file against a page attachment of the same name.
 * `index` is an optional pre-fetched `Map(title -> attachment)` so a batch can
 * list attachments once. Returns the decision (and performs it unless `dry`).
 * Internal — call syncAttachmentsToPage (the batch entry point) instead.
 *
 * @returns {Promise<{file:string, action:'upload'|'update'|'skip', hash:string}>}
 */
async function syncAttachment({ pageId, absPath, dry = false, index = null }) {
  const file = basename(absPath)
  // A referenced asset with no local file (a dangling `./assets/...` link, or an
  // external `http(s)://` image that never resolves to a path) is skipped with a
  // WARN rather than crashing the whole publish on ENOENT.
  if (!existsSync(absPath)) {
    process.stdout.write(`WARN: referenced asset not found locally, skipping: ${file}\n`)
    return { file, action: 'missing', hash: null }
  }
  const localHash = sha256File(absPath)
  const att = index
    ? index.get(file)
    : (await listAttachments(pageId)).find((a) => a.title === file)
  const storedHash = att ? parseHash(await readStoredComment(att)) : null
  const action = decideAction({ exists: Boolean(att), localHash, storedHash })

  if (!dry) {
    if (action === 'upload') {
      await uploadAttachment(pageId, absPath, formatComment(localHash))
    } else if (action === 'update') {
      await updateAttachment(pageId, att.id, absPath, formatComment(localHash))
    }
  }

  return { file, action, hash: localHash }
}

/**
 * Hash-gated push of an arbitrary set of local files (absolute paths) to a page.
 * Lists page attachments once and reuses the index across all files.
 *
 * When `prune` is set, managed attachments (carrying our `sha256:` gate) whose
 * name is no longer among `files` are deleted (removed/renamed images); unmanaged
 * unreferenced attachments are returned in `warnings`, never deleted. `dry`
 * computes every decision (including the prune plan in `prunedTitles`) without
 * calling any write API.
 *
 * @param {object} o
 * @param {string} o.pageId
 * @param {string[]} o.files    absolute paths of the files to sync
 * @param {boolean} [o.dry]     compute decisions without calling write APIs
 * @param {boolean} [o.prune]   delete managed orphans no longer referenced
 * @returns {Promise<{uploaded:number,updated:number,skipped:number,missing:number,pruned:number,prunedTitles:string[],warnings:string[],results:Array<{file:string,action:string,hash:string}>}>}
 */
export async function syncAttachmentsToPage({ pageId, files, dry = false, prune = false }) {
  const existing = await listAttachments(pageId)
  const index = new Map(existing.map((a) => [a.title, a]))

  const results = []
  let uploaded = 0
  let updated = 0
  let skipped = 0
  let missing = 0

  for (const absPath of files) {
    const r = await syncAttachment({ pageId, absPath, dry, index })
    if (r.action === 'upload') uploaded += 1
    else if (r.action === 'update') updated += 1
    else if (r.action === 'missing') missing += 1
    else skipped += 1
    results.push(r)
  }

  let pruned = 0
  let prunedTitles = []
  let warnings = []
  if (prune) {
    const keepTitles = files.map((f) => basename(f))
    const resolved = []
    for (const att of existing) {
      const managed = parseHash(await readStoredComment(att)) !== null
      resolved.push({ id: att.id, title: att.title, managed })
    }
    const decision = decidePrune({ attachments: resolved, keepTitles })
    warnings = decision.warnings
    prunedTitles = decision.prune.map((p) => p.title)
    for (const p of decision.prune) {
      if (!dry) await deleteAttachment(pageId, p.id)
      pruned += 1
    }
  }

  return { uploaded, updated, skipped, missing, pruned, prunedTitles, warnings, results }
}
