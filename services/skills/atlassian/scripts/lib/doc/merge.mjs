/**
 * Body-scoped, git-native 3-way merge for canonical FSD/ISD documents.
 *
 * The round-trip problem: between two publishes a document can drift on BOTH
 * sides — locally (the working copy edited in a feature branch) and remotely
 * (the live Confluence page edited on the wiki). Republishing the working copy
 * blindly would clobber the wiki edits; republishing the pull would clobber the
 * local edits. A 3-way merge with a common ancestor reconciles them and only
 * flags genuine overlaps as conflicts.
 *
 * Why body-scoped: the document chrome (title, header card, approval roster,
 * status lozenge, `- Confluence:` / `- Page ID:` meta) almost always differs
 * across base/ours/theirs — status is ours to set, the meta stamp is
 * instance-specific, approvals get filled on the wiki. Feeding chrome to a text
 * merge produces nothing but false conflicts. So the 3-way runs over the BODY
 * only (the region from the first body section to EOF); chrome is resolved
 * deterministically — the working copy's chrome is kept (status = ours, meta =
 * script-owned) and any wiki-side chrome change is SURFACED (see `chromeDrift`)
 * for the human to port, never silently merged.
 *
 * The engine is `git merge-file` (git plumbing) — it works on plain files, needs
 * no repository, and writes standard `<<<<<<< / ||||||| / ======= / >>>>>>>`
 * conflict markers, so a conflict lands as markers in the working copy body that
 * the human resolves, commits, and re-runs — exactly the git workflow devs know.
 *
 * API:
 *   splitHeadBody(md, bodySections) → { head, body, found }
 *   mergeDocBody({ base, ours, theirs, bodySections })
 *     → { merged, conflicts, chromeChanges }
 *   chromeDrift(baseHead, theirsHead) → string[]   (wiki chrome lines not in base)
 */

import { execFileSync } from 'node:child_process'
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { collectHeadings } from './model.mjs'

// ─── Head / body split ─────────────────────────────────────────────────────────

/**
 * Split a canonical document into its chrome `head` and its `body`, cut at the
 * first body-section H2 (case-insensitive, fence-aware). `bodySections` is the
 * typed vocabulary of H2 titles that begin the body (from getDocType().bodySections).
 *
 * The cut is textual (raw line slices, not a parse→serialize round-trip) so the
 * working copy's head — including the `- Confluence:` / `- Page ID:` meta lines
 * that the structured model deliberately drops — is preserved byte-for-byte.
 *
 * `found` is false when no body section is present; callers must not body-merge a
 * document whose body boundary is unknown (it would silently drop everything
 * below the chrome).
 */
export function splitHeadBody(md, bodySections = []) {
  const text = String(md == null ? '' : md)
  const lines = text.split(/\r?\n/)
  const bodySet = new Set(bodySections.map((s) => String(s).toLowerCase()))

  let bodyStart = -1
  for (const h of collectHeadings(text)) {
    if (h.level === 2 && bodySet.has(h.title.toLowerCase())) { bodyStart = h.i; break }
  }
  if (bodyStart === -1) return { head: text, body: '', found: false }

  return {
    head: lines.slice(0, bodyStart).join('\n'),
    body: lines.slice(bodyStart).join('\n'),
    found: true,
  }
}

// ─── 3-way merge ────────────────────────────────────────────────────────────────

/**
 * 3-way merge a canonical document, scoping the text merge to the body.
 *
 *   base   — the common ancestor (the working copy at its seed/first-appearance commit,
 *            i.e. the discovery baseline pulled from the wiki)
 *   ours   — the current working copy
 *   theirs — the freshly-pulled live page (canonical markdown, gitignored baseline)
 *
 * Returns:
 *   merged        — ours' chrome + the 3-way-merged body (with conflict markers
 *                   inline when `conflicts > 0`)
 *   conflicts     — number of conflict hunks (0 = clean apply)
 *   chromeChanges — wiki-side chrome lines absent from base (informational; the
 *                   human ports these — chrome is never silently merged)
 */
export function mergeDocBody({ base, ours, theirs, bodySections = [] }) {
  const oursSplit = splitHeadBody(ours, bodySections)
  if (!oursSplit.found) {
    throw new Error(
      'Cannot 3-way merge: no body section found in the working copy ' +
        `(expected one of: ${bodySections.join(', ') || '<none configured>'}).`,
    )
  }
  const baseSplit = splitHeadBody(base, bodySections)
  const theirsSplit = splitHeadBody(theirs, bodySections)

  const { merged: mergedBody, conflicts } = gitMergeFile(
    baseSplit.body,
    oursSplit.body,
    theirsSplit.body,
  )

  const head = oursSplit.head.replace(/\s+$/, '')
  const body = mergedBody.replace(/^\s+/, '').replace(/\s+$/, '')
  const merged = body ? `${head}\n\n${body}\n` : `${head}\n`

  return { merged, conflicts, chromeChanges: chromeDrift(baseSplit.head, theirsSplit.head) }
}

// ─── Chrome drift (wiki-side chrome changes) ────────────────────────────────────

// Instance-specific / script-owned meta lines never count as drift: they are
// stamped per working copy and always differ across base/ours/theirs.
const isMetaLine = (line) => /^-\s*(confluence|page\s*id)\s*:/i.test(line)

const normChromeLines = (head) =>
  String(head || '')
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l && !isMetaLine(l))

/**
 * Chrome lines present on the wiki (`theirs`) but not in the ancestor (`base`) —
 * i.e. header-card / approval-roster / references edits made directly on the
 * page since it was last synced. Surfaced so the human can port them into the
 * working copy; the merge itself keeps ours' chrome rather than merging these.
 */
export function chromeDrift(baseHead, theirsHead) {
  const baseSet = new Set(normChromeLines(baseHead))
  return normChromeLines(theirsHead).filter((l) => !baseSet.has(l))
}

// ─── git merge-file wrapper ─────────────────────────────────────────────────────

const withTrailingNewline = (s) => {
  const t = String(s == null ? '' : s)
  return t === '' || t.endsWith('\n') ? t : `${t}\n`
}

/**
 * Run `git merge-file -p --diff3` over three body strings via temp files.
 * `-p` prints the merged result to stdout (leaving inputs untouched); the exit
 * code is the number of conflict hunks (0 = clean), or negative on error.
 */
function gitMergeFile(baseBody, oursBody, theirsBody) {
  const dir = mkdtempSync(join(tmpdir(), 'doc-merge-'))
  try {
    const ours = join(dir, 'ours')
    const base = join(dir, 'base')
    const theirs = join(dir, 'theirs')
    writeFileSync(ours, withTrailingNewline(oursBody))
    writeFileSync(base, withTrailingNewline(baseBody))
    writeFileSync(theirs, withTrailingNewline(theirsBody))

    const args = [
      'merge-file', '-p', '--diff3',
      '-L', 'working copy', '-L', 'base (discovery baseline)', '-L', 'confluence (live)',
      ours, base, theirs,
    ]
    try {
      const merged = execFileSync('git', args, { encoding: 'utf8' })
      return { merged, conflicts: 0 }
    } catch (err) {
      // Non-zero exit with a positive status = conflict count; stdout still holds
      // the merged text (with markers). Anything else is a real failure.
      if (typeof err.status === 'number' && err.status > 0 && typeof err.stdout === 'string') {
        return { merged: err.stdout, conflicts: err.status }
      }
      throw new Error(`git merge-file failed: ${err.stderr || err.message}`)
    }
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
}
