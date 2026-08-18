/**
 * Tiny Node I/O helpers shared across the Atlassian CLIs and exporters, so the
 * same "read-file-or-treat-as-literal" and bounded-concurrency patterns aren't
 * copy-pasted into every script.
 *
 * Responsibility: filesystem + concurrency plumbing only. No Confluence/Jira
 *   knowledge. Edit here when the shared I/O behaviour changes.
 */

import { readFileSync } from 'node:fs'
import { stat } from 'node:fs/promises'

/**
 * Read `input` as a UTF-8 file; if it is not a readable path, return `input`
 * verbatim (so a CLI arg can be either a file path or an inline literal).
 */
export function tryReadFile(input) {
  try {
    return readFileSync(input, 'utf8')
  } catch {
    return input
  }
}

/** True when `path` exists (any type). Never throws. */
export async function fileExists(path) {
  try {
    await stat(path)
    return true
  } catch {
    return false
  }
}

/** `decodeURIComponent` that returns the input unchanged on a malformed escape. */
export function safeDecodeURIComponent(input) {
  try {
    return decodeURIComponent(input)
  } catch {
    return input
  }
}

/**
 * Make a Confluence attachment title / URL basename safe to use as an on-disk
 * file name: decode percent-escapes, flatten path separators, strip accents and
 * anything outside `[A-Za-z0-9._-]`, and bound the length. Shared by the bulk
 * exporter and the typed reverse pull so a pulled asset lands under the same
 * name in both.
 */
export function sanitizeFileName(input) {
  const decoded = safeDecodeURIComponent(input)
  const withoutSlashes = decoded.replaceAll('/', '-').replaceAll('\\', '-')
  const normalized = withoutSlashes.normalize('NFKD').replace(/[\u0300-\u036f]/g, '')
  const cleaned = normalized.replace(/[^a-zA-Z0-9._-]+/g, '-').replace(/^-+|-+$/g, '')

  return cleaned.length > 0 ? cleaned.slice(0, 160) : 'asset'
}

/**
 * Run thunks with bounded concurrency, preserving result order.
 * @template T
 * @param {Array<() => Promise<T>>} tasks
 * @param {number} limit
 * @returns {Promise<T[]>}
 */
export async function withConcurrency(tasks, limit) {
  const results = new Array(tasks.length)
  let next = 0
  const workers = Array.from({ length: Math.max(1, Math.min(limit, tasks.length)) }, async () => {
    while (next < tasks.length) {
      const i = next++
      results[i] = await tasks[i]()
    }
  })
  await Promise.all(workers)
  return results
}
