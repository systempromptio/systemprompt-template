#!/usr/bin/env node
import { mkdir, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises'
import { createHash } from 'node:crypto'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'
import { api, baseUrl, AUTH } from './lib/atlassian/auth.mjs'
import { parseArgs } from './lib/util/cli-args.mjs'
import { withConcurrency, fileExists, safeDecodeURIComponent, sanitizeFileName } from './lib/util/node-io.mjs'
import {
  makeTurndown,
  stripNonContentHtml,
  absolutizeRootRelativeUrls,
  parseExportHeader,
} from './lib/atlassian/html-to-markdown.mjs'
import { downloadAttachment } from './lib/atlassian/attachments.mjs'
import { makeAccountNameResolver } from './lib/atlassian/users.mjs'
import { collectDrawioAttachments } from './lib/diagrams/pull.mjs'
import { reconstructDiagramsInDoc } from './lib/diagrams/reconstruct.mjs'
import { REVERSE_CLI } from './lib/diagrams/reverse-cli.mjs'
import { pullTypedMarkdown } from './lib/doc/pull.mjs'
import { NotDocTypeError } from './lib/doc/storage-to-doc.mjs'
import { listDocTypes } from './lib/doc/types/index.mjs'

const require = createRequire(import.meta.url)
const domino = require('@mixmark-io/domino')

const SPACE_KEY = process.env.CONFLUENCE_SPACE_KEY || '' // used only for fallback web URLs
const DEFAULT_OUTPUT_DIR = '.project/project-context/inbox/confluence'
const BASE_HOSTNAME = new URL(baseUrl).hostname

// account-id → display name for the typed reverse (cached across pages).
const resolveAccountName = makeAccountNameResolver(api)

const API_CONCURRENCY = 8
const EXPORT_CONCURRENCY = 5
const ASSET_CONCURRENCY = 10

const HELP_TEXT = `Usage:
  node export-confluence-to-markdown.mjs [root_page_id] [output_dir] [--type <fsd|isd>] [--full] [--prune] [--rename]

Options:
  --index <page_id>   Root Confluence page id
  --out <dir>         Output directory (default: ${DEFAULT_OUTPUT_DIR})
  --type <fsd|isd>    Typed reverse pull: reconstruct each page as canonical
                      authored markdown from its STORAGE (strips wiki chrome).
                      Pages that don't parse as this type fall back to the
                      generic render dump. Omit for today's generic export.
  --full              Re-export ALL pages (ignores versions)
  --prune             Remove local pages not present under root page
  --rename            Rename local files when titles changed (default: keep existing)
  --help              Show this help
`

function slugify(input) {
  const normalized = input
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 80)

  return normalized.length > 0 ? normalized : 'page'
}

function buildAncestryPath(pageId, pageIndex, rootPageId) {
  const segments = []
  let currentId = pageId

  while (currentId && currentId !== rootPageId) {
    const entry = pageIndex.get(currentId)
    if (!entry || !entry.parentId) break
    currentId = entry.parentId
    if (currentId === rootPageId) break
    const parentEntry = pageIndex.get(currentId)
    if (parentEntry) segments.unshift(parentEntry.slug)
  }

  return segments
}

async function cleanEmptyDirs(dir) {
  let entries
  try {
    entries = await readdir(dir, { withFileTypes: true })
  } catch {
    return
  }

  for (const entry of entries) {
    if (entry.isDirectory()) {
      await cleanEmptyDirs(join(dir, entry.name))
    }
  }

  try {
    entries = await readdir(dir, { withFileTypes: true })
  } catch {
    return
  }

  if (entries.length === 0) {
    await rm(dir, { recursive: true, force: true }).catch(() => null)
  }
}

function sha1Short(input) {
  return createHash('sha1').update(input).digest('hex').slice(0, 8)
}

function splitExt(fileName) {
  const lastDot = fileName.lastIndexOf('.')
  if (lastDot <= 0 || lastDot === fileName.length - 1) {
    return { base: fileName, ext: '' }
  }
  return { base: fileName.slice(0, lastDot), ext: fileName.slice(lastDot) }
}

