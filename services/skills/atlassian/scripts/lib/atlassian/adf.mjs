function parseTableRow(line) {
  const trimmed = line.trim()
  if (!trimmed.startsWith('|')) return null
  const inner = trimmed.replace(/^\|/, '').replace(/\|$/, '')
  return inner.split('|').map((cell) => cell.trim())
}

function isTableSeparatorRow(cells) {
  return cells.length > 0 && cells.every((cell) => /^:?-{3,}:?$/.test(cell))
}

function buildTableCell(cellType, cellText) {
  const inline = parseInlineFormatting(cellText)
  if (cellType === 'tableHeader') {
    inline.forEach((node) => {
      if (node.type === 'text') {
        node.marks = node.marks || []
        if (!node.marks.some((mark) => mark.type === 'strong')) {
          node.marks.push({ type: 'strong' })
        }
      }
    })
  }
  return {
    type: cellType,
    attrs: { colspan: 1, rowspan: 1 },
    content: [{ type: 'paragraph', content: inline }],
  }
}

function buildTableFromRows(dataRows, hasHeader) {
  const tableRows = dataRows.map((cells, rowIndex) => {
    const isHeaderRow = hasHeader && rowIndex === 0
    return {
      type: 'tableRow',
      content: cells.map((cellText) =>
        buildTableCell(isHeaderRow ? 'tableHeader' : 'tableCell', cellText)
      ),
    }
  })
  return {
    type: 'table',
    attrs: { isNumberColumnEnabled: false, layout: 'default' },
    content: tableRows,
  }
}

