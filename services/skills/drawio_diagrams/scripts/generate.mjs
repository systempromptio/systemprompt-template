#!/usr/bin/env node
/**
 * Diagram generator — a pure producer (the primary CLI, type-agnostic).
 *
 * Responsibility: read one or more specs (YAML), validate everything up front (invalid =>
 *   no output), then for each spec route to its registered type to emit `.drawio` + render
 *   `.png`, and print a JSON manifest of paths. It does NOT edit any markdown; inserting the
 *   image reference is the caller's job.
 * Edit here when: you change the CLI surface, output naming, or the manifest shape. For how
 *   a diagram is drawn, edit the type under `types/<type>/`.
 * Do NOT: add type-specific branching here — dispatch stays through the registry.
 *
 * Usage:
 *   node generate.mjs --out-dir <dir> --spec-file <file.yaml>
 *   node generate.mjs --out-dir <dir> --spec-stdin < spec.yaml
 *   node generate.mjs --out-dir <dir> --md <doc.md>        # extract ```drawio:<type>:<id> blocks
 *   [--name <slug>] [--scale 1.5] [--background #ffffff|transparent] [--no-png]
 */
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { join } from 'node:path'
import YAML from 'yaml'
import { parseArgs, failJson as fail } from './lib/cli.mjs'
import { validateSpec, warnSpec, extractDrawioBlocks, slugify } from './lib/spec.mjs'
import { getType } from './types/index.mjs'
import { renderPng } from './render.mjs'

function loadSpecs(flags) {
  if (flags.md) {
    const md = readFileSync(flags.md, 'utf8')
    const blocks = extractDrawioBlocks(md)
    if (!blocks.length) fail(`no \`\`\`drawio:<type>:<id> blocks found in ${flags.md}`)
    // Header is authoritative for type+id; the body carries the scenario only.
    return blocks.map(({ type, id, body }) => {
      const spec = { type, id, ...(YAML.parse(body) || {}) }
      return { text: YAML.stringify(spec), spec }
    })
  }
  let text
  if (flags['spec-file']) text = readFileSync(flags['spec-file'], 'utf8')
  else if (flags['spec-stdin']) text = readFileSync(0, 'utf8')
  else fail('provide one of --spec-file, --spec-stdin, or --md')

  const docs = YAML.parseAllDocuments(text)
  return docs
    .filter((d) => d.contents != null)
    .map((d) => ({ text: String(d), spec: d.toJS() }))
}

const { flags } = parseArgs(process.argv.slice(2), { booleans: ['spec-stdin', 'no-png'] })
if (!flags['out-dir']) fail('--out-dir is required')

const entries = loadSpecs(flags)

// Validate everything up front; produce nothing on any error.
const problems = []
entries.forEach(({ spec }, i) => {
  const errs = validateSpec(spec)
  if (errs.length) problems.push({ index: i, id: spec?.id, errors: errs })
})
if (problems.length) fail({ message: 'spec validation failed', problems })

mkdirSync(flags['out-dir'], { recursive: true })

const seen = new Set()
const diagrams = entries.map(({ spec, text }, i) => {
  const type = getType(spec.type)
  let slug = slugify(flags.name && entries.length === 1 ? flags.name : spec.id)
  while (seen.has(slug)) slug = `${slug}-${i}`
  seen.add(slug)

  const drawioXml = type.emit(spec, { specYaml: text })
  const drawioPath = join(flags['out-dir'], `${slug}.drawio`)
  writeFileSync(drawioPath, drawioXml)

  const result = { id: spec.id, type: spec.type, title: spec.title ?? spec.id, slug, drawio: drawioPath }

  // Soft advisories (e.g. text over its box budget) never block a file — surface them so the
  // caller can tighten the wording, but the diagram still renders.
  const warns = warnSpec(spec)
  if (warns.length) result.warnings = warns

  if (!flags['no-png']) {
    const { png } = renderPng(drawioXml, {
      background: flags.background ?? '#ffffff',
      scale: flags.scale ? Number(flags.scale) : 1.5,
    })
    const pngPath = join(flags['out-dir'], `${slug}.png`)
    writeFileSync(pngPath, png)
    result.png = pngPath
  }
  return result
})

console.log(JSON.stringify({ ok: true, diagrams }, null, 2))
