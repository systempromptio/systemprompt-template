/**
 * Orchestration layer for publishing markdown documents to Confluence.
 *
 * publishDoc(options) → { pageId, url, inlineReport }
 *
 * options:
 *   mdPath          – absolute path to the source .md file
 *   docType         – 'fsd' | 'isd' | undefined  (undefined = untyped/raw publish)
 *   pageId          – existing Confluence page ID (UPDATE mode)
 *   title           – page title (CREATE mode, required when no pageId)
 *   parent          – parent page ID override (CREATE mode)
 *                     With a profile and no --parent, falls back to CONFLUENCE_<TYPE>_PARENT_ID.
 *                     Without a profile, parent is required.
 *   render          – 'template' | 'markdown' | undefined. A typed publish (docType set)
 *                     defaults to 'template' (the FSD/ISD chrome); undefined + no docType is
 *                     plain 'markdown'. An explicit 'markdown' opts a typed publish out.
 *   mentionMap      – { 'Full Name': 'accountId', … }
 *   skipAttachments – boolean
 *   skipMatrix      – boolean. A typed publish refreshes the parent page's approval
 *                     matrix (see matrix.mjs); set this to leave the parent alone.
 *   dry             – boolean (writes preview HTML, no API calls)
 *   dryOutPath      – file path for dry-run output (default: os.tmpdir()/doc-preview.html)
 *   spaceKey        – Confluence space key for URL building (default: CONFLUENCE_SPACE_KEY env)
 *   comment         – Confluence version-history message for this update (<=50 chars).
 *                     Falls back to the doc-type profile's versionMessage when absent.
 */

import { readFileSync, writeFileSync } from 'node:fs'
import { resolve, dirname, join } from 'node:path'
import { tmpdir } from 'node:os'
// atlassian/auth and atlassian/url are lazy-imported inside publishDoc so --dry works
// without credentials (importing them triggers load-env + the creds check).
import { mdToStorage, collectAssets } from './md-to-storage.mjs'
import { resolveProfile, tocMacro } from './profiles.mjs'
import { getDocType } from './types/index.mjs'
import { renderDoc } from './render.mjs'
import { statusColours, deriveClientName } from './status-vocab.mjs'
import { resolveMentions } from '../atlassian/users.mjs'
import { stampDocMeta, readDocMeta } from './meta.mjs'
import { escHtml as esc, escAttr } from '../util/xhtml.mjs'

// Confluence API client — assigned once credentials are lazily imported inside
// publishDoc(). Module-scoped so the helpers below can reach it.
let api

// ─── Space resolution ─────────────────────────────────────────────────────────

// Confluence REST v2 addresses spaces by numeric spaceId (Long), but humans and
// config use the space key (e.g. "SFPA"). Resolve the key to its numeric id via
// GET /wiki/api/v2/spaces?keys=<KEY> so callers keep configuring CONFLUENCE_SPACE_KEY.
async function resolveSpaceId(spaceKey) {
  const data = await api(`spaces?keys=${encodeURIComponent(spaceKey)}`)
  const space = data?.results?.find((s) => s.key === spaceKey) ?? data?.results?.[0]
  if (!space?.id) {
    throw new Error(`Confluence space not found for key "${spaceKey}". Check CONFLUENCE_SPACE_KEY.`)
  }
  return space.id
}

// ─── Inline comment re-anchoring ─────────────────────────────────────────────

// A typed publish's page title must name the document type ("… FSD" / "… ISD")
// so the Confluence page tree is self-describing and reviewers can tell the
// document's purpose at a glance. Append the token when the authored title (or
// the document's h1) does not already carry it as a standalone word.
function ensureDocTypeInTitle(title, docType) {
  const t = String(title == null ? '' : title).trim()
  const token = String(docType || '').toUpperCase()
  if (!t || (token !== 'FSD' && token !== 'ISD')) return t
  return new RegExp(`\\b${token}\\b`, 'i').test(t) ? t : `${t} ${token}`
}

// The integration/specification name for the page label: the effective title
// minus its doc-type token ("Consent Manager Integration ISD" → "Consent Manager
// Integration"). ensureDocTypeInTitle appended the token, so strip a standalone
// FSD/ISD word (anywhere) back out.
function titleWithoutDocType(title, docType) {
  const token = String(docType || '').toUpperCase()
  const t = String(title == null ? '' : title).trim()
  if (token !== 'FSD' && token !== 'ISD') return t
  return t.replace(new RegExp(`\\b${token}\\b`, 'ig'), '').replace(/\s{2,}/g, ' ').trim()
}

