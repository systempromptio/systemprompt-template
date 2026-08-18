#!/usr/bin/env node
/**
 * Reverse converter: a generated `.drawio` -> the ```drawio:<type>:<id> authoring
 * block it was produced from. The `.drawio` is self-describing (it carries the
 * full spec as base64 `data-spec`), so this reconstructs the block WITHOUT parsing
 * draw.io geometry. It is the diagrams-side half of the Confluence -> markdown
 * reverse pull; the Atlassian skill downloads the attachment and splices the
 * block into the doc.
 *
 * Usage:
 *   node reverse.mjs --drawio <file.drawio>
 *
 * Prints a JSON manifest to stdout:
 *   { ok, id, type, title, png, block }
 *
 * Exit code 0 = ok, 1 = failure (JSON error envelope on stderr).
 */
import { readFileSync } from 'node:fs'
import { parseArgs, failJson as fail } from './lib/cli.mjs'
import { readEmbeddedSpec, specToBlock } from './lib/spec.mjs'

const { positional, flags } = parseArgs(process.argv.slice(2))
const file = flags.drawio || positional[0]
if (!file) fail('Usage: node reverse.mjs --drawio <file.drawio>')

let spec
try {
  spec = readEmbeddedSpec(readFileSync(file, 'utf8'))
} catch (err) {
  fail(`cannot read embedded spec from ${file}: ${err.message}`)
}

if (!spec || !spec.type || !spec.id) {
  fail(`embedded spec in ${file} is missing type/id`)
}

const manifest = {
  ok: true,
  id: spec.id,
  type: spec.type,
  title: spec.title ?? spec.id,
  png: `${spec.id}.png`,
  block: specToBlock(spec),
}
console.log(JSON.stringify(manifest, null, 2))
