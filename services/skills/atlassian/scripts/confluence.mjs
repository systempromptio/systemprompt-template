#!/usr/bin/env node
import { readFileSync, writeFileSync, statSync, existsSync, mkdtempSync, rmSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { tmpdir } from 'node:os'
import { api, baseUrl, die } from './lib/atlassian/auth.mjs'
import { buildAdfFromText } from './lib/atlassian/adf.mjs'
import { buildPageUrl, getSpaceKeyForPage } from './lib/atlassian/url.mjs'
import {
  listAttachments,
  uploadAttachment,
  uploadOrUpdateAttachment,
  downloadAttachment,
} from './lib/atlassian/attachments.mjs'
import { makeAccountNameResolver } from './lib/atlassian/users.mjs'
import { collectDrawioAttachments } from './lib/diagrams/pull.mjs'
import { reconstructDiagramsInDoc } from './lib/diagrams/reconstruct.mjs'
import { REVERSE_CLI } from './lib/diagrams/reverse-cli.mjs'
import { refreshDocMatrix } from './lib/doc/matrix.mjs'
import { stampDocMeta } from './lib/doc/meta.mjs'
import { resolveProfile } from './lib/doc/profiles.mjs'
import { pullTypedMarkdown } from './lib/doc/pull.mjs'
import { getDocType, listDocTypes } from './lib/doc/types/index.mjs'
import { parseArgs } from './lib/util/cli-args.mjs'
import { tryReadFile } from './lib/util/node-io.mjs'

const SPACE_KEY = process.env.CONFLUENCE_SPACE_KEY
if (!SPACE_KEY) {
  console.error('ERROR: CONFLUENCE_SPACE_KEY must be set in .cursor/.project/.env')
  process.exit(1)
}

const COMMANDS = {
  'get-page': {
    usage: '<page_id> [storage|adf] [--type <fsd|isd>] [--body-only] [--into <path>] [--assets-dir <dir>]',
    desc: 'Read page content. Default returns raw storage XHTML; "adf" returns atlas_doc_format JSON. --type=<fsd|isd> instead returns the page reversed to canonical authored markdown (the typed reverse pull — strips the wiki chrome; use for a clean baseline/re-sync). --into <path> writes the body to a working copy and stamps its Confluence URL + Page ID header. --assets-dir <dir> (typed only) downloads body images + diagram sources into <dir> and KEEPS them (localize into the working copy ./assets on discovery); omit it and assets go to a throwaway temp dir (baseline/re-sync).',
    async run(pageId, ...rest) {
      if (!pageId) die('page_id required')

      // format is the first bare (non-flag) token; flags: --type, --body-only,
      // --into <path>. "storage" fetches raw storage XHTML, "adf" atlas_doc_format;
      // --type routes to the typed reverse pull (canonical markdown) instead.
      const { flags, positional } = parseArgs(rest, { booleans: ['body-only'] })
      const format = positional[0] || 'storage'
      const bodyOnly = Boolean(flags['body-only'])
      const intoPath = typeof flags.into === 'string' ? flags.into : null
      const assetsDirFlag = typeof flags['assets-dir'] === 'string' ? flags['assets-dir'] : null
      const type = typeof flags.type === 'string' ? flags.type.toLowerCase() : ''
      if (type && !listDocTypes().includes(type)) {
        die(`Unknown --type "${type}". Known types: ${listDocTypes().join(', ')}.`)
      }

      // ── Typed reverse pull: STORAGE → canonical authored markdown (FSD/ISD),
      // the inverse of a typed publish. Body images + diagram sources download
      // into --assets-dir when given (KEPT — localize into the working copy's
      // ./assets at discovery) or a throwaway temp dir otherwise (baseline/re-sync
      // never clobbers ./assets). Either way the markdown links `./assets/...` so
      // it diffs cleanly.
      if (type) {
        const data = await api(`pages/${pageId}?body-format=storage`)
        const storageXhtml = data?.body?.storage?.value
        if (typeof storageXhtml !== 'string') die(`No storage body for page ${pageId}`)
        const url = buildPageUrl(SPACE_KEY, data.id)
        const attachments = await listAttachments(pageId)
        const localizeAssets = Boolean(assetsDirFlag)
        const assetsDir = localizeAssets ? resolve(assetsDirFlag) : mkdtempSync(join(tmpdir(), 'conf-pull-'))
        try {
          const { markdown } = await pullTypedMarkdown({
            type,
            pageId: data.id,
            storageXhtml,
            title: data.title,
            imageRelPrefix: './assets',
            assetsDir,
            attachments,
            downloadAttachment,
            resolveAccountId: makeAccountNameResolver(api),
          })
          const { md } = await reconstructDiagramsInDoc({
            md: markdown,
            pageId: data.id,
            attachments,
            assetsDir,
            imageRelPrefix: './assets',
            downloadAttachment,
            reverseCli: REVERSE_CLI,
          })
          const out = md.endsWith('\n') ? md : `${md}\n`
          if (intoPath) {
            writeFileSync(intoPath, out, 'utf8')
            const stamp = stampDocMeta(intoPath, { url, pageId: data.id })
            console.log(`Wrote page ${data.id} as ${type.toUpperCase()} markdown -> ${intoPath}`)
            if (stamp.changed) console.log(`Stamped Confluence URL + Page ID into ${intoPath}`)
            if (localizeAssets) console.log(`Localized assets into ${assetsDir}`)
          } else {
            console.log(out)
          }
        } finally {
          // Only the throwaway temp dir is removed; a caller-provided --assets-dir
          // is the working copy's ./assets (or a submit staging dir) and is kept.
          if (!localizeAssets) rmSync(assetsDir, { recursive: true, force: true })
        }
        return
      }

      const bodyFormat = format === 'adf' ? 'atlas_doc_format' : 'storage'
      const data = await api(`pages/${pageId}?body-format=${bodyFormat}`)
      const body = format === 'adf' ? data.body.atlas_doc_format.value : data.body.storage.value
      const url = buildPageUrl(SPACE_KEY, data.id)

      if (intoPath) {
        writeFileSync(intoPath, body.endsWith('\n') ? body : `${body}\n`, 'utf8')
        // Same script-owned stamp as first publish, so pull and publish agree.
        const stamp = stampDocMeta(intoPath, { url, pageId: data.id })
        console.log(`Wrote page ${data.id} body -> ${intoPath}`)
        if (stamp.changed) console.log(`Stamped Confluence URL + Page ID into ${intoPath}`)
        return
      }

      if (bodyOnly) {
        console.log(body)
      } else {
        console.log(`Page ID: ${data.id}`)
        console.log(`Title: ${data.title}`)
        console.log(`Space ID: ${data.spaceId}`)
        console.log(`Parent ID: ${data.parentId || 'none'}`)
        console.log(`Status: ${data.status}`)
        console.log(`Version: ${data.version.number}`)
        console.log(`Created: ${data.createdAt}`)
        console.log(`Updated: ${data.version.createdAt}`)
        console.log(`URL: ${url}`)
        console.log('---')
        console.log(body)
      }
    },
  },

  'pull-diagrams': {
    usage: '<page_id> --into <doc.md>',
    desc: 'Reverse pull: rebuild ```drawio:<type>:<id> blocks + ./assets from a page\'s .drawio attachments (remote wins; existing block replaced in place).',
    async run(pageId, ...rest) {
      if (!pageId) die('page_id required')
      const { flags } = parseArgs(rest, {})
      const intoPath = typeof flags.into === 'string' ? flags.into : null
      if (!intoPath) die('--into <doc.md> required')

      const attachments = await listAttachments(pageId)
      if (!collectDrawioAttachments(attachments).length) {
        console.log(`No .drawio attachments on page ${pageId} — nothing to pull`)
        return
      }

      const docDir = dirname(resolve(intoPath))
      const assetsDir = join(docDir, 'assets')
      const md0 = existsSync(intoPath) ? readFileSync(intoPath, 'utf8') : ''

      const { md, pulled } = await reconstructDiagramsInDoc({
        md: md0,
        pageId,
        attachments,
        assetsDir,
        imageRelPrefix: './assets',
        downloadAttachment,
        reverseCli: REVERSE_CLI,
      })

      writeFileSync(intoPath, md.endsWith('\n') ? md : `${md}\n`, 'utf8')
      const url = buildPageUrl(SPACE_KEY, pageId)
      stampDocMeta(intoPath, { url, pageId })
      console.log(`Pulled ${pulled.length} diagram(s) into ${intoPath}: ${pulled.join(', ')}`)
    },
  },

  'create-page': {
    usage: '<space_id> <title> <body_or_file> [parent_id] [storage|adf]',
    desc: 'Create new page. storage = XHTML, adf = ADF JSON or plain text (auto-converted)',
    async run(spaceId, title, bodyInput, parentId, format = 'storage') {
      if (!spaceId || !title || !bodyInput) die('space_id, title, body_or_file required')
      const bodyContent = tryReadFile(bodyInput)
      const { representation, value } = resolveBody(bodyContent, format)

      const payload = {
        spaceId,
        status: 'current',
        title,
        body: { representation, value },
      }
      if (parentId) payload.parentId = parentId

      const data = await api('pages', { method: 'POST', body: payload })
      console.log('SUCCESS')
      console.log(`Page ID: ${data.id}`)
      console.log(`Title: ${data.title}`)
      console.log(`URL: ${buildPageUrl(SPACE_KEY, data.id)}`)
    },
  },

  'update-page': {
    usage: '<page_id> <body_or_file> [storage|adf] [version_message]',
    desc: 'Update page (replaces ALL body). Auto-increments version. storage = XHTML, adf = ADF JSON or plain text',
    async run(pageId, bodyInput, format = 'storage', versionMsg = 'Updated via script') {
      if (!pageId || !bodyInput) die('page_id, body_or_file required')
      const bodyContent = tryReadFile(bodyInput)
      const { representation, value } = resolveBody(bodyContent, format)

      const current = await api(`pages/${pageId}`)
      const newVersion = current.version.number + 1

      console.log(`Page: ${current.title}`)
      console.log(`Version: ${current.version.number} → ${newVersion}`)

      const payload = {
        id: pageId,
        status: current.status,
        title: current.title,
        body: { representation, value },
        version: { number: newVersion, message: versionMsg },
      }

      await api(`pages/${pageId}`, { method: 'PUT', body: payload })
      console.log('SUCCESS')
      console.log(`URL: ${buildPageUrl(SPACE_KEY, pageId)}`)
    },
  },

  'delete-page': {
    usage: '<page_id>',
    desc: 'Delete page',
    async run(pageId) {
      if (!pageId) die('page_id required')
      await api(`pages/${pageId}`, { method: 'DELETE' })
      console.log(`SUCCESS — page ${pageId} deleted`)
    },
  },

  'copy-page': {
    usage: '<source_page_id> <parent_page_id> [new_title]',
    desc: 'Copy page (API v1, preserves everything)',
    async run(sourceId, parentId, newTitle) {
      if (!sourceId || !parentId) die('source_page_id, parent_page_id required')

      const payload = {
        copyAttachments: true,
        copyPermissions: true,
        copyProperties: true,
        copyLabels: true,
        destination: { type: 'parent_page', value: parentId },
      }
      if (newTitle) payload.pageTitle = newTitle

      console.log(`Copying page ${sourceId} to parent ${parentId}...`)
      const data = await api(`content/${sourceId}/copy`, { method: 'POST', body: payload, version: 'v1' })
      console.log('SUCCESS')
      console.log(`Page ID: ${data.id || 'pending'}`)
      console.log(`Title: ${data.title || data.pageTitle || 'N/A'}`)
      if (data.id) console.log(`URL: ${buildPageUrl(SPACE_KEY, data.id)}`)
    },
  },

  'search': {
    usage: '<cql_query> [limit]',
    desc: 'Search with CQL',
    async run(cql, limit = '25') {
      if (!cql) die('cql_query required')
      const encoded = encodeURIComponent(cql)
      const data = await api(`content/search?cql=${encoded}&limit=${limit}&expand=version,space`, { version: 'v1' })

      console.log(`Results: ${data.size} of ${data.totalSize || '?'}`)
      console.log('---')
      for (const r of data.results || []) {
        console.log(`ID: ${r.id}`)
        console.log(`Title: ${r.title}`)
        console.log(`Type: ${r.type}`)
        console.log(`Space: ${r.space?.key || 'N/A'}`)
        console.log(`Version: ${r.version?.number || 'N/A'}`)
        console.log(`URL: ${baseUrl}/wiki${r._links.webui}`)
        console.log('---')
      }
    },
  },

  'list-spaces': {
    usage: '[limit] [global|personal]',
    desc: 'List spaces',
    async run(limit = '25', type) {
      let url = `spaces?limit=${limit}`
      if (type) url += `&type=${type}`
      const data = await api(url)

      console.log(`Spaces: ${data.results.length}`)
      console.log('---')
      for (const s of data.results) {
        console.log(`ID: ${s.id}`)
        console.log(`Key: ${s.key}`)
        console.log(`Name: ${s.name}`)
        console.log(`Type: ${s.type}`)
        console.log(`Status: ${s.status}`)
        console.log('---')
      }
    },
  },

  'list-pages': {
    usage: '<space_id> [limit] [sort] [title_filter]',
    desc: 'List pages in space',
    async run(spaceId, limit = '25', sort, title) {
      if (!spaceId) die('space_id required')
      let url = `spaces/${spaceId}/pages?limit=${limit}&status=current`
      if (sort) url += `&sort=${sort}`
      if (title) url += `&title=${encodeURIComponent(title)}`
      const data = await api(url)

      console.log(`Pages: ${data.results.length}`)
      console.log('---')
      for (const p of data.results) {
        console.log(`ID: ${p.id}`)
        console.log(`Title: ${p.title}`)
        console.log(`Status: ${p.status}`)
        console.log(`Parent ID: ${p.parentId || 'none'}`)
        console.log(`Version: ${p.version?.number || 'N/A'}`)
        console.log('---')
      }
    },
  },

  'list-children': {
    usage: '<page_id> [limit] [depth]',
    desc: 'List child pages',
    async run(pageId, limit = '25', depth) {
      if (!pageId) die('page_id required')
      let url = `pages/${pageId}/children?limit=${limit}`
      if (depth) url += `&depth=${depth}`
      const data = await api(url)

      console.log(`Children: ${data.results.length}`)
      console.log('---')
      for (const p of data.results) {
        console.log(`ID: ${p.id}`)
        console.log(`Title: ${p.title}`)
        console.log(`Status: ${p.status}`)
        console.log('---')
      }
    },
  },

  'doc-matrix': {
    usage: '--type <fsd|isd> [--page-id <id>] [--first-column <title>] [--dry]',
    desc: 'Manual/dry-run entry point for the approval matrix — a typed publish already refreshes it automatically, so reach for this only to preview the columns or repair a parent page. Scans the typed children for the keys their header tables carry (the approver roles are whatever the documents define — no role list is hard-coded) and upserts one Page Properties Report scoped by label + ancestor. Columns run status, author, approvers, package; the other card fields (WBS, project name, Jira reference) are left out. An existing report with the same id is replaced in place; the rest of the page is untouched. Parent defaults to the type\'s CONFLUENCE_<TYPE>_PARENT_ID. --dry prints the columns and the macro without writing.',
    async run(...rest) {
      const { flags } = parseArgs(rest, { booleans: ['dry'] })
      const type = typeof flags.type === 'string' ? flags.type.toLowerCase() : ''
      if (!type) die(`--type required. Known types: ${listDocTypes().join(', ')}.`)
      if (!listDocTypes().includes(type)) {
        die(`Unknown --type "${type}". Known types: ${listDocTypes().join(', ')}.`)
      }

      const { propertiesId } = getDocType(type)
      const { parentEnv } = resolveProfile(type)
      const parentId = typeof flags['page-id'] === 'string' ? flags['page-id'] : process.env[parentEnv]
      if (!parentId) die(`--page-id required, or set ${parentEnv} in .cursor/.project/.env`)
      const firstColumn = typeof flags['first-column'] === 'string' ? flags['first-column'] : 'Document'

      const r = await refreshDocMatrix({
        api,
        type,
        parentId,
        propertiesId,
        firstColumn,
        dry: Boolean(flags.dry),
      })

      for (const c of r.contributors) console.log(`  ${c.id}  ${c.title} — ${c.keys} key(s)`)
      console.log(`Children scanned: ${r.scanned}, contributing: ${r.contributors.length}`)
      for (const p of r.unlabelled) {
        console.log(`  SKIPPED (no "${type}" label, so the report cannot show it): ${p.id} ${p.title}`)
      }
      if (!r.columns.length) {
        // Not an error: a parent whose children predate the Content Properties
        // wrapper simply has nothing to aggregate until they are republished.
        console.log(`No "${propertiesId}" Content Properties found on any child — republish the ${type.toUpperCase()} pages first.`)
        return
      }
      console.log(`Columns (${r.columns.length}): ${firstColumn} | ${r.columns.join(' | ')}`)
      if (r.dropped.length) console.log(`Card fields left out of the matrix: ${r.dropped.join(', ')}`)

      if (flags.dry) {
        console.log('--- macro (dry run, nothing written) ---')
        console.log(r.macro)
        return
      }

      console.log(r.action === 'unchanged' ? 'Matrix already up to date' : `SUCCESS — matrix ${r.action}`)
      console.log(`URL: ${buildPageUrl(SPACE_KEY, parentId)}`)
    },
  },

  'comments': {
    usage: '<page_id> [footer|inline] [limit]',
    desc: 'Read comments with threaded replies',
    async run(pageId, type = 'footer', limit = '25') {
      if (!pageId) die('page_id required')
      const endpoint = type === 'inline' ? 'inline-comments' : 'footer-comments'
      const data = await api(`pages/${pageId}/${endpoint}?limit=${limit}&body-format=storage`)

      console.log(`Comments (${type}): ${data.results.length}`)
      console.log('---')
      for (const c of data.results) {
        console.log(`ID: ${c.id}`)
        console.log(`Status: ${c.status}`)
        console.log(`Created: ${c.createdAt}`)
        if (c.body?.storage) console.log(`Body: ${c.body.storage.value}`)

        try {
          const children = await api(`${endpoint}/${c.id}/children?body-format=storage`)
          if (children.results?.length) {
            console.log(`Replies: ${children.results.length}`)
            for (const r of children.results) {
              console.log(`  Reply ID: ${r.id}`)
              console.log(`  Author: ${r.version?.authorId || 'unknown'}`)
              console.log(`  Created: ${r.version?.createdAt || r.createdAt}`)
              if (r.body?.storage) console.log(`  Body: ${r.body.storage.value}`)
              console.log(`  ---`)
            }
          }
        } catch { /* no children endpoint for this comment type */ }

        console.log('---')
      }
    },
  },

  'add-comment': {
    usage: '<page_id> <body_or_file> [footer|inline] [parent_comment_id]',
    desc: 'Add comment or reply. Accepts plain text (with #/##/### headings, - bullets) or ADF JSON. Top-level inline: set INLINE_TEXT_SELECTION, INLINE_MATCH_COUNT, INLINE_MATCH_INDEX env vars. Reply: pass parent_comment_id (selection env vars are ignored — replies inherit the parent anchor)',
    async run(pageId, bodyInput, type = 'footer', parentId) {
      if (!pageId || !bodyInput) die('page_id, body_or_file required')

      const bodyContent = tryReadFile(bodyInput)
      let adfBody
      try {
        adfBody = JSON.parse(bodyContent)
      } catch {
        adfBody = buildAdfFromText(bodyContent)
      }

      const payload = {
        body: { representation: 'atlas_doc_format', value: JSON.stringify(adfBody) },
      }

      if (parentId) {
        // Reply: only parentCommentId + body. pageId / inlineCommentProperties
        // are for top-level comments and cause HTTP 400 on a reply.
        payload.parentCommentId = parentId
      } else {
        payload.pageId = pageId
        if (type === 'inline') {
          payload.inlineCommentProperties = {
            textSelection: process.env.INLINE_TEXT_SELECTION || '',
            textSelectionMatchCount: parseInt(process.env.INLINE_MATCH_COUNT || '1'),
            textSelectionMatchIndex: parseInt(process.env.INLINE_MATCH_INDEX || '0'),
          }
        }
      }

      const endpoint = type === 'inline' ? 'inline-comments' : 'footer-comments'
      const data = await api(endpoint, { method: 'POST', body: payload })
      console.log('SUCCESS')
      console.log(`Comment ID: ${data.id}`)
      console.log(`URL: ${buildPageUrl(SPACE_KEY, pageId)}?focusedCommentId=${data.id}`)
    },
  },

  'delete-comment': {
    usage: '<comment_id> [footer|inline]',
    desc: 'Delete a comment',
    async run(commentId, type = 'footer') {
      if (!commentId) die('comment_id required')
      const endpoint = type === 'inline' ? 'inline-comments' : 'footer-comments'
      await api(`${endpoint}/${commentId}`, { method: 'DELETE' })
      console.log(`SUCCESS — comment ${commentId} deleted`)
    },
  },

  'list-attachments': {
    usage: '<page_id>',
    desc: 'List attachments on a page',
    async run(pageId) {
      if (!pageId) die('page_id required')
      const attachments = await listAttachments(pageId)
      console.log(`Attachments: ${attachments.length}`)
      console.log('---')
      for (const att of attachments) {
        console.log(`ID: ${att.id}`)
        console.log(`Title: ${att.title}`)
        console.log(`Media type: ${att.mediaType}`)
        console.log(`Size: ${att.fileSize} bytes`)
        console.log(`Version: ${att.version?.number || 'N/A'}`)
        console.log('---')
      }
    },
  },

  'upload-attachment': {
    usage: '<page_id> <file_path> [comment]',
    desc: 'Upload file as page attachment (fails if same filename exists; use upload-attachment-update)',
    async run(pageId, filePath, comment) {
      if (!pageId || !filePath) die('page_id, file_path required')
      const spaceKey = await getSpaceKeyForPage(pageId)
      const att = await uploadAttachment(pageId, filePath, comment || 'Attachment uploaded via script')
      console.log('SUCCESS')
      console.log(`Attachment ID: ${att.id}`)
      console.log(`Title: ${att.title}`)
      console.log(`URL: ${buildPageUrl(spaceKey, pageId)}`)
    },
  },

  'upload-attachment-update': {
    usage: '<page_id> <file_path> [comment]',
    desc: 'Upload file or update existing attachment with the same filename',
    async run(pageId, filePath, comment) {
      if (!pageId || !filePath) die('page_id, file_path required')
      const spaceKey = await getSpaceKeyForPage(pageId)
      const att = await uploadOrUpdateAttachment(pageId, filePath, comment || 'Attachment updated via script')
      console.log('SUCCESS')
      console.log(`Attachment ID: ${att.id}`)
      console.log(`Title: ${att.title}`)
      console.log(`Action: ${att.created ? 'created' : 'updated'}`)
      console.log(`URL: ${buildPageUrl(spaceKey, pageId)}`)
    },
  },

  'modify-table': {
    usage: '<input.json> <output.json> <heading_text> <column_name> <notes_json>',
    desc: 'Add column to ADF table found after a heading. notes_json = JSON array of strings per row',
    async run(inputFile, outputFile, headingText, columnName, notesJson) {
      if (!inputFile || !outputFile || !headingText || !columnName || !notesJson) {
        die('input.json, output.json, heading_text, column_name, notes_json required')
      }

      const data = JSON.parse(readFileSync(inputFile, 'utf8'))
      const notes = JSON.parse(notesJson)

      const makeTextCell = (text) => ({
        type: 'tableCell',
        attrs: { colspan: 1, rowspan: 1 },
        content: [{ type: 'paragraph', content: text ? [{ text, type: 'text' }] : [] }],
      })

      const makeHeaderCell = (text) => ({
        type: 'tableHeader',
        attrs: { colspan: 1, rowspan: 1 },
        content: [{ type: 'paragraph', content: [{ text, type: 'text', marks: [{ type: 'strong' }] }] }],
      })

      let found = false

      const walk = (node) => {
        if (!node?.content) return
        for (let i = 0; i < node.content.length; i++) {
          const child = node.content[i]
          if (child.type === 'heading' && child.content) {
            const text = child.content.map((c) => c.text || '').join('')
            if (text.includes(headingText)) {
              for (let j = i + 1; j < node.content.length; j++) {
                if (node.content[j].type === 'table') {
                  const table = node.content[j]
                  table.content.forEach((row, idx) => {
                    if (idx === 0) {
                      row.content.push(makeHeaderCell(columnName))
                    } else {
                      row.content.push(makeTextCell(notes[idx - 1] || ''))
                    }
                  })
                  found = true
                  console.log(`Added "${columnName}" column to table after "${headingText}" heading`)
                  console.log(`Rows modified: ${table.content.length}`)
                  return
                }
              }
            }
          }
          walk(child)
        }
      }

      walk(data)

      if (!found) die(`Table not found after heading containing "${headingText}"`)

      writeFileSync(outputFile, JSON.stringify(data))
      console.log(`Output: ${outputFile} (${statSync(outputFile).size} bytes)`)
    },
  },
}

function resolveBody(content, format) {
  if (format !== 'adf') return { representation: 'storage', value: content }
  try {
    const parsed = JSON.parse(content)
    return { representation: 'atlas_doc_format', value: JSON.stringify(parsed) }
  } catch {
    return { representation: 'atlas_doc_format', value: JSON.stringify(buildAdfFromText(content)) }
  }
}

const [command, ...args] = process.argv.slice(2)

const wantsHelp =
  !command ||
  command === 'help' ||
  command === '--help' ||
  command === '-h' ||
  !COMMANDS[command]

if (wantsHelp) {
  console.log('Usage: node confluence.mjs <command> [args]\n')
  console.log('Commands:')
  for (const [name, { usage, desc }] of Object.entries(COMMANDS)) {
    console.log(`  ${name} ${usage}`)
    console.log(`    ${desc}\n`)
  }
  process.exit(command && command !== 'help' && command !== '--help' && command !== '-h' ? 1 : 0)
}

if (args.includes('--help') || args.includes('-h')) {
  const { usage, desc } = COMMANDS[command]
  console.log(`Usage: node confluence.mjs ${command} ${usage}`)
  console.log(`  ${desc}\n`)
  process.exit(0)
}

try {
  await COMMANDS[command].run(...args)
} catch (err) {
  console.error(`ERROR: ${err.message}`)
  process.exit(1)
}