export function buildAdfFromText(rawText) {
  const lines = rawText.split(/\r?\n/)
  const content = []

  let idx = 0
  while (idx < lines.length) {
    const line = lines[idx]
    const trimmed = line.trim()
    if (!line && idx < lines.length - 1) {
      idx += 1
      continue
    }

    const h1 = trimmed.startsWith('# ') && !trimmed.startsWith('## ')
    const h2 = trimmed.startsWith('## ') && !trimmed.startsWith('### ')
    const h3 = trimmed.startsWith('### ')
    if (trimmed.startsWith('```')) {
      const language = trimmed.slice(3).trim()
      const codeLines = []
      idx += 1

      while (idx < lines.length && !lines[idx].trim().match(/^```\s*$/)) {
        codeLines.push(lines[idx])
        idx += 1
      }

      if (idx < lines.length) idx += 1

      content.push({
        type: 'codeBlock',
        attrs: language ? { language } : {},
        content: [{ type: 'text', text: codeLines.join('\n') }],
      })
      continue
    }

    if (h1) {
      content.push({
        type: 'heading',
        attrs: { level: 1 },
        content: parseInlineFormatting(trimmed.slice(2).trim()),
      })
      idx += 1
      continue
    }
    if (h2) {
      content.push({
        type: 'heading',
        attrs: { level: 2 },
        content: parseInlineFormatting(trimmed.slice(3).trim()),
      })
      idx += 1
      continue
    }
    if (h3) {
      content.push({
        type: 'heading',
        attrs: { level: 3 },
        content: parseInlineFormatting(trimmed.slice(4).trim()),
      })
      idx += 1
      continue
    }

    if (trimmed.startsWith('- ')) {
      const items = []
      while (idx < lines.length && lines[idx] && lines[idx].trimStart().startsWith('- ')) {
        const itemLine = lines[idx].trimStart().slice(2).trim()
        items.push({
          type: 'listItem',
          content: [{ type: 'paragraph', content: parseInlineFormatting(itemLine) }],
        })
        idx += 1
      }
      content.push({ type: 'bulletList', content: items })
      continue
    }

    const tableCells = parseTableRow(line)
    if (tableCells) {
      const allRows = []
      while (idx < lines.length) {
        const cells = parseTableRow(lines[idx])
        if (!cells) break
        allRows.push(cells)
        idx += 1
      }

      let hasHeader = false
      let dataRows = allRows
      if (allRows.length >= 2 && isTableSeparatorRow(allRows[1])) {
        hasHeader = true
        dataRows = [allRows[0], ...allRows.slice(2)]
      }

      if (dataRows.length > 0) {
        content.push(buildTableFromRows(dataRows, hasHeader))
      }
      continue
    }

    const olFirstMatch = trimmed.match(/^\d+\.\s(.*)/)
    if (olFirstMatch) {
      const items = []
      // Consume first item from current line, then consecutive numbered lines (invariant: idx points at the line we're parsing)
      let itemLine = olFirstMatch[1].trim()
      items.push({
        type: 'listItem',
        content: [{ type: 'paragraph', content: parseInlineFormatting(itemLine) }],
      })
      idx += 1
      while (idx < lines.length && lines[idx]) {
        const match = lines[idx].trimStart().match(/^\d+\.\s(.*)/)
        if (!match) break
        itemLine = match[1].trim()
        items.push({
          type: 'listItem',
          content: [{ type: 'paragraph', content: parseInlineFormatting(itemLine) }],
        })
        idx += 1
      }
      content.push({ type: 'orderedList', content: items })
      continue
    }

    if (trimmed) {
      content.push({ type: 'paragraph', content: parseInlineFormatting(trimmed) })
    }
    idx += 1
  }

  if (content.length === 0) {
    return { type: 'doc', version: 1, content: [{ type: 'paragraph', content: [{ type: 'text', text: '' }] }] }
  }

  return { type: 'doc', version: 1, content }
}

// Inline token types: color ({color:red}text{color}), bold (**text** or *text*), code ({{text}} or `text`),
// wiki links ([title|url]), markdown links ([title](url)), bare URLs
const INLINE_RE = new RegExp(
  '\\{color:([^}]+)\\}(.*?)\\{color\\}'      + '|' + // {color:red}text{color}
  '\\*\\*(.+?)\\*\\*'                       + '|' + // **bold**
  '(?<!\\*)\\*([^*]+?)\\*(?!\\*)'            + '|' + // *bold* (not inside **)
  '\\{\\{(.+?)\\}\\}'                        + '|' + // {{code}}
  '`([^`]+)`'                                + '|' + // `code`
  '\\[([^\\]]+?)\\|([^\\]]+?)\\]'            + '|' + // [title|url] wiki link
  '\\[([^\\]]+?)\\]\\(([^)]+?)\\)'           + '|' + // [title](url) md link
  '(https?://\\S+)',                                  // bare URL
  'g'
)

// Internal to this module (buildAdfFromText). Not part of the public API.
function parseInlineFormatting(text) {
  const nodes = []
  let lastIndex = 0

  for (const m of text.matchAll(INLINE_RE)) {
    if (m.index > lastIndex) {
      nodes.push({ type: 'text', text: text.slice(lastIndex, m.index) })
    }

    const [, colorVal, colorText, doubleBold, singleBold, wikiCode, backtickCode, wikiTitle, wikiUrl, mdTitle, mdUrl, bareUrl] = m

    if (colorVal !== undefined) {
      const innerNodes = parseInlineFormatting(colorText);
      const normalizedColor = colorVal.toLowerCase();
      const mappedColor = normalizedColor === 'red' ? '#ff5630' : normalizedColor === 'green' ? '#36b37e' : normalizedColor;
      innerNodes.forEach(n => {
        n.marks = n.marks || [];
        // Only add color mark if one doesn't exist (innermost color wins)
        if (!n.marks.some(mark => mark.type === 'textColor')) {
          n.marks.push({ type: 'textColor', attrs: { color: mappedColor } });
        }
        nodes.push(n);
      });
    } else if (doubleBold !== undefined || singleBold !== undefined) {
      const innerNodes = parseInlineFormatting(doubleBold || singleBold);
      innerNodes.forEach(n => {
        n.marks = n.marks || [];
        if (!n.marks.some(mark => mark.type === 'strong')) {
          n.marks.push({ type: 'strong' });
        }
        nodes.push(n);
      });
    } else if (wikiCode !== undefined || backtickCode !== undefined) {
      nodes.push({ type: 'text', text: wikiCode || backtickCode, marks: [{ type: 'code' }] })
    } else if (wikiTitle !== undefined) {
      nodes.push({ type: 'text', text: wikiTitle, marks: [{ type: 'link', attrs: { href: wikiUrl } }] })
    } else if (mdTitle !== undefined) {
      nodes.push({ type: 'text', text: mdTitle, marks: [{ type: 'link', attrs: { href: mdUrl } }] })
    } else if (bareUrl !== undefined) {
      nodes.push({ type: 'text', text: bareUrl, marks: [{ type: 'link', attrs: { href: bareUrl } }] })
    }

    lastIndex = m.index + m[0].length
  }

  if (lastIndex < text.length) {
    nodes.push({ type: 'text', text: text.slice(lastIndex) })
  }

  return nodes.length > 0 ? nodes : [{ type: 'text', text }]
}
