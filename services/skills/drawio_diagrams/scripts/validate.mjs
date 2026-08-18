#!/usr/bin/env node
/**
 * Validate diagram specs without producing files.
 *
 * Usage:
 *   node validate.mjs <spec-file.yaml>
 *   node validate.mjs --md <doc.md>          # validate ```drawio:<type>:<id> blocks
 *   cat spec.yaml | node validate.mjs --stdin
 *
 * Exit code 0 = all valid (warnings do not fail), 1 = validation errors, 2 = usage error.
 */
import { readFileSync } from 'node:fs'
import YAML from 'yaml'
import { parseArgs, die } from './lib/cli.mjs'
import { validateSpec, warnSpec, extractDrawioBlocks } from './lib/spec.mjs'

const { positional, flags } = parseArgs(process.argv.slice(2), { booleans: ['stdin'] })

let entries
if (flags.md) {
  const md = readFileSync(flags.md, 'utf8')
  // Header is authoritative for type+id; the body carries the scenario only.
  entries = extractDrawioBlocks(md).map(({ type, id, body }) => ({ type, id, ...(YAML.parse(body) || {}) }))
} else if (flags.stdin) {
  entries = YAML.parseAllDocuments(readFileSync(0, 'utf8')).filter((d) => d.contents != null).map((d) => d.toJS())
} else if (positional[0]) {
  entries = YAML.parseAllDocuments(readFileSync(positional[0], 'utf8')).filter((d) => d.contents != null).map((d) => d.toJS())
} else {
  die('Usage: node validate.mjs <spec-file.yaml> | --md <doc.md> | --stdin')
}

let hasErrors = false
entries.forEach((spec, i) => {
  const errs = validateSpec(spec)
  const warns = warnSpec(spec)
  const label = spec?.id ? `"${spec.id}"` : `#${i}`
  if (errs.length) {
    hasErrors = true
    console.error(`✗ spec ${label} (${spec?.type ?? 'unknown type'}):`)
    for (const e of errs) console.error(`    - ${e}`)
  } else {
    console.log(`✓ spec ${label} (${spec.type})`)
  }
  // Warnings never change the exit code — they advise, they do not gate.
  for (const w of warns) console.error(`    ⚠ ${w}`)
})

process.exit(hasErrors ? 1 : 0)
