#!/usr/bin/env node
/**
 * Validate Confluence export: compare exported .md files to live Confluence pages.
 * Usage: node validate-confluence-export.mjs [--dir <path>] [--dir <path> ...] [--sample N] [--report <path>]
 * Default: validates confluence-awsf and confluence-trsample in cwd.
 */
import { readFile, readdir, writeFile } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'
import { api, baseUrl } from './lib/atlassian/auth.mjs'
import { parseArgs } from './lib/util/cli-args.mjs'
import {
  makeTurndown,
  stripNonContentHtml,
  absolutizeRootRelativeUrls,
  parseExportHeader,
} from './lib/atlassian/html-to-markdown.mjs'

const require = createRequire(import.meta.url)
const domino = require('@mixmark-io/domino')

const __dirname = dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = resolve(__dirname, '..', '..', '..', '..')

function toPlainText(mdOrHtml) {
  const noHtml = mdOrHtml.replace(/<[^>]+>/g, ' ')
  const noMd = noHtml
    .replace(/!\[[^\]]*\]\([^)]+\)/g, ' ')
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/[#*_~`]/g, '')
    .replace(/^\s*[-+*]\s+/gm, ' ')
    .replace(/^\s*\d+\.\s+/gm, ' ')
  return noMd.replace(/\s+/g, ' ').trim()
}

function wordCount(s) {
  return (s || '').split(/\s+/).filter(Boolean).length
}

async function collectExportedPages(pagesDir) {
  const list = []
  async function scan(dir) {
    const entries = await readdir(dir, { withFileTypes: true }).catch(() => [])
    for (const e of entries) {
      const full = join(dir, e.name)
      if (e.isDirectory()) await scan(full)
      else if (e.isFile() && e.name.endsWith('.md')) {
        const raw = await readFile(full, 'utf8').catch(() => '')
        const parsed = parseExportHeader(raw)
        if (parsed.pageId) list.push({ ...parsed, filePath: full })
      }
    }
  }
  await scan(pagesDir)
  return list
}

async function fetchConfluencePage(pageId) {
  const data = await api(`content/${pageId}?expand=body.export_view,version`, { version: 'v1' })
  const title = typeof data?.title === 'string' ? data.title : null
  const version = data?.version?.number ?? null
  const html = data?.body?.export_view?.value
  if (typeof html !== 'string') return { title, version, bodyPlain: '', bodyMd: '' }
  const prepared = stripNonContentHtml(absolutizeRootRelativeUrls(html, baseUrl))
  const window = domino.createWindow(prepared)
  const htmlFromBody = window.document.body?.innerHTML ?? ''
  const turndown = makeTurndown()
  const bodyMd = turndown.turndown(htmlFromBody).trim()
  const bodyPlain = toPlainText(bodyMd)
  return { title, version, bodyPlain, bodyMd }
}

function shuffle(arr) {
  const a = [...arr]
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]]
  }
  return a
}

const HELP = `Validate Confluence export: compare exported .md to live Confluence pages.

Usage:
  node validate-confluence-export.mjs [options]

Options:
  --dir <path>    Add pages dir to validate (default: confluence-awsf/pages, confluence-trsample/pages)
  --sample N      Validate only N random pages (default: all)
  --report <path> Write JSON report to file
  --help          Show this help

Runs from repo root. Uses .cursor/.project/.env for Confluence API.
Compares: title (exact) and body word count (ratio 0.88–1.12). Body mismatch can mean
page was updated since export or has macros/expand that export to less text.
`

