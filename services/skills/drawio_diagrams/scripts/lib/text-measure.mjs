/**
 * Lightweight, DOM-free text measurement provider for drawio2svg in Node.
 *
 * drawio2svg's text-measure throws in a headless environment unless a provider is set
 * (there is no built-in fallback). We supply a deterministic heuristic: glyph width is
 * approximated as a fraction of the font size. This is good enough for laying out label
 * backgrounds / edge-label line counts; resvg rasterizes the actual glyphs from fonts.
 */
const AVG_GLYPH = 0.55 // average glyph advance as a fraction of font size

function stripTags(s) {
  return String(s ?? '')
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<[^>]+>/g, '')
}

function lineWidth(line, fontSize, bold) {
  return line.length * fontSize * AVG_GLYPH * (bold ? 1.06 : 1)
}

export function makeHeuristicTextMeasureProvider() {
  return {
    measureText(text, fontSize, _ff, fontWeight = 'normal', _fs, isHtml = false) {
      const t = isHtml ? stripTags(text) : String(text ?? '')
      const bold = fontWeight === 'bold' || Number(fontWeight) >= 600
      const lines = t.split('\n')
      const width = Math.max(0, ...lines.map((l) => lineWidth(l, fontSize, bold)))
      return { width, height: lines.length * fontSize * 1.2 }
    },
    measureTextLayout(text, fontSize, _ff, fontWeight = 'normal', _fs, containerWidth, isHtml = false) {
      const t = isHtml ? stripTags(text) : String(text ?? '')
      const bold = fontWeight === 'bold' || Number(fontWeight) >= 600
      const lines = t.split('\n')
      let width = Math.max(0, ...lines.map((l) => lineWidth(l, fontSize, bold)))
      const lineHeight = Math.round(fontSize * 1.2)
      let lineCount = lines.length
      if (containerWidth && width > containerWidth) {
        lineCount = Math.max(lineCount, Math.ceil(width / containerWidth))
      }
      if (containerWidth) width = Math.min(width, containerWidth)
      return { width, height: lineCount * lineHeight, lineCount, lineHeight }
    },
  }
}