function shouldDownloadConfluenceAsset(urlObj) {
  const p = urlObj.pathname

  if (urlObj.hostname === BASE_HOSTNAME) {
    return (
      p.startsWith('/wiki/download/attachments/') ||
      p.startsWith('/wiki/download/resources/') ||
      p.startsWith('/wiki/images/') ||
      p.startsWith('/wiki/s/') ||
      p.includes('/_/images/')
    )
  }

  return false
}

function mapAssetUrl(urlObj) {
  if (
    urlObj.hostname === 'confluence.ontrq.com' &&
    urlObj.pathname.includes('/_/images/icons/emoticons/')
  ) {
    const fileName = urlObj.pathname.split('/').pop() || ''
    const baseName = fileName.includes('.') ? fileName.slice(0, fileName.lastIndexOf('.')) : fileName
    if (baseName.length > 0) {
      return new URL(`${baseUrl}/wiki/images/icons/emoticons/${baseName}.png`)
    }
  }

  return urlObj
}

// Confluence Cloud keeps attachment binaries in Media Services. The web-UI
// download servlet (/wiki/download/attachments/<pageId>/<file>) that export_view
// renders into img src does NOT accept API-token Basic auth — it 401s. The only
// auth-working path is the REST attachment-download endpoint, which 302-redirects
// to a Media URL carrying its own short-lived token. So for any attachment-servlet
// URL we resolve the owning page's attachments via the v2 API and swap in its
// downloadLink; downloadToFile then follows the redirect to Media (Authorization is
// sent only to this site's host, and the Media redirect carries its own token).
const ATTACHMENT_SERVLET_RE = /^\/wiki\/download\/attachments\/(\d+)\/([^/]+)$/
const attachmentCache = new Map() // attPageId -> Array<attachment> (full v2 objects)
const attachmentLinkCache = new Map() // attPageId -> Map<title, absoluteDownloadUrl>