async function main() {
  const args = process.argv.slice(2)
  if (args.includes('--help') || args.includes('-h')) {
    process.stdout.write(HELP)
    return
  }
  const { flags } = parseArgs(args, { repeatable: ['dir'] })
  const dirs = (Array.isArray(flags.dir) ? flags.dir : flags.dir ? [flags.dir] : [])
    .filter((d) => typeof d === 'string')
    .map((d) => resolve(process.cwd(), d))
  const sample = typeof flags.sample === 'string' ? parseInt(flags.sample, 10) : null
  const reportPath = typeof flags.report === 'string' ? resolve(process.cwd(), flags.report) : null
  const pagesDirs = dirs.length > 0 ? dirs : [
    join(REPO_ROOT, 'confluence-awsf', 'pages'),
    join(REPO_ROOT, 'confluence-trsample', 'pages'),
  ].filter((p) => p)

  const all = []
  for (const pagesDir of pagesDirs) {
    const name = pagesDir.includes('confluence-awsf') ? 'confluence-awsf' : pagesDir.includes('confluence-trsample') ? 'confluence-trsample' : pagesDir
    const pages = await collectExportedPages(pagesDir)
    all.push({ name, pages })
  }

  const toValidate = all.flatMap(({ name, pages }) => pages.map((p) => ({ ...p, exportName: name })))
  const validated = sample != null && sample > 0 ? shuffle(toValidate).slice(0, sample) : toValidate

  const results = { ok: [], titleMismatch: [], bodyMismatch: [], error: [] }
  const WORD_RATIO_MIN = 0.88
  const WORD_RATIO_MAX = 1.12

  process.stdout.write(`Validating ${validated.length} pages (${sample != null ? `sample of ${toValidate.length}` : 'all'})...\n`)

  for (let i = 0; i < validated.length; i++) {
    const exp = validated[i]
    const { pageId, title: expTitle, body: expBody, filePath, exportName } = exp
    try {
      const live = await fetchConfluencePage(pageId)
      if (live.title === null && !live.bodyPlain && !live.bodyMd) {
        results.error.push({ pageId, exportName, filePath, reason: 'Confluence returned no content' })
        continue
      }
      const titleOk = live.title !== null && live.title.trim() === (expTitle || '').trim()
      const expPlain = toPlainText(expBody)
      const expWords = wordCount(expPlain)
      const liveWords = wordCount(live.bodyPlain)
      const ratio = liveWords > 0 ? expWords / liveWords : 1
      const bodyOk = ratio >= WORD_RATIO_MIN && ratio <= WORD_RATIO_MAX

      if (!titleOk) {
        results.titleMismatch.push({
          pageId,
          exportName,
          filePath,
          exported: expTitle,
          live: live.title,
        })
      }
      if (!bodyOk) {
        results.bodyMismatch.push({
          pageId,
          exportName,
          filePath,
          exportedWords: expWords,
          liveWords,
          ratio: Math.round(ratio * 100) / 100,
        })
      }
      if (titleOk && bodyOk) {
        results.ok.push({ pageId, exportName, title: expTitle })
      }
    } catch (err) {
      results.error.push({
        pageId,
        exportName,
        filePath,
        reason: err instanceof Error ? err.message : String(err),
      })
    }
    if ((i + 1) % 50 === 0) process.stdout.write(`  ${i + 1}/${validated.length}\n`)
  }

  const total = validated.length
  const okCount = results.ok.length
  const tCount = results.titleMismatch.length
  const bCount = results.bodyMismatch.length
  const eCount = results.error.length

  process.stdout.write('\n--- Summary ---\n')
  process.stdout.write(`Total checked: ${total}\n`)
  process.stdout.write(`OK (title + body match): ${okCount}\n`)
  process.stdout.write(`Title mismatch: ${tCount}\n`)
  process.stdout.write(`Body word-count mismatch: ${bCount}\n`)
  process.stdout.write(`Errors (fetch failed): ${eCount}\n`)

  if (tCount > 0) {
    process.stdout.write('\nTitle mismatches (first 10):\n')
    results.titleMismatch.slice(0, 10).forEach((r) => {
      process.stdout.write(`  ${r.pageId} [${r.exportName}]: exported "${(r.exported || '').slice(0, 40)}..." vs live "${(r.live || '').slice(0, 40)}..."\n`)
    })
  }
  if (bCount > 0) {
    process.stdout.write('\nBody mismatches (first 10):\n')
    results.bodyMismatch.slice(0, 10).forEach((r) => {
      process.stdout.write(`  ${r.pageId} [${r.exportName}]: exported ${r.exportedWords} words vs live ${r.liveWords} (ratio ${r.ratio})\n`)
    })
  }
  if (eCount > 0) {
    process.stdout.write('\nErrors (first 10):\n')
    results.error.slice(0, 10).forEach((r) => {
      process.stdout.write(`  ${r.pageId} [${r.exportName}]: ${r.reason}\n`)
    })
  }

  if (reportPath) {
    await writeFile(
      reportPath,
      JSON.stringify(
        {
          summary: { total, ok: okCount, titleMismatch: tCount, bodyMismatch: bCount, error: eCount },
          ok: results.ok,
          titleMismatch: results.titleMismatch,
          bodyMismatch: results.bodyMismatch,
          error: results.error,
        },
        null,
        2
      ),
      'utf8'
    )
    process.stdout.write(`\nReport written to ${reportPath}\n`)
  }
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
