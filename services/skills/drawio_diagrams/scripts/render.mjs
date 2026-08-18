#!/usr/bin/env node
/**
 * Render a .drawio (mxGraph XML) file to SVG and PNG — fully browserless. Type-agnostic.
 *
 * Responsibility: rasterize any render-safe mxGraph XML. Pipeline: drawio2svg (vendored
 *   bundle) -> SVG string -> @resvg/resvg-js -> PNG, using a bundled Inter font so output is
 *   byte-identical on every OS. Only native-text styles rasterize (see styles.mjs).
 * Inputs: XML string. Outputs: { svg, png }.
 * Edit here when: you change rasterization options (scale, background, font) or the SVG step.
 * Do NOT: assume system fonts are available; the bundled font is what guarantees determinism.
 *
 * Usage: node render.mjs <input.drawio> [output.png] [--scale=1.5] [--background=#ffffff|transparent]
 */
import { readFileSync, writeFileSync, existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { DOMParser, XMLSerializer } from '@xmldom/xmldom'
import { Resvg } from '@resvg/resvg-js'
import { parseArgs, die } from './lib/cli.mjs'
import { makeHeuristicTextMeasureProvider } from './lib/text-measure.mjs'

// The vendored bundle uses global DOMParser to parse the .drawio XML and to build
// the SVG document, and XMLSerializer to serialize it. xmldom provides both (the
// native-text render path never touches HTML/foreignObject).
if (!globalThis.DOMParser) globalThis.DOMParser = DOMParser
if (!globalThis.XMLSerializer) globalThis.XMLSerializer = XMLSerializer

const here = dirname(fileURLToPath(import.meta.url))
const { convert, setTextMeasureProvider } = await import(join(here, 'lib', 'drawio2svg.mjs'))
setTextMeasureProvider(makeHeuristicTextMeasureProvider())

// Bundled font => byte-identical rendering on Windows/macOS/Linux with nothing to install.
// We disable system fonts so resvg can ONLY use this file (Inter, a variable TTF covering
// all weights). If it's ever missing, we fall back to system fonts rather than blank text.
const FONT_FILE = join(here, 'assets', 'fonts', 'Inter.ttf')
const FONT = existsSync(FONT_FILE)
  ? { fontFiles: [FONT_FILE], loadSystemFonts: false, defaultFontFamily: 'Inter', sansSerifFamily: 'Inter' }
  : { loadSystemFonts: true, defaultFontFamily: 'Inter' }

/**
 * @param {string} xml  render-safe mxGraph XML
 * @param {{ background?: string, padding?: number }} [opts]
 * @returns {string} SVG markup
 */
export function renderSvg(xml, { background = '#ffffff', padding = 24 } = {}) {
  const backgroundColor = background === 'transparent' ? null : background
  return convert(xml, { backgroundColor, padding })
}

/**
 * @param {string} xml  render-safe mxGraph XML
 * @param {{ background?: string, scale?: number, padding?: number }} [opts]
 * @returns {{ svg: string, png: Buffer }}
 */
export function renderPng(xml, { background = '#ffffff', scale = 1.5, padding = 24 } = {}) {
  const svg = renderSvg(xml, { background, padding })
  const opts = { fitTo: { mode: 'zoom', value: scale }, font: FONT }
  if (background && background !== 'transparent') opts.background = background
  const png = new Resvg(svg, opts).render().asPng()
  return { svg, png }
}

if (process.argv[1] && import.meta.url === `file://${process.argv[1]}`) {
  const { positional, flags } = parseArgs(process.argv.slice(2))
  const input = positional[0]
  if (!input) {
    die('Usage: node render.mjs <input.drawio> [output.png] [--scale=1.5] [--background=#ffffff|transparent]')
  }
  const output = positional[1] || input.replace(/\.(drawio|xml)$/i, '') + '.png'
  const xml = readFileSync(input, 'utf8')
  const { png } = renderPng(xml, {
    background: flags.background ?? '#ffffff',
    scale: flags.scale ? Number(flags.scale) : 1.5,
  })
  writeFileSync(output, png)
  console.log(output)
}