// Confluence label names are single tokens (no spaces, lowercased): slugify.
function slugifyLabel(s) {
  return String(s == null ? '' : s)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

// Tag a typed page with its doc type + the integration/spec name so the page
// tree is filterable by both. Additive (Confluence keeps any pre-existing
// labels); a duplicate name is a no-op server-side.
async function applyDocLabels(pageId, docType, title) {
  const names = [slugifyLabel(docType), slugifyLabel(titleWithoutDocType(title, docType))].filter(Boolean)
  if (!names.length) return []
  await api(`content/${pageId}/label`, {
    method: 'POST',
    version: 'v1',
    body: names.map((name) => ({ prefix: 'global', name })),
  })
  return names
}

async function fetchOpenInlineComments(pageId) {
  const results = []
  let url = `pages/${pageId}/inline-comments?body-format=storage&limit=100`
  while (url) {
    const data = await api(url)
    for (const c of data.results || []) {
      if (c.resolutionStatus && c.resolutionStatus !== 'open') continue
      const ref = c.properties?.inlineMarkerRef
      const sel = c.properties?.inlineOriginalSelection
      if (ref && sel) results.push({ id: c.id, ref, sel })
    }
    const next = data._links?.next
    url = next ? next.replace(/^.*\/wiki\/api\/v2\//, '') : null
  }
  return results
}

function injectInlineMarkers(body, comments) {
  const restored = []
  const lost = []
  for (const c of comments) {
    const needle = esc(c.sel)
    const idx = body.indexOf(needle)
    if (idx === -1) { lost.push(c); continue }
    const count = body.split(needle).length - 1
    const wrapped =
      `<ac:inline-comment-marker ac:ref="${escAttr(c.ref)}">${needle}</ac:inline-comment-marker>`
    body = body.slice(0, idx) + wrapped + body.slice(idx + needle.length)
    restored.push({ ...c, ambiguous: count > 1 })
  }
  return { body, restored, lost }
}

// ─── Content-appearance (page width) property ──────────────────────────────────

// value: 'default' (Narrow, fixed width) or 'full-width'. Always written when a
// profile is active so switching widths takes effect (otherwise a previously set
// value would linger on the page).
async function setContentAppearance(pageId, value) {
  const key = 'content-appearance-published'
  const existing = await api(`pages/${pageId}/properties`).catch(() => null)
  const prop = existing?.results?.find((p) => p.key === key)
  if (prop) {
    if (prop.value === value) return
    await api(`pages/${pageId}/properties/${prop.id}`, {
      method: 'PUT',
      body: { key, value, version: { number: prop.version.number + 1 } },
    })
  } else {
    await api(`pages/${pageId}/properties`, { method: 'POST', body: { key, value } })
  }
}

// ─── Main ─────────────────────────────────────────────────────────────────────

export async function publishDoc({
  mdPath,
  docType,
  pageId: pageId0,
  title,
  parent,
  mentionMap = {},
  render,
  skipAttachments = false,
  skipValidation = false,
  skipMatrix = false,
  dry = false,
  dryOutPath,
  spaceKey,
  comment,
}) {
  const profile = resolveProfile(docType)
  // Confluence version-history message for this update: caller-supplied --comment
  // (capped at 50 chars) wins over the doc-type profile default, which wins over
  // the generic fallback.
  const versionMessage = (() => {
    const c = typeof comment === 'string' ? comment.trim() : ''
    if (c) return c.slice(0, 50)
    return profile?.versionMessage ?? 'Updated'
  })()
  // Typed publishes route through the doc-type registry (fsd default when a
  // template render is forced without --type); the agnostic core stays type-free.
  const docTypeImpl = getDocType(docType)
  const mdAbs = resolve(mdPath)
  const mdDir = dirname(mdAbs)
  const md = readFileSync(mdAbs, 'utf8')
  // A typed publish renders the FSD/ISD chrome by default; only an explicit
  // --render=markdown opts out. Untyped stays plain unless --render=template.
  const useTemplate = render === 'template' || (Boolean(profile) && render !== 'markdown')

  // Template mode enforces the markdown formatting canon (required H2 sections)
  // before doing anything else, so a malformed document never reaches Confluence.
  // Layout (dividers / spacing) is owned by the renderer, not validated here.
  // Bypass with --skip-validation.
  if (useTemplate && !skipValidation) {
    const fmt = docTypeImpl.validateFormat(md)
    for (const w of fmt.warnings) console.warn(`WARN (format): ${w}`)
    if (!fmt.ok) {
      throw new Error(`Document format is invalid:\n  - ${fmt.errors.join('\n  - ')}\n(use --skip-validation to bypass)`)
    }
  }

  // Template mode parses the canonical markdown into a model up-front so the
  // title can default from the h1 and mentions can be resolved before rendering.
  const model = useTemplate ? docTypeImpl.parse(md) : null
  const assetPaths = collectAssets(md)
  // Resolve the vocabulary's client-review placeholder against this document, so
  // a backtick status word in the body badges like the chrome lozenges do.
  const badgeMap = profile ? statusColours(model ? deriveClientName(model) : '') : {}

  // "Linked Jira Tickets" macro inputs: the live page id (known on update or from
  // a prior publish's stamped header) + the site's Confluence app id / cloudId.
  // Absent any of these the renderer omits the section (it self-heals on the next
  // publish once the page exists / config is set).
  const knownPageId = pageId0 || readDocMeta(md).pageId || ''
  const jiraAppId = process.env.CONFLUENCE_APP_ID || ''
  const jiraCloudId = process.env.CONFLUENCE_CLOUD_ID || ''

  // Build the storage body from a resolved mention map. In template mode the
  // Nunjucks chrome renderer owns the TOC/status/mention chrome; in markdown mode
  // we keep the legacy mdToStorage + TOC-macro path.
  const buildBody = (mmap) => {
    if (useTemplate) {
      // Pass the profile's status colour map so backtick status words in the
      // body render as coloured badges too (chrome badges come from the model).
      return renderDoc(model, {
        type: docType,
        mentionMap: mmap,
        includeToc: true,
        badgeMap,
        pageId: knownPageId,
        jiraAppId,
        jiraCloudId,
      })
    }
    const { body: rawBody } = mdToStorage(md, { mentionMap: mmap, badgeMap })
    return profile ? tocMacro(profile) + '\n' + rawBody : rawBody
  }

  // On a typed publish the title must carry the doc-type token; append it when
  // the authored title / h1 omits it (untyped publishes are left untouched).
  let effectiveTitle = ensureDocTypeInTitle(title || (useTemplate ? model.title : undefined), docType)

  if (dry) {
    const body = buildBody(mentionMap)
    const out = dryOutPath || join(tmpdir(), 'doc-preview.html')
    writeFileSync(out, body)
    console.log(`Dry run — wrote ${out}, no API calls`)

    // Inline-comment risk preview (read-only). On an UPDATE, surface which open
    // inline comments this edit WOULD drop BEFORE the real publish — so the human
    // decides with the risk in hand, not after the fact. Purely read-only (no
    // PUT). Kept non-fatal: with no credentials / offline, --dry still works and
    // simply skips the preview.
    let inlineReport = null
    if (pageId0) {
      try {
        const auth = await import('../atlassian/auth.mjs')
        api = auth.api
        const comments = await fetchOpenInlineComments(pageId0)
        if (comments.length) {
          const r = injectInlineMarkers(body, comments)
          inlineReport = { restored: r.restored, lost: r.lost }
          console.log(
            `Inline-comment risk (preview): ${comments.length} open, ` +
              `${r.restored.length} would re-anchor, ${r.lost.length} would be dropped`,
          )
          for (const c of r.lost) {
            console.log(`  WOULD DROP comment ${c.id}: "${c.sel.slice(0, 80)}"`)
          }
        } else {
          console.log('Inline-comment risk (preview): no open inline comments on the live page')
        }
      } catch (err) {
        console.log(`Inline-comment risk (preview): skipped (${err.message})`)
      }
    }

    // Attachment plan (preview). On an UPDATE, show which assets WOULD upload /
    // update / be pruned BEFORE the real publish, so orphan deletions (removed or
    // renamed images) are visible at the offer -> preview gate. Read-only.
    if (pageId0 && !skipAttachments && assetPaths.length) {
      try {
        const { collectDiagrams } = await import('../diagrams/publish.mjs')
        const { syncAttachmentsToPage } = await import('../diagrams/attachment-sync.mjs')
        const d = collectDiagrams({ mdDir, assetPaths })
        const plan = await syncAttachmentsToPage({
          pageId: pageId0,
          files: [...d.images, ...d.companions],
          dry: true,
          prune: true,
        })
        console.log(
          `Attachment plan (preview): ${plan.uploaded} upload, ${plan.updated} update, ` +
            `${plan.skipped} unchanged, ${plan.pruned} prune` +
            (plan.missing ? `, ${plan.missing} missing` : ''),
        )
        for (const t of plan.prunedTitles) console.log(`  WOULD DELETE attachment ${t}`)
        for (const w of plan.warnings) console.log(`  NOTE unreferenced (not managed, kept): ${w}`)
      } catch (err) {
        console.log(`Attachment plan (preview): skipped (${err.message})`)
      }
    }
    return { pageId: null, url: null, inlineReport }
  }

  // Load credentials + .env before validating config. Importing auth.mjs triggers
  // load-env.mjs as a side effect, so process.env is populated before the checks below.
  // Kept after the --dry early-return so dry runs still work without credentials.
  // Assign the module-scoped bindings so the helpers above can use the client.
  const auth = await import('../atlassian/auth.mjs')
  api = auth.api
  const { pageUrl } = await import('../atlassian/url.mjs')

  // From here on credentials + parent are required
  const SPACE_KEY =
    spaceKey || process.env.CONFLUENCE_SPACE_KEY || (process.env.CONFLUENCE_SPACES || '').split(',')[0].trim()
  if (!SPACE_KEY) throw new Error('CONFLUENCE_SPACE_KEY must be set in .cursor/.project/.env')

  // Resolve @mention account ids on the fly (template mode only). Explicit
  // mentionMap entries win over looked-up ids.
  let effectiveMentions = mentionMap
  if (useTemplate) {
    const names = docTypeImpl.collectMentionNames(model)
    const { map, unresolved } = await resolveMentions(api, names)
    effectiveMentions = { ...map, ...mentionMap }
    if (unresolved.length) {
      console.log(`Mentions: ${Object.keys(map).length} resolved, unresolved (rendered as text): ${unresolved.join(', ')}`)
    } else if (names.length) {
      console.log(`Mentions: ${names.length} resolved`)
    }
  }
  let body = buildBody(effectiveMentions)

  // Resolve parent page ID
  let parentId = parent
  if (!parentId && profile) {
    parentId = process.env[profile.parentEnv]
  }
  if (!pageId0 && !parentId) {
    throw new Error(
      docType
        ? `Parent page ID required. Set ${profile.parentEnv} in .env or pass --parent=<id>.`
        : 'Parent page ID required when --type is not set. Pass --parent=<id>.',
    )
  }
  if (!pageId0 && !effectiveTitle) {
    throw new Error('--title is required when creating a new page (no --page-id given).')
  }

  // Lazy-load the hash-gated attachment-sync layer + diagram-aware delta
  // (only needed for real publishes).
  const { syncAttachmentsToPage } = await import('../diagrams/attachment-sync.mjs')
  const { collectDiagrams } = await import('../diagrams/publish.mjs')

  // Resolve referenced images and detect generated diagrams (image with a
  // sibling <slug>.drawio). Diagram images get their .drawio companion uploaded
  // alongside the PNG (the durable "this is a diagram" signal is that sibling
  // attachment).
  const diagrams = collectDiagrams({ mdDir, assetPaths })

  let pageId = pageId0
  let inlineReport = null
  // The page the approval matrix belongs on: the document's OWN parent as
  // Confluence reports it, not the configured default — a doc republished under a
  // different parent must refresh the report that actually aggregates it.
  let docParentId = null

  if (pageId) {
    // UPDATE — re-anchor open inline comments first
    const comments = await fetchOpenInlineComments(pageId)
    if (comments.length) {
      const r = injectInlineMarkers(body, comments)
      body = r.body
      inlineReport = r
      console.log(
        `Inline comments: ${comments.length} open, ${r.restored.length} re-anchored, ${r.lost.length} could not be re-anchored`,
      )
    }
    const cur = await api(`pages/${pageId}`)
    await api(`pages/${pageId}`, {
      method: 'PUT',
      body: {
        id: pageId,
        status: cur.status,
        title: cur.title,
        body: { representation: 'storage', value: body },
        version: { number: cur.version.number + 1, message: versionMessage },
      },
    })
    docParentId = cur.parentId || null
    console.log(`Updated page ${pageId} → v${cur.version.number + 1}`)
  } else {
    // CREATE — v2 requires the numeric spaceId, resolved from the configured space key
    const spaceId = await resolveSpaceId(SPACE_KEY)
    const data = await api('pages', {
      method: 'POST',
      body: {
        spaceId,
        status: 'current',
        title: effectiveTitle,
        parentId,
        body: { representation: 'storage', value: body },
      },
    })
    pageId = data.id
    docParentId = data.parentId || parentId || null
    console.log(`Created page ${pageId}`)
  }

  // Page width (profile-driven). Always set when a profile is active so the
  // value can be switched between publishes.
  if (profile?.contentAppearance) {
    await setContentAppearance(pageId, profile.contentAppearance)
    console.log(`Content appearance: ${profile.contentAppearance}`)
  }

  // Page labels (typed publishes only): doc type + integration/spec name, so the
  // page tree is filterable. Secondary metadata — a failure here must not fail
  // the publish (mirrors the metadata-stamp step below).
  if (docType) {
    try {
      const labels = await applyDocLabels(pageId, docType, effectiveTitle)
      if (labels.length) console.log(`Labels: ${labels.join(', ')}`)
    } catch (err) {
      console.log(`NOTE — could not apply page labels: ${err.message}`)
    }
  }

  // Approval matrix on the parent (typed publishes only). The header we just wrote
  // is a Content Properties source, so the parent's aggregated report is rebuilt
  // here instead of by a follow-up step: it can never drift from the documents, and
  // a new document appears in the matrix the moment it is published. Runs AFTER the
  // labels because the report is scoped by them. Nothing is written when the
  // rebuilt report is identical, so a re-publish does not churn the parent's
  // version history. Secondary metadata — a failure must not fail the publish.
  if (docType && !skipMatrix) {
    if (!docParentId) {
      console.log('NOTE — page has no parent, so no approval matrix was refreshed')
    } else {
      try {
        const { refreshDocMatrix } = await import('./matrix.mjs')
        const m = await refreshDocMatrix({
          api,
          type: docType,
          parentId: docParentId,
          propertiesId: getDocType(docType).propertiesId,
        })
        if (m.action === 'skipped') {
          console.log(`Approval matrix: nothing to aggregate on parent ${docParentId} yet`)
        } else {
          const stale = m.unlabelled.length ? `, ${m.unlabelled.length} unlabelled page(s) not shown` : ''
          console.log(
            `Approval matrix: ${m.action} on parent ${docParentId} — ${m.contributors.length} document(s), ` +
              `columns ${m.columns.join(' | ')}${stale}`,
          )
        }
      } catch (err) {
        console.log(`NOTE — could not refresh the approval matrix: ${err.message}`)
      }
    }
  }

  // Attachments — hash-gated: unchanged files are skipped (stable link kept),
  // only changed bytes cause a new attachment version. Diagram images also carry
  // their .drawio companion (downloadable source) in the same push.
  if (!skipAttachments && assetPaths.length) {
    const files = [...diagrams.images, ...diagrams.companions]
    const r = await syncAttachmentsToPage({ pageId, files, prune: true })
    console.log(
      `Attachments: ${r.uploaded} uploaded, ${r.updated} updated, ${r.skipped} unchanged` +
        (r.missing ? `, ${r.missing} missing` : '') +
        (r.pruned ? `, ${r.pruned} pruned` : '') +
        (diagrams.companions.length ? ` (incl. ${diagrams.companions.length} .drawio)` : ''),
    )
    for (const t of r.prunedTitles) console.log(`  Deleted orphaned attachment: ${t}`)
    for (const w of r.warnings) console.log(`  NOTE — unreferenced attachment kept (not managed by us): ${w}`)
  } else if (skipAttachments) {
    console.log('Skipped attachment upload')
  }

  const url = pageUrl(SPACE_KEY, pageId)
  console.log('SUCCESS')
  console.log(`Page ID: ${pageId}`)
  console.log(`URL: ${url}`)

  // Script-owned link stamping: record the live page identity in the working
  // copy's header, in the canonical format (the same a later pull re-stamps).
  try {
    const stamp = stampDocMeta(mdAbs, { url, pageId })
    if (stamp.changed) console.log(`Stamped Confluence URL + Page ID into ${mdAbs}`)
  } catch (err) {
    console.log(`NOTE — could not stamp metadata header into ${mdAbs}: ${err.message}`)
  }

  if (inlineReport?.lost?.length) {
    console.log('')
    console.log('WARNING — inline comments that could NOT be re-anchored (text changed/removed):')
    for (const c of inlineReport.lost) {
      console.log(`  - comment ${c.id}: "${c.sel.slice(0, 80)}"`)
      console.log(`    ${url}?focusedCommentId=${c.id}`)
    }
    console.log('  These are now dangling in Confluence (resolved). Re-add manually if still relevant.')
  }
  if (inlineReport?.restored?.some((c) => c.ambiguous)) {
    console.log('')
    console.log('NOTE — some re-anchored comments matched text that appears multiple times; verify placement:')
    for (const c of inlineReport.restored.filter((x) => x.ambiguous)) {
      console.log(`  - comment ${c.id}: "${c.sel.slice(0, 80)}"`)
    }
  }

  return { pageId, url, inlineReport }
}
