/**
 * End-to-end smoke test of the browserless pipeline: spec -> emit -> SVG -> PNG. Guards the
 * two things that silently break rendering: foreignObject labels and a missing bundled font.
 */
import test from 'node:test'
import assert from 'node:assert/strict'
import { existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { renderPng } from '../render.mjs'
import { emit } from '../types/sequence/emit.mjs'
import { validSpec } from './_fixtures.mjs'

const here = dirname(fileURLToPath(import.meta.url))

test('the bundled Inter font ships with the skill', () => {
  assert.ok(existsSync(join(here, '..', 'assets', 'fonts', 'Inter.ttf')), 'Inter.ttf must be vendored for cross-OS output')
})

test('spec renders to a real PNG via native-text SVG', () => {
  const { svg, png } = renderPng(emit(validSpec()))
  assert.ok(svg.includes('<svg'))
  assert.ok(!/foreignObject/i.test(svg), 'SVG must contain no foreignObject (would be dropped by resvg)')
  assert.ok(png.length > 100, 'PNG should be non-trivial')
  // PNG magic number: 0x89 'P' 'N' 'G'.
  assert.deepEqual([png[0], png[1], png[2], png[3]], [0x89, 0x50, 0x4e, 0x47])
})
