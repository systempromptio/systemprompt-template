/**
 * Asset reconciliation helpers for the FSD/ISD round-trip.
 *
 * Binaries are never 3-way merged; instead the markdown body merge decides which
 * images the reconciled document references, and this module decides which of
 * those must be LOCALIZED into the working copy's `./assets`. The submit-time
 * baseline pull stages the live wiki's binaries in a throwaway dir; after the
 * body merge, any referenced image that is not already local but IS staged came
 * from the wiki (a client added it) and is copied in. Anything referenced but
 * present in neither place is a dangling link surfaced as a warning.
 *
 * Pure and string/set-only so it is unit-testable offline; the actual file I/O
 * lives in the `merge-baseline.mjs` CLI driver.
 */
import { basename } from 'node:path'

/**
 * Local image basenames a document references. Skips remote (`http(s)://`) and
 * inline (`data:`) sources — only files that live under the working `./assets`
 * participate in localization / attachment sync.
 *
 * @param {string[]} assetPaths  raw `![](...)` src paths (e.g. from collectAssets)
 * @returns {string[]} de-duplicated local basenames
 */
export function referencedLocalAssetNames(assetPaths) {
  const names = []
  for (const p of assetPaths || []) {
    if (/^(https?:|data:)/i.test(String(p))) continue
    names.push(basename(String(p)))
  }
  return [...new Set(names)]
}

/**
 * Decide which referenced assets to localize from the staging dir and which are
 * dangling. Pure — operates on filename sets.
 *
 * @param {object} o
 * @param {string[]} o.referenced  local basenames the reconciled doc references
 * @param {Iterable<string>} o.present  basenames already in the working ./assets
 * @param {Iterable<string>} o.staged   basenames available in the staging dir
 * @returns {{ localize: string[], dangling: string[] }}
 */
export function planAssetLocalization({ referenced, present, staged }) {
  const have = new Set(present)
  const stage = new Set(staged)
  const localize = []
  const dangling = []
  for (const name of referenced || []) {
    if (have.has(name)) continue // already local (ours, possibly edited)
    if (stage.has(name)) localize.push(name) // wiki-origin -> copy into ./assets
    else dangling.push(name) // referenced but nowhere -> warn
  }
  return { localize, dangling }
}
