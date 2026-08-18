#!/usr/bin/env node
/**
 * Reconcile a freshly-pulled Confluence baseline into the FSD/ISD working copy
 * with a git-native, body-scoped 3-way merge — the safe-re-publish gate.
 *
 * The three merge inputs:
 *   ours    – the current working copy (<feature>.md)
 *   theirs  – the freshly-pulled live page (<feature>.baseline.md, gitignored),
 *             produced by `confluence.mjs get-page <id> --type=<type> --into ...`
 *   base    – the common ancestor: <feature>.md as it stood at its seed/add
 *             commit, read straight from git history (`git show <sha>:<path>`).
 *             No committed baseline file — git IS the durable record.
 *
 * On a clean apply the working copy is rewritten in place with the merged result
 * (local edits + wiki edits reconciled) and the command exits 0. On conflicts
 * the merged text — WITH `<<<<<<<` markers — is written into the working copy so
 * the human resolves them there, commits, and re-runs; the command exits 2.
 * Wiki-side chrome changes (approvals/references filled on the page) are surfaced
 * as NOTES to port by hand — chrome is kept from the working copy, never merged.
 *
 * Usage:
 *   node merge-baseline.mjs <working.md> <baseline.md> --type=<fsd|isd> \
 *     [--base-ref=<gitref>] [--assets-from=<dir>] [--check]
 *
 * Flags:
 *   --type=<fsd|isd>   doc type (supplies the body-section vocabulary). Required.
 *   --base-ref=<ref>   override the derived ancestor with `git show <ref>:<path>`
 *                      (e.g. a tag or the last-publish commit). Advanced/testing.
 *   --assets-from=<dir> after a clean merge, localize any newly-referenced (wiki-
 *                      origin) images from <dir> into the working copy's ./assets
 *                      (the submit baseline-asset staging dir). Dangling refs warn.
 *   --check            do not write the working copy; report only (dry preview).
 *
 * Exit codes: 0 = clean (working copy updated), 2 = conflicts (markers written),
 *   3 = usage / no derivable ancestor.
 */

import { execFileSync } from 'node:child_process'
import { readFileSync, writeFileSync, existsSync, mkdirSync, copyFileSync, readdirSync } from 'node:fs'
import { dirname, join } from 'node:path'

import { parseArgs } from './lib/util/cli-args.mjs'
import { getDocType, listDocTypes } from './lib/doc/types/index.mjs'
import { mergeDocBody } from './lib/doc/merge.mjs'
import { collectAssets } from './lib/doc/md-to-storage.mjs'
import { referencedLocalAssetNames, planAssetLocalization } from './lib/doc/assets.mjs'

const { flags, positional } = parseArgs(process.argv.slice(2), { booleans: ['check'] })
const [workingPath, baselinePath] = positional
const type = flags.type ? String(flags.type).toLowerCase() : ''

function die(msg) {
  console.error(`ERROR: ${msg}`)
  process.exit(3)
}

if (!workingPath || !baselinePath) die('Usage: merge-baseline.mjs <working.md> <baseline.md> --type=<fsd|isd>')
if (!type) die('--type=<fsd|isd> is required.')
if (!listDocTypes().includes(type)) die(`Unknown --type "${type}". Known types: ${listDocTypes().join(', ')}.`)
if (!existsSync(workingPath)) die(`Working copy not found: ${workingPath}`)
if (!existsSync(baselinePath)) die(`Baseline not found: ${baselinePath} (run get-page --type=${type} --into first)`)

const git = (args) => execFileSync('git', args, { encoding: 'utf8' })

// Repo-relative path for `git show` (works regardless of the shell's cwd within
// the repo). Empty when the file is untracked — then there is no ancestor.
function repoRelPath(path) {
  try {
    const rel = git(['ls-files', '--full-name', '--', path]).split(/\r?\n/).find(Boolean)
    return rel || ''
  } catch {
    return ''
  }
}