// Full attachment list for a page (v2, paginated + cached). Shared by the
// servlet-URL resolver AND the diagram reconstruction, so a page is listed once.
async function loadPageAttachments(attPageId) {
  if (attachmentCache.has(attPageId)) return attachmentCache.get(attPageId)
  const all = []
  let path = `pages/${attPageId}/attachments?limit=250`
  try {
    while (path) {
      const data = await api(path)
      for (const a of data?.results || []) all.push(a)
      const next = data?._links?.next
      path = next ? next.replace(/^.*\/wiki\/api\/v2\//, '') : null
    }
  } catch {
    // Leave whatever was gathered; callers fall back gracefully on a miss.
  }
  attachmentCache.set(attPageId, all)
  return all
}

async function loadPageAttachmentLinks(attPageId) {
  if (attachmentLinkCache.has(attPageId)) return attachmentLinkCache.get(attPageId)
  const map = new Map()
  for (const a of await loadPageAttachments(attPageId)) {
    if (a?.title && a?.downloadLink && !map.has(a.title)) {
      map.set(a.title, a.downloadLink.startsWith('http') ? a.downloadLink : `${baseUrl}/wiki${a.downloadLink}`)
    }
  }
  attachmentLinkCache.set(attPageId, map)
  return map
}

// Map an attachment-servlet URL to its auth-working REST download URL; every other
// URL (static resources, emoticons, cross-host) passes through unchanged.
async function resolveDownloadableUrl(urlObj) {
  const m = ATTACHMENT_SERVLET_RE.exec(urlObj.pathname)
  if (!m) return urlObj
  const name = safeDecodeURIComponent(m[2])
  const links = await loadPageAttachmentLinks(m[1])
  const dl = links.get(name)
  return dl ? new URL(dl) : urlObj
}

async function downloadToFile(urlObj, destPath) {
  const headers = { Accept: '*/*' }
  if (urlObj.hostname === BASE_HOSTNAME) {
    headers.Authorization = AUTH
  }

  const res = await fetch(urlObj.toString(), { headers })

  if (!res.ok) {
    throw new Error(`HTTP ${res.status}: ${res.statusText}`)
  }

  const buffer = Buffer.from(await res.arrayBuffer())
  await writeFile(destPath, buffer)
}

async function localizeAssetsInHtml(
  html,
  { pageId, assetsRootDir, assetsRelativeFromPage, diagramPngTitles = new Set() },
) {
  const window = domino.createWindow(html)
  const { document } = window

  const pageAssetsDir = join(assetsRootDir, pageId)
  await mkdir(pageAssetsDir, { recursive: true })

  const urlToLocal = new Map()

  const appendStyle = (node, styleChunk) => {
    const existing = node.getAttribute('style') || ''
    const cleaned = existing.trim().replace(/;$/, '')
    const next = cleaned.length > 0 ? `${cleaned}; ${styleChunk}` : styleChunk
    node.setAttribute('style', next)
  }

  const allNodes = [
    ...Array.from(document.querySelectorAll('img[src]')).map((node) => ({ node, attr: 'src' })),
    ...Array.from(document.querySelectorAll('a[href]')).map((node) => ({ node, attr: 'href' })),
  ]

  // Collect unique download tasks to avoid fetching the same asset twice
  const pendingDownloads = new Map()
  const nodeUpdates = []

  for (const { node, attr } of allNodes) {
    const rawValue = node.getAttribute(attr)
    if (!rawValue || rawValue.startsWith('#') || rawValue.startsWith('data:')) continue

    let urlObj
    try {
      urlObj = new URL(rawValue, baseUrl)
    } catch {
      continue
    }

    urlObj = mapAssetUrl(urlObj)
    if (!shouldDownloadConfluenceAsset(urlObj)) continue

    const originalUrl = urlObj.toString()

    if (!pendingDownloads.has(originalUrl)) {
      const rawFileName = urlObj.pathname.split('/').pop() || 'asset'
      const sanitized = sanitizeFileName(rawFileName)
      let fileName
      if (diagramPngTitles.has(safeDecodeURIComponent(rawFileName))) {
        // A generated-diagram image: keep the clean slug (`<id>.png`) so it stays
        // paired with its `<id>.drawio` sibling and re-publish's hash-gate matches
        // the attachment title. No collision-avoidance suffix here.
        fileName = sanitized
      } else {
        const { base, ext } = splitExt(sanitized)
        fileName = `${base.slice(0, 120)}-${sha1Short(originalUrl)}${ext}`
      }
      const destPath = join(pageAssetsDir, fileName)
      pendingDownloads.set(originalUrl, { urlObj, destPath, fileName })
    }

    nodeUpdates.push({ node, attr, originalUrl })
  }

  // Download all unique assets in parallel
  await withConcurrency(
    Array.from(pendingDownloads.entries()).map(
      ([originalUrl, { urlObj, destPath, fileName }]) =>
        async () => {
          if (!(await fileExists(destPath))) {
            try {
              await downloadToFile(await resolveDownloadableUrl(urlObj), destPath)
            } catch (err) {
              process.stdout.write(
                `WARN: failed to download asset for ${pageId}: ${originalUrl} (${
                  err instanceof Error ? err.message : String(err)
                })\n`
              )
              return
            }
          }
          urlToLocal.set(originalUrl, `${assetsRelativeFromPage}/${pageId}/${fileName}`)
        }
    ),
    ASSET_CONCURRENCY
  )

  // Update DOM nodes with local paths
  for (const { node, attr, originalUrl } of nodeUpdates) {
    const localRelative = urlToLocal.get(originalUrl)
    if (localRelative) node.setAttribute(attr, localRelative)
  }

  for (const img of Array.from(document.querySelectorAll('img'))) {
    appendStyle(img, 'max-width: 100%; height: auto;')
  }

  for (const cell of Array.from(document.querySelectorAll('th,td'))) {
    appendStyle(cell, 'vertical-align: top;')
  }

  for (const wrapper of Array.from(document.querySelectorAll('span.confluence-embedded-file-wrapper'))) {
    const parent = wrapper.parentNode
    if (!parent) continue
    while (wrapper.firstChild) parent.insertBefore(wrapper.firstChild, wrapper)
    parent.removeChild(wrapper)
  }

  return document.body.innerHTML
}

async function readExistingExports(pagesDir) {
  const map = new Map()

  async function scanDir(dir) {
    let entries
    try {
      entries = await readdir(dir, { withFileTypes: true })
    } catch {
      return
    }

    await Promise.all(
      entries.map(async (entry) => {
        const fullPath = join(dir, entry.name)

        if (entry.isDirectory()) {
          await scanDir(fullPath)
          return
        }

        if (!entry.isFile() || !entry.name.endsWith('.md')) return

        let content
        try {
          content = await readFile(fullPath, 'utf8')
        } catch {
          return
        }

        const header = parseExportHeader(content)
        if (!header.pageId) return

        map.set(header.pageId, {
          fileName: entry.name,
          filePath: fullPath,
          version: header.version,
          title: header.title,
        })
      })
    )
  }

  await scanDir(pagesDir)
  return map
}

async function fetchPageMeta(pageId) {
  const data = await api(`content/${pageId}?expand=version`, { version: 'v1' })

  const title = typeof data?.title === 'string' ? data.title : `Confluence page ${pageId}`
  const versionNumber = data?.version?.number
  const updatedAt = data?.version?.when
  const webUi = data?._links?.webui
  const url = typeof webUi === 'string'
    ? `${baseUrl}/wiki${webUi}`
    : (SPACE_KEY ? `${baseUrl}/wiki/spaces/${SPACE_KEY}/pages/${pageId}` : `${baseUrl}/wiki/pages/viewpage.action?pageId=${pageId}`)

  return {
    pageId,
    title,
    versionNumber: typeof versionNumber === 'number' ? versionNumber : null,
    updatedAt: typeof updatedAt === 'string' ? updatedAt : null,
    url,
  }
}

async function fetchPageAsMarkdown(pageId, { assetsRootDir, assetsRelativeFromPage, type = '' }) {
  // A typed reverse also needs the raw storage body; the generic path only needs
  // the rendered export_view.
  const expand = type ? 'body.export_view,body.storage,version' : 'body.export_view,version'
  const data = await api(`content/${pageId}?expand=${expand}`, { version: 'v1' })

  const title = typeof data?.title === 'string' ? data.title : `Confluence page ${pageId}`
  const versionNumber = data?.version?.number
  const updatedAt = data?.version?.when
  const webUi = data?._links?.webui
  const url = typeof webUi === 'string'
    ? `${baseUrl}/wiki${webUi}`
    : (SPACE_KEY ? `${baseUrl}/wiki/spaces/${SPACE_KEY}/pages/${pageId}` : `${baseUrl}/wiki/pages/viewpage.action?pageId=${pageId}`)

  const htmlValue = data?.body?.export_view?.value
  if (typeof htmlValue !== 'string') {
    throw new Error(`No export_view body for page ${pageId}`)
  }

  // Diagram round-trip: find the generated diagrams (a `<id>.drawio` with an
  // optional sibling `<id>.png`) up front so their images are downloaded under
  // the clean slug and, after turndown, the flat image is replaced in place by
  // the editable ```drawio block reconstructed from the `.drawio` source.
  const attachments = await loadPageAttachments(pageId)
  const diagramPairs = collectDrawioAttachments(attachments)
  const diagramPngTitles = new Set(diagramPairs.filter((p) => p.png).map((p) => p.png.title))

  // Metadata header block: the page title + Confluence/Page ID/Version/Updated
  // lines that readExistingExports reads for incremental re-export. Shared by
  // both the typed and generic bodies (which follow it, minus their own H1).
  const headerLines = [
    `# ${title}`,
    '',
    `- Confluence: ${url}`,
    `- Page ID: ${pageId}`,
    typeof versionNumber === 'number' ? `- Version: ${versionNumber}` : null,
    typeof updatedAt === 'string' ? `- Updated: ${updatedAt}` : null,
    '',
  ].filter(Boolean)

  const imageRelPrefix = `${assetsRelativeFromPage}/${pageId}`

  // ── Typed reverse: STORAGE → canonical authored markdown (chrome stripped).
  // On a page that isn't this type (or any parse failure) we WARN and drop
  // through to the generic export below (which is why export_view is fetched too).
  let body = null
  const storageValue = data?.body?.storage?.value
  if (type && typeof storageValue === 'string' && storageValue) {
    try {
      const { markdown: doc } = await pullTypedMarkdown({
        type,
        pageId,
        storageXhtml: storageValue,
        title,
        imageRelPrefix,
        assetsDir: join(assetsRootDir, pageId),
        attachments,
        downloadAttachment,
        resolveAccountId: resolveAccountName,
      })
      // serializeDoc emits its own `# Title`; drop it so the metadata header owns
      // the single H1.
      body = doc.replace(/^#\s+.*(?:\r?\n)+/, '').trimEnd()
    } catch (err) {
      const why = err instanceof NotDocTypeError ? `not a ${type} document` : `typed reverse failed`
      process.stdout.write(
        `WARN: ${pageId} — ${title}: ${why} (${err instanceof Error ? err.message : String(err)}); falling back to generic export\n`,
      )
    }
  }

  // ── Generic path (default, or typed fallback): export_view → Turndown dump.
  if (body === null) {
    const html = absolutizeRootRelativeUrls(stripNonContentHtml(htmlValue), baseUrl)
    const localizedHtml = await localizeAssetsInHtml(html, {
      pageId,
      assetsRootDir,
      assetsRelativeFromPage,
      diagramPngTitles,
    })
    const turndown = makeTurndown()
    body = turndown.turndown(localizedHtml).trim()
  }

  let markdown = `${headerLines.join('\n')}\n${body}\n`

  if (diagramPairs.length) {
    const { md } = await reconstructDiagramsInDoc({
      md: markdown,
      pageId,
      attachments,
      assetsDir: join(assetsRootDir, pageId),
      imageRelPrefix: `${assetsRelativeFromPage}/${pageId}`,
      downloadAttachment,
      reverseCli: REVERSE_CLI,
      skipExistingPng: true,
    })
    markdown = md.endsWith('\n') ? md : `${md}\n`
  }

  return markdown
}

async function listDirectChildren(parentPageId) {
  const results = []
  let next = `pages/${parentPageId}/children?limit=250`

  while (next) {
    const data = await api(next)
    if (Array.isArray(data?.results)) results.push(...data.results)

    const nextPath = data?._links?.next
    next = typeof nextPath === 'string' ? new URL(nextPath, baseUrl).toString() : null
  }

  return results
}

async function listSubtreePages(rootPageId) {
  const discovered = new Set()
  const pages = []
  let currentLevel = [rootPageId]

  while (currentLevel.length > 0) {
    const childrenPerParent = await withConcurrency(
      currentLevel.map((parentId) => () => listDirectChildren(parentId)),
      API_CONCURRENCY
    )

    const nextLevel = []

    for (let i = 0; i < currentLevel.length; i++) {
      const parentId = currentLevel[i]
      const children = childrenPerParent[i]

      for (const child of children) {
        const id = typeof child?.id === 'string' ? child.id : null
        if (!id || discovered.has(id)) continue
        discovered.add(id)

        pages.push({
          id,
          title: typeof child?.title === 'string' ? child.title : `Confluence page ${id}`,
          parentId,
          childPosition: typeof child?.childPosition === 'number' ? child.childPosition : null,
        })

        nextLevel.push(id)
      }
    }

    currentLevel = nextLevel
  }

  return pages
}

function uniqueById(pages) {
  const seen = new Set()
  const result = []

  for (const p of pages) {
    const id = typeof p?.id === 'string' ? p.id : null
    if (!id || seen.has(id)) continue
    seen.add(id)
    result.push(p)
  }

  return result
}

function sortTreeEntries(a, b) {
  const posA = typeof a.childPosition === 'number' ? a.childPosition : Number.MAX_SAFE_INTEGER
  const posB = typeof b.childPosition === 'number' ? b.childPosition : Number.MAX_SAFE_INTEGER
  if (posA !== posB) return posA - posB

  return a.title.localeCompare(b.title)
}

function buildTreeLines(childrenByParentId, parentId, depth = 0) {
  const children = childrenByParentId.get(parentId) || []
  const sorted = children.slice().sort(sortTreeEntries)

  const lines = []

  for (const child of sorted) {
    lines.push(`${'  '.repeat(depth)}- [${child.title}](${child.relativePath})`)
    lines.push(...buildTreeLines(childrenByParentId, child.pageId, depth + 1))
  }

  return lines
}

function parseCliArgs(rawArgs) {
  const { flags, positional } = parseArgs(rawArgs, {
    booleans: ['full', 'prune', 'rename', 'help'],
  })
  const type = typeof flags.type === 'string' ? flags.type.toLowerCase() : ''
  if (type && !listDocTypes().includes(type)) {
    console.error(`Unknown --type "${type}". Known types: ${listDocTypes().join(', ')}.`)
    process.exit(1)
  }
  return {
    indexPageId: positional[0] || (typeof flags.index === 'string' ? flags.index : ''),
    outputDir: positional[1] || (typeof flags.out === 'string' ? flags.out : DEFAULT_OUTPUT_DIR),
    type,
    full: Boolean(flags.full),
    prune: Boolean(flags.prune),
    rename: Boolean(flags.rename),
    help: Boolean(flags.help) || rawArgs.includes('-h'),
  }
}

async function main() {
  const opts = parseCliArgs(process.argv.slice(2))
  if (opts.help) {
    process.stdout.write(`${HELP_TEXT}\n`)
    return
  }

  const indexPageId = opts.indexPageId
  if (!indexPageId) {
    console.error('No root page id. Pass --index <page_id> or a positional page id.')
    process.exit(1)
  }
  const outputDir = opts.outputDir

  const repoRoot = process.cwd()
  const outputRoot = resolve(repoRoot, outputDir)
  const pagesDir = join(outputRoot, 'pages')
  const assetsRootDir = join(outputRoot, 'assets')

  await mkdir(pagesDir, { recursive: true })

  // Discover existing exports and the full page tree concurrently
  const [existingExports, subtreeRaw] = await Promise.all([
    readExistingExports(pagesDir),
    listSubtreePages(indexPageId),
  ])

  const subtreePages = uniqueById(subtreeRaw)
  const exportTargets = uniqueById([{ id: indexPageId, parentId: null, childPosition: -1 }, ...subtreePages])
  const remotePageIds = new Set(exportTargets.map((p) => p.id).filter(Boolean))

  // Precompute hierarchy data
  const hasChildrenSet = new Set()
  for (const p of exportTargets) {
    if (p.parentId) hasChildrenSet.add(p.parentId)
  }

  const pageIndex = new Map()
  for (const p of exportTargets) {
    const id = typeof p?.id === 'string' ? p.id : null
    if (!id) continue
    pageIndex.set(id, {
      parentId: typeof p?.parentId === 'string' ? p.parentId : null,
      slug: slugify(typeof p?.title === 'string' ? p.title : `page-${id}`),
    })
  }

  // Ensure unique slugs among siblings
  const siblingsByParent = new Map()
  for (const [id, entry] of pageIndex) {
    const key = entry.parentId ?? '__root__'
    const siblings = siblingsByParent.get(key) || []
    siblings.push({ id, slug: entry.slug })
    siblingsByParent.set(key, siblings)
  }
  for (const [, siblings] of siblingsByParent) {
    const slugCounts = new Map()
    for (const s of siblings) {
      slugCounts.set(s.slug, (slugCounts.get(s.slug) || 0) + 1)
    }
    for (const s of siblings) {
      if (slugCounts.get(s.slug) > 1) {
        s.slug = `${s.slug}-${s.id}`
        pageIndex.get(s.id).slug = s.slug
      }
    }
  }

  // Phase 1: fetch metadata for all pages in parallel
  process.stdout.write(`Fetching metadata for ${exportTargets.length} pages...\n`)
  const metas = await withConcurrency(
    exportTargets.map((p) => () => {
      const pageId = typeof p?.id === 'string' ? p.id : null
      return pageId ? fetchPageMeta(pageId) : Promise.resolve(null)
    }),
    API_CONCURRENCY
  )

  // Phase 2: compute paths for all pages (synchronous)
  const pageInfos = []
  for (let i = 0; i < exportTargets.length; i++) {
    const p = exportTargets[i]
    const meta = metas[i]
    const pageId = typeof p?.id === 'string' ? p.id : null
    if (!pageId || !meta) continue

    const existing = existingExports.get(pageId)
    const localVersion = existing?.version ?? null

    const slug = pageIndex.get(pageId)?.slug ?? slugify(meta.title)
    if (pageIndex.has(pageId)) pageIndex.get(pageId).slug = slug

    const ancestrySegments = buildAncestryPath(pageId, pageIndex, indexPageId)
    const isParent = hasChildrenSet.has(pageId)
    const isRoot = pageId === indexPageId

    let pageDir
    let computedFileName
    if (isRoot) {
      pageDir = pagesDir
      computedFileName = '_index.md'
    } else if (isParent) {
      pageDir = join(pagesDir, ...ancestrySegments, slug)
      computedFileName = '_index.md'
    } else {
      pageDir = join(pagesDir, ...ancestrySegments)
      computedFileName = `${slug}.md`
    }

    const filePath = join(pageDir, computedFileName)

    const relativeFromPages = isRoot
      ? '_index.md'
      : isParent
        ? join(...ancestrySegments, slug, '_index.md')
        : join(...ancestrySegments, `${slug}.md`)

    const depthFromPages = isRoot
      ? 0
      : isParent
        ? ancestrySegments.length + 1
        : ancestrySegments.length
    const assetsRelativeFromPage = '../'.repeat(depthFromPages + 1) + 'assets'

    pageInfos.push({
      p,
      pageId,
      meta,
      existing,
      localVersion,
      pageDir,
      filePath,
      relativeFromPages,
      assetsRelativeFromPage,
    })
  }

  // Phase 3: handle renames, create dirs, and check file existence — all in parallel
  await Promise.all(pageInfos.map(({ existing, filePath, pageDir }) =>
    Promise.all([
      mkdir(pageDir, { recursive: true }),
      existing && existing.filePath !== filePath
        ? (async () => {
            const oldPath = existing.filePath
            const newExists = await fileExists(filePath)
            if (newExists) {
              await rm(oldPath, { force: true }).catch(() => null)
            } else if (await fileExists(oldPath)) {
              await mkdir(dirname(filePath), { recursive: true })
              await rename(oldPath, filePath).catch(() => null)
            }
          })()
        : Promise.resolve(),
    ])
  ))

  // Check which files already exist on disk (after renames)
  const fileExistsResults = await Promise.all(pageInfos.map(({ filePath }) => fileExists(filePath)))

  const summary = {
    total: exportTargets.length,
    exported: 0,
    updated: 0,
    created: 0,
    skipped: 0,
  }

  const toExport = pageInfos.map((info, i) => ({
    info,
    shouldExport:
      opts.full ||
      !info.existing ||
      !fileExistsResults[i] ||
      info.meta.versionNumber === null ||
      info.localVersion === null ||
      info.meta.versionNumber !== info.localVersion,
  }))

  for (const { info, shouldExport } of toExport) {
    if (!shouldExport) {
      process.stdout.write(`Skipping ${info.pageId} — ${info.meta.title} (no changes)\n`)
      summary.skipped += 1
    }
  }

  // Phase 4: export changed pages in parallel
  await withConcurrency(
    toExport
      .filter(({ shouldExport }) => shouldExport)
      .map(({ info }) => async () => {
        const { pageId, meta, existing, filePath, assetsRelativeFromPage } = info
        process.stdout.write(`Exporting ${pageId} — ${meta.title} (${existing ? 'update' : 'new'})\n`)
        await rm(join(assetsRootDir, pageId), { recursive: true, force: true }).catch(() => null)

        const markdown = await fetchPageAsMarkdown(pageId, { assetsRootDir, assetsRelativeFromPage, type: opts.type })
        await writeFile(filePath, markdown, 'utf8')

        summary.exported += 1
        if (existing) summary.updated += 1
        else summary.created += 1
      }),
    EXPORT_CONCURRENCY
  )

  // Build index entries (order preserved from exportTargets)
  const indexEntries = toExport.map(({ info }) => ({
    title: info.meta.title,
    pageId: info.pageId,
    relativePath: `./pages/${info.relativeFromPages}`,
    url: info.meta.url,
    parentId: typeof info.p?.parentId === 'string' ? info.p.parentId : null,
    childPosition: typeof info.p?.childPosition === 'number' ? info.p.childPosition : null,
  }))

  // Root meta is already in the metas array — no extra API call needed
  const rootMetaFromBatch = metas[exportTargets.findIndex((p) => p.id === indexPageId)] ?? null
  const rootMeta = rootMetaFromBatch ?? (await fetchPageMeta(indexPageId))

  const readmePath = join(outputRoot, 'README.md')
  const childrenByParentId = new Map()
  for (const entry of indexEntries) {
    const key = entry.parentId
    const items = childrenByParentId.get(key) || []
    items.push(entry)
    childrenByParentId.set(key, items)
  }

  const rootEntry = indexEntries.find((e) => e.pageId === indexPageId) || null
  const rootPageFileLine = rootEntry
    ? `- Root page file: [${rootMeta.title}](${rootEntry.relativePath})`
    : `- Root page file: ${rootMeta.title} (not exported)`

  const readmeLines = [
    '# Confluence documentation (export)',
    '',
    `- Root: ${rootMeta.url}`,
    rootPageFileLine,
    '',
    '## Tree',
    '',
    ...buildTreeLines(childrenByParentId, null),
    '',
  ]

  const nextReadme = `${readmeLines.join('\n')}\n`
  const prevReadme = (await readFile(readmePath, 'utf8').catch(() => null)) ?? null
  if (prevReadme !== nextReadme) {
    await writeFile(readmePath, nextReadme, 'utf8')
  }

  // Migration: remove old files that have been re-exported to new hierarchical paths
  const exportedPaths = new Set(indexEntries.map((e) => join(pagesDir, e.relativePath.replace(/^\.\/pages\//, ''))))
  for (const [pageId, existing] of existingExports) {
    if (!remotePageIds.has(pageId)) continue
    if (!exportedPaths.has(existing.filePath) && (await fileExists(existing.filePath))) {
      await rm(existing.filePath, { force: true }).catch(() => null)
      process.stdout.write(`Migrated ${pageId}: removed old file ${existing.filePath}\n`)
    }
  }

  if (opts.prune) {
    const localIds = Array.from(existingExports.keys())
    const orphanIds = localIds.filter((id) => !remotePageIds.has(id))
    for (const orphanId of orphanIds) {
      const orphan = existingExports.get(orphanId)
      if (!orphan) continue
      await rm(orphan.filePath, { force: true }).catch(() => null)
      await rm(join(assetsRootDir, orphanId), { recursive: true, force: true }).catch(() => null)
      process.stdout.write(`Pruned ${orphanId} — ${orphan.title || orphan.fileName}\n`)
    }
  }

  await cleanEmptyDirs(pagesDir)

  const scriptDir = dirname(fileURLToPath(import.meta.url))
  process.stdout.write(`\nDone.\n`)
  process.stdout.write(`- Output: ${outputRoot}\n`)
  process.stdout.write(`- Script: ${scriptDir}\n`)
  process.stdout.write(
    `- Summary: total=${summary.total}, exported=${summary.exported} (new=${summary.created}, updated=${summary.updated}), skipped=${summary.skipped}\n`
  )
}

main().catch((err) => {
  console.error(`ERROR: ${err instanceof Error ? err.message : String(err)}`)
  process.exit(1)
})
