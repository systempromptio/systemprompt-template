/**
 * stories-md.mjs — pure (no I/O, no network) parsing and backfilling for the
 * deterministic `stories.md` produced by generate-functional-artifacts.mjs. Kept separate from
 * jira.mjs so the text logic is unit-testable without Jira credentials.
 *
 * A story block looks like:
 *
 *   ### TBD-1 — <WBS> — <name>
 *
 *   **User story:** …
 *
 *   **Acceptance criteria:** [<WBS> — Acceptance Criteria](<fsd-url>#Acceptance-Criteria[.N])
 *
 *   **Implementation notes:**
 *
 *   - …
 *
 *   **References:**
 *
 *   - FSD: [<feature>](<url>)
 *
 * User story and acceptance criteria are paragraphs (bold label + value), not list
 * items; implementation notes and references are bold headers followed by a bullet
 * list — so the Jira description renders with real structure instead of one flat
 * bullet list.
 *
 * On `create-stories`, the heading id (`TBD-n`) is replaced with the real Jira key,
 * a `Jira tickets created via /opsx:propose-approved: <keys>` line is written above
 * `## Stories`, and a `- Jira: [<KEY>](<browse-url>)` reference is added to each
 * story. All three are idempotent (safe to re-run / refresh).
 */

// Parse the story blocks under `## Stories`. Returns [{ id, title, body, block }]
// where `body` is the block text without its heading line and `block` is the raw
// block text (heading + body, including its trailing newlines).
export function parseStoryBlocks(md) {
  const storiesIdx = md.search(/^##\s+Stories\s*$/m)
  const rest = storiesIdx === -1 ? md : md.slice(md.indexOf('\n', storiesIdx) + 1)
  return rest
    .split(/(?=^###\s)/m)
    .filter((p) => /^###\s/.test(p))
    .map((block) => {
      const hm = /^###\s+(\S+)\s+[—-]\s+(.*)$/m.exec(block)
      const id = hm ? hm[1].trim() : ''
      const title = hm ? hm[2].trim() : ''
      const bodyLines = block.split('\n')
      bodyLines.shift() // drop the heading line
      const body = bodyLines.join('\n').trim()
      return { id, title, body, block }
    })
}

// Extract the linked Confluence doc from a story block: the first Confluence
// `/wiki/.../pages/<id>/<slug>` URL found (the `**Acceptance criteria:**` deep
// link, else a `- FSD:` / `- ISD:` reference). The URL fragment (e.g.
// `#Acceptance-Criteria`) is stripped so the remote link targets the page, and
// the slug is decoded into a human title. Returns { url, pageId, title } with
// nulls/'' when the doc is unpublished (a local file reference has no pageId).
export function extractDocLink(block) {
  const linkRe = /\]\(([^)]+)\)/g
  let m
  while ((m = linkRe.exec(block)) !== null) {
    const href = m[1].trim()
    const pm = /\/wiki\/[^\s)]*?\/pages\/(\d+)(?:\/([^)?#\s]*))?/.exec(href)
    if (pm) {
      const url = href.split('#')[0]
      const title = pm[2] ? decodeURIComponent(pm[2].replace(/\+/g, ' ')).trim() : ''
      return { url, pageId: pm[1], title }
    }
  }
  return { url: null, pageId: null, title: '' }
}

// Build the Jira remote-link payload for a Confluence page. Using an
// `application` of type `com.atlassian.confluence` + a `globalId` of
// `appId=<...>&pageId=<...>` is what makes Jira treat it as a first-class
// "Confluence content" link (surfaced by the page's Jira Links button and the
// `issuesWithRemoteLinksByGlobalId` JQL) — a plain `{object:{url,title}}` link is
// just a web link and is invisible to both. The globalId also makes re-POSTs an
// idempotent upsert (same id updates in place instead of duplicating).
export function buildConfluenceRemoteLink({ appId, pageId, url, title }) {
  return {
    globalId: `appId=${appId}&pageId=${pageId}`,
    application: { type: 'com.atlassian.confluence', name: 'System Confluence' },
    relationship: 'Wiki Page',
    object: { url, title: title || url },
  }
}

// The description text sent to Jira for a story: the block body verbatim, minus
// any `- Jira:` reference a previous run added (so the idempotency hash is stable
// across re-runs).
export function descriptionForIssue(body) {
  return body
    .replace(/^\s*-\s*Jira:\s*\[.*$/gm, '')
    .replace(/\n{3,}/g, '\n\n')
    .trim()
}

// Insert or update the `- Jira: [<KEY>](<url>)` reference inside a single story
// block string. Idempotent. References is a bold-header paragraph followed by
// top-level `- ` reference bullets, so the Jira ref is a sibling bullet.
export function injectJiraRef(block, key, url) {
  const jiraLine = `- Jira: [${key}](${url})`
  const lines = block.split('\n')

  const existingJira = lines.findIndex((l) => /^\s*-\s*Jira:\s*\[/.test(l))
  if (existingJira !== -1) {
    lines[existingJira] = jiraLine
    return lines.join('\n')
  }

  const refIdx = lines.findIndex((l) => /^\s*\*\*References:\*\*\s*$/.test(l))
  if (refIdx === -1) {
    // No References section — append one at the end of the block (after trimming
    // the block's trailing blank lines, then restoring one).
    while (lines.length && lines[lines.length - 1].trim() === '') lines.pop()
    lines.push('', '**References:**', '', jiraLine, '')
    return lines.join('\n')
  }

  // Insert after the last contiguous reference bullet under `**References:**`
  // (skipping the blank line the header is followed by).
  let i = refIdx + 1
  while (i < lines.length && lines[i].trim() === '') i++
  while (i < lines.length && /^\s*-\s+/.test(lines[i])) i++
  lines.splice(i, 0, jiraLine)
  return lines.join('\n')
}

// Backfill created/reused Jira keys + links into the whole stories.md string.
// `results` is [{ id, title, key, url }] (entries without a key are ignored for
// linking but still excluded from the created-list). Returns the new markdown.
export function backfillStoriesMd(md, results) {
  const byId = new Map(results.filter((r) => r.key).map((r) => [r.id, r]))
  const keys = results.filter((r) => r.key).map((r) => r.key)
  if (!keys.length) return md

  const storiesIdx = md.search(/^##\s+Stories\s*$/m)
  if (storiesIdx === -1) return md
  const nlAfter = md.indexOf('\n', storiesIdx)
  const pre = md.slice(0, storiesIdx)
  const storiesHeaderLine = md.slice(storiesIdx, nlAfter === -1 ? md.length : nlAfter)
  const rest = nlAfter === -1 ? '' : md.slice(nlAfter + 1)

  const newRest = rest
    .split(/(?=^###\s)/m)
    .map((part) => {
      if (!/^###\s/.test(part)) return part
      const hm = /^###\s+(\S+)\s+[—-]\s+/m.exec(part)
      if (!hm) return part
      const r = byId.get(hm[1].trim())
      if (!r) return part
      let p = part.replace(/^(###\s+)\S+(\s+[—-]\s+)/m, `$1${r.key}$2`)
      p = injectJiraRef(p, r.key, r.url)
      return p
    })
    .join('')

  const noteLine = `Jira tickets created via /opsx:propose-approved: ${keys.join(', ')}`
  const newPre = /^Jira tickets created via .*$/m.test(pre)
    ? pre.replace(/^Jira tickets created via .*$/m, noteLine)
    : `${pre.replace(/\s*$/, '')}\n\n${noteLine}\n\n`

  return `${newPre}${storiesHeaderLine}\n${newRest}`
}