// Ancestor content = the working copy at its seed/add commit (the FIRST commit
// that added the path), read from history. This is the deterministic no-pointer
// base the round-trip model relies on: both the live page and the current
// working copy descend from it. `--base-ref` overrides it when a caller knows a
// tighter ancestor (e.g. the exact last-publish commit).
function deriveBase(path) {
  const rel = repoRelPath(path)
  if (flags['base-ref']) {
    const ref = String(flags['base-ref'])
    if (!rel) return null
    try { return git(['show', `${ref}:${rel}`]) } catch { return null }
  }
  if (!rel) return null
  let seed = ''
  try {
    const shas = git(['log', '--diff-filter=A', '--format=%H', '--', rel]).split(/\r?\n/).filter(Boolean)
    seed = shas[shas.length - 1] || ''
  } catch {
    return null
  }
  if (!seed) return null
  try { return git(['show', `${seed}:${rel}`]) } catch { return null }
}

const ours = readFileSync(workingPath, 'utf8')
const theirs = readFileSync(baselinePath, 'utf8')
const base = deriveBase(workingPath)

if (base == null) {
  die(
    `Could not derive a merge ancestor for ${workingPath} from git history. ` +
      'Commit the working copy on the feature branch first (the round-trip merge needs a committed ' +
      'ancestor), or pass --base-ref=<commit> to name one explicitly.',
  )
}

const bodySections = getDocType(type).bodySections
let result
try {
  result = mergeDocBody({ base, ours, theirs, bodySections })
} catch (err) {
  die(err.message)
}

// Report --------------------------------------------------------------------------
if (result.chromeChanges.length) {
  console.log('NOTE — wiki chrome changed since the last sync (kept working-copy chrome; port by hand if needed):')
  for (const line of result.chromeChanges) console.log(`  + ${line}`)
  console.log('')
}

if (result.conflicts > 0) {
  if (!flags.check) writeFileSync(workingPath, result.merged, 'utf8')
  console.error(
    `CONFLICT — ${result.conflicts} hunk(s) between local and live edits.` +
      (flags.check ? ' (--check: working copy not modified)' : ` Conflict markers written into ${workingPath}.`),
  )
  console.error('Resolve the <<<<<<< markers in the working copy, commit, then re-run this merge.')
  process.exit(2)
}

if (flags.check) {
  console.log(`Clean 3-way merge — ${workingPath} would apply without conflicts (--check: not written).`)
  process.exit(0)
}

const changed = result.merged !== ours
writeFileSync(workingPath, result.merged, 'utf8')
console.log(
  changed
    ? `Clean 3-way merge — reconciled live edits into ${workingPath}.`
    : `Clean 3-way merge — ${workingPath} already up to date (no changes).`,
)

// Asset localization: after a clean merge, pull any newly-referenced (wiki-origin)
// binaries from the staging dir into the working copy's ./assets, so publish sees
// a complete asset set. Binaries are never merged — this is a deterministic copy
// of images the reconciled body references but does not yet have locally.
if (flags['assets-from']) {
  const stagingDir = String(flags['assets-from'])
  const assetsDir = join(dirname(workingPath), 'assets')
  const referenced = referencedLocalAssetNames(collectAssets(result.merged))
  const present = existsSync(assetsDir) ? readdirSync(assetsDir) : []
  const staged = existsSync(stagingDir) ? readdirSync(stagingDir) : []
  const { localize, dangling } = planAssetLocalization({ referenced, present, staged })
  if (localize.length) {
    mkdirSync(assetsDir, { recursive: true })
    for (const name of localize) copyFileSync(join(stagingDir, name), join(assetsDir, name))
    console.log(`Localized ${localize.length} wiki asset(s) into ${assetsDir}: ${localize.join(', ')}`)
  }
  if (dangling.length) {
    console.log(`WARN — ${dangling.length} referenced asset(s) not found locally or in staging: ${dangling.join(', ')}`)
  }
}

process.exit(0)
