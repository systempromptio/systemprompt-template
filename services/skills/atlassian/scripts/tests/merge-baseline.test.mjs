import { test } from 'node:test'
import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import { mkdtempSync, writeFileSync, readFileSync, rmSync, mkdirSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const SCRIPT = join(HERE, '..', 'merge-baseline.mjs')

const DOC = (body) =>
  [
    '# Store Locator FSD',
    '',
    '- Page ID: 123',
    '',
    '## General FSD Information',
    '',
    '|  |  |',
    '| --- | --- |',
    '| FSD Status | on review |',
    '',
    body.trim(),
    '',
  ].join('\n')

const BASE_BODY = [
  '## In Scope Functional Requirements',
  '',
  '### RQ-1 - Alpha',
  '',
  'Original alpha.',
  '',
  '### RQ-2 - Beta',
  '',
  'Original beta.',
].join('\n')

// A throwaway git repo with the working copy committed once (so the seed/add
// commit — the merge ancestor — exists in history), then diverged locally.
function scaffold({ ours, theirs }) {
  const repo = mkdtempSync(join(tmpdir(), 'merge-baseline-'))
  const git = (args) => execFileSync('git', args, { cwd: repo, encoding: 'utf8' })
  git(['init', '-q'])
  git(['config', 'user.email', 'test@example.com'])
  git(['config', 'user.name', 'Test'])

  const relDir = join('openspec', 'changes', 'store-locator', 'fsd')
  mkdirSync(join(repo, relDir), { recursive: true })
  const workingRel = join(relDir, 'store-locator.md')
  const baselineRel = join(relDir, 'store-locator.baseline.md')

  writeFileSync(join(repo, workingRel), DOC(BASE_BODY)) // ancestor
  git(['add', '.'])
  git(['commit', '-qm', 'seed'])

  writeFileSync(join(repo, workingRel), ours) // local (uncommitted) edit
  writeFileSync(join(repo, baselineRel), theirs) // simulated fresh wiki pull

  return { repo, workingRel, baselineRel, workingAbs: join(repo, workingRel) }
}

const run = (repo, args) => {
  try {
    const stdout = execFileSync('node', [SCRIPT, ...args], { cwd: repo, encoding: 'utf8' })
    return { status: 0, stdout }
  } catch (err) {
    return { status: err.status, stdout: err.stdout || '', stderr: err.stderr || '' }
  }
}

test('CLI: derives base from git history and reconciles independent edits (exit 0)', () => {
  const ours = DOC(BASE_BODY.replace('Original alpha.', 'Local alpha edit.'))
  const theirs = DOC(BASE_BODY.replace('Original beta.', 'Wiki beta edit.'))
  const { repo, workingRel, baselineRel, workingAbs } = scaffold({ ours, theirs })
  try {
    const r = run(repo, [workingRel, baselineRel, '--type=fsd'])
    assert.equal(r.status, 0, r.stderr)
    const merged = readFileSync(workingAbs, 'utf8')
    assert.match(merged, /Local alpha edit\./)
    assert.match(merged, /Wiki beta edit\./)
    assert.doesNotMatch(merged, /<<<<<<</)
  } finally {
    rmSync(repo, { recursive: true, force: true })
  }
})

test('CLI: overlapping edits write conflict markers into the working copy (exit 2)', () => {
  const ours = DOC(BASE_BODY.replace('Original alpha.', 'Local wins.'))
  const theirs = DOC(BASE_BODY.replace('Original alpha.', 'Wiki wins.'))
  const { repo, workingRel, baselineRel, workingAbs } = scaffold({ ours, theirs })
  try {
    const r = run(repo, [workingRel, baselineRel, '--type=fsd'])
    assert.equal(r.status, 2)
    const merged = readFileSync(workingAbs, 'utf8')
    assert.match(merged, /<<<<<<< working copy/)
    assert.match(merged, />>>>>>> confluence \(live\)/)
  } finally {
    rmSync(repo, { recursive: true, force: true })
  }
})

test('CLI: --check reports conflicts without writing the working copy', () => {
  const ours = DOC(BASE_BODY.replace('Original alpha.', 'Local wins.'))
  const theirs = DOC(BASE_BODY.replace('Original alpha.', 'Wiki wins.'))
  const { repo, workingRel, baselineRel, workingAbs } = scaffold({ ours, theirs })
  try {
    const before = readFileSync(workingAbs, 'utf8')
    const r = run(repo, [workingRel, baselineRel, '--type=fsd', '--check'])
    assert.equal(r.status, 2)
    assert.equal(readFileSync(workingAbs, 'utf8'), before, 'working copy untouched under --check')
  } finally {
    rmSync(repo, { recursive: true, force: true })
  }
})

test('CLI: --assets-from localizes a wiki-added image and warns on a dangling ref', () => {
  // Wiki (theirs) added TWO image refs to Beta: one it also attached (wiki.png,
  // staged) and one dangling (ghost.png, nowhere). Ours edited Alpha only.
  const ours = DOC(BASE_BODY.replace('Original alpha.', 'Local alpha edit.'))
  const theirs = DOC(
    BASE_BODY.replace(
      'Original beta.',
      'Original beta.\n\n![flow](./assets/wiki.png)\n\n![missing](./assets/ghost.png)',
    ),
  )
  const { repo, workingRel, baselineRel, workingAbs } = scaffold({ ours, theirs })
  try {
    // Staging dir holds the wiki-origin binary (as the --assets-dir pull would).
    const stagingRel = join('openspec', 'changes', 'store-locator', 'fsd', '.store-locator.baseline-assets')
    mkdirSync(join(repo, stagingRel), { recursive: true })
    writeFileSync(join(repo, stagingRel, 'wiki.png'), 'PNGDATA')

    const r = run(repo, [workingRel, baselineRel, '--type=fsd', `--assets-from=${stagingRel}`])
    assert.equal(r.status, 0, r.stderr)

    const localized = join(repo, 'openspec', 'changes', 'store-locator', 'fsd', 'assets', 'wiki.png')
    assert.ok(existsSync(localized), 'wiki-added image copied into ./assets')
    assert.equal(readFileSync(localized, 'utf8'), 'PNGDATA')
    assert.match(readFileSync(workingAbs, 'utf8'), /!\[flow\]\(\.\/assets\/wiki\.png\)/)

    assert.match(r.stdout, /Localized 1 wiki asset/)
    assert.match(r.stdout, /ghost\.png/) // dangling ref surfaced as a WARN
  } finally {
    rmSync(repo, { recursive: true, force: true })
  }
})

test('CLI: --assets-from never overwrites a locally-present (edited) image', () => {
  // Both sides add mine.png identically (clean), and ours also edits Alpha; the
  // image already exists locally with edited bytes, staging holds different bytes.
  const withImg = BASE_BODY.replace('Original beta.', 'Original beta.\n\n![m](./assets/mine.png)')
  const ours = DOC(withImg.replace('Original alpha.', 'Local alpha edit.'))
  const theirs = DOC(withImg)
  const { repo, workingRel, baselineRel } = scaffold({ ours, theirs })
  try {
    const assetsRel = join('openspec', 'changes', 'store-locator', 'fsd', 'assets')
    mkdirSync(join(repo, assetsRel), { recursive: true })
    writeFileSync(join(repo, assetsRel, 'mine.png'), 'LOCAL-EDITED')
    const stagingRel = join('openspec', 'changes', 'store-locator', 'fsd', '.store-locator.baseline-assets')
    mkdirSync(join(repo, stagingRel), { recursive: true })
    writeFileSync(join(repo, stagingRel, 'mine.png'), 'WIKI-VERSION')

    const r = run(repo, [workingRel, baselineRel, '--type=fsd', `--assets-from=${stagingRel}`])
    assert.equal(r.status, 0, r.stderr)
    assert.equal(
      readFileSync(join(repo, assetsRel, 'mine.png'), 'utf8'),
      'LOCAL-EDITED',
      'local image bytes preserved, not clobbered by staging',
    )
  } finally {
    rmSync(repo, { recursive: true, force: true })
  }
})

test('CLI: errors (exit 3) when the working copy has no committed ancestor', () => {
  const ours = DOC(BASE_BODY)
  const theirs = DOC(BASE_BODY)
  const { repo, baselineRel } = scaffold({ ours, theirs })
  try {
    // Point at an untracked path: no add commit → no derivable base.
    const untracked = join('openspec', 'changes', 'store-locator', 'fsd', 'untracked.md')
    writeFileSync(join(repo, untracked), ours)
    const r = run(repo, [untracked, baselineRel, '--type=fsd'])
    assert.equal(r.status, 3)
    assert.match(r.stderr, /ancestor/i)
  } finally {
    rmSync(repo, { recursive: true, force: true })
  }
})
