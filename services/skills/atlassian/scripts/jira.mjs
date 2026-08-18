#!/usr/bin/env node
import { createHash } from 'node:crypto'
import { mkdirSync, readdirSync, readFileSync, statSync, unlinkSync, writeFileSync } from 'node:fs'
import { basename, dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { jiraApi, agileApi, baseUrl, AUTH, die } from './lib/atlassian/auth.mjs'
import { buildAdfFromText } from './lib/atlassian/adf.mjs'
import { parseStoryBlocks, descriptionForIssue, backfillStoriesMd, extractDocLink, buildConfluenceRemoteLink } from './lib/jira/stories-md.mjs'

const __dirname = dirname(fileURLToPath(import.meta.url))

const DEFAULT_BOARD_ID = process.env.JIRA_BOARD_ID ? parseInt(process.env.JIRA_BOARD_ID, 10) : null
const PROJECT_KEY = process.env.JIRA_PROJECT_KEY
const TMP_DIR = resolve(__dirname, '../../../../tmp')
const FAILED_PAYLOAD_PATH = join(TMP_DIR, 'last-jira-payload.json')

// Instance/project-specific Jira field mappings. Custom-field IDs and option IDs
// differ per Jira instance, so they are configured via .env (see env.example),
// never hard-coded. Discover them via the Jira field metadata API during onboarding.
function parseJsonEnv(name) {
    const raw = process.env[name]
    if (!raw) return {}
    try {
        return JSON.parse(raw)
    } catch {
        die(`${name} must be valid JSON (got: ${raw})`)
    }
}
const ASSIGNEE_MAP = parseJsonEnv('JIRA_ASSIGNEE_MAP') // { "FE": "<accountId>", "BE": "<accountId>" }
// Confluence application id (from an app link) used in the remote-link globalId
// (`appId=<...>&pageId=<...>`) so create-stories can back-link each Story to its
// FSD/ISD page as a first-class Confluence link. Discover via `link-config`.
const CONFLUENCE_APP_ID = process.env.CONFLUENCE_APP_ID || ''

const COMMANDS = {
    'get-issue': {
        usage: '<issue_key> [fields]',
        desc: 'Get issue details including all comments (comments are always fetched automatically)',
        async run(
            issueKey,
            fields = 'summary,status,issuetype,priority,assignee,description,created,updated,comment'
        ) {
            if (!issueKey) die('issue_key required')
            if (!fields.includes('comment')) fields += ',comment'
            const data = await jiraApi(`issue/${issueKey}?fields=${fields}`)
            const f = data.fields || {}

            console.log(`Key: ${data.key}`)
            console.log(`ID: ${data.id}`)
            console.log(`Summary: ${f.summary || 'N/A'}`)
            console.log(`Type: ${f.issuetype?.name || 'N/A'}`)
            console.log(`Status: ${f.status?.name || 'N/A'}`)
            console.log(`Priority: ${f.priority?.name || 'N/A'}`)
            console.log(`Assignee: ${f.assignee?.displayName || 'Unassigned'}`)
            console.log(`Created: ${f.created || 'N/A'}`)
            console.log(`Updated: ${f.updated || 'N/A'}`)
            console.log(`URL: ${baseUrl}/browse/${data.key}`)

            if (f.description) {
                console.log('---')
                const text = extractAdfText(f.description)
                console.log(`Description: ${text}`)
            }

            const comments = f.comment?.comments ?? []
            if (comments.length > 0) {
                console.log('---')
                console.log(`Comments: ${comments.length}`)
                for (const c of comments) {
                    console.log('---')
                    console.log(`Comment ID: ${c.id}`)
                    console.log(`Author: ${c.author?.displayName || 'Unknown'}`)
                    console.log(`Created: ${c.created}`)
                    console.log(`Body: ${extractAdfText(c.body)}`)
                }
            }
        }
    },

    'get-attachments': {
        usage: '<issue_key>',
        desc: 'Download image attachments to .cursor/tmp/jira-attachments/ and print their paths for AI to read. Clears previous attachments on each run.',
        async run(issueKey) {
            if (!issueKey) die('issue_key required')

            const SAFE_DIR = resolve(__dirname, '../../../../tmp/jira-attachments')
            const expectedSuffix = join('.cursor', 'tmp', 'jira-attachments')
            if (!SAFE_DIR.endsWith(expectedSuffix)) {
                die(`Unexpected safe dir path: ${SAFE_DIR}`)
            }

            mkdirSync(SAFE_DIR, { recursive: true })

            for (const entry of readdirSync(SAFE_DIR)) {
                const filePath = join(SAFE_DIR, entry)
                if (!filePath.startsWith(SAFE_DIR))
                    die(`Refusing to delete outside safe dir: ${filePath}`)
                if (statSync(filePath).isFile()) unlinkSync(filePath)
            }

            const IMAGE_EXTENSIONS = new Set([
                '.png',
                '.jpg',
                '.jpeg',
                '.gif',
                '.webp',
                '.bmp',
                '.svg',
                '.tif',
                '.tiff'
            ])

            const data = await jiraApi(`issue/${issueKey}?fields=attachment`)
            const attachments = data.fields?.attachment ?? []
            const images = attachments.filter((a) => {
                if (a.mimeType?.startsWith('image/')) return true
                const ext = a.filename?.slice(a.filename.lastIndexOf('.')).toLowerCase()
                return ext ? IMAGE_EXTENSIONS.has(ext) : false
            })

            if (images.length === 0) {
                console.log(`No image attachments found on ${issueKey}`)
                return
            }

            console.log(`Attachments: ${images.length} image(s) on ${issueKey}`)
            console.log('---')

            for (const att of images) {
                const res = await fetch(att.content, { headers: { Authorization: AUTH } })
                if (!res.ok) {
                    console.warn(`Warning: skipped ${att.filename} — HTTP ${res.status}`)
                    continue
                }
                const buf = await res.arrayBuffer()
                const filePath = join(SAFE_DIR, att.filename)
                writeFileSync(filePath, Buffer.from(buf))
                console.log(filePath)
            }

            console.log('---')
            console.log(
                'Read the file paths above. They will be cleared on the next get-attachments run.'
            )
        }
    },

    'create-issue': {
        usage: '<type> <summary> [description] [priority] [assignee_id] [parent_key]',
        desc: 'Create issue. Type: Task|Bug|Story|Epic|Spike',
        async run(issueType, summary, description, priority = 'Major', assigneeId, parentKey) {
            if (!issueType || !summary) die('type, summary required')

            if (!PROJECT_KEY) die('JIRA_PROJECT_KEY must be set in .cursor/.project/.env')
            const fields = {
                project: { key: PROJECT_KEY },
                issuetype: { name: issueType },
                summary,
                priority: { name: priority }
            }

            if (description) {
                const descInput = tryReadFile(description)
                if (typeof description === 'string' && description.endsWith('.json')) {
                    const adf = JSON.parse(descInput)
                    if (adf.type !== 'doc' || adf.version !== 1)
                        die('ADF JSON must have type "doc" and version 1')
                    fields.description = adf
                } else {
                    fields.description = buildAdfFromText(descInput)
                }
            }
            if (assigneeId) fields.assignee = { accountId: assigneeId }
            if (parentKey) fields.parent = { key: parentKey }

            const data = await jiraApi('issue', { method: 'POST', body: { fields } })
            console.log('SUCCESS')
            console.log(`Key: ${data.key}`)
            console.log(`ID: ${data.id}`)
            console.log(`URL: ${baseUrl}/browse/${data.key}`)
        }
    },

    'create-story': {
        usage: '--summary=<...> --desc-file=<...> [--priority=Major] [--assignee=FE|BE|<id>] [--epic=PROJ-123] [--labels=csv] [--dedup=<keywords>] [--dry-run]',
        desc: 'Create a Story from an approved stories.md block. Pre-flight validates Story fields, the assignee, optional epic parent, and duplicate candidates before any write. Assignee defaults to the API token user when --assignee is omitted (override with FE|BE from JIRA_ASSIGNEE_MAP or a raw accountId). Embeds an idempotency hash in description — re-runs return the existing key instead of creating a duplicate. On HTTP failure, dumps the request body to .cursor/tmp/last-jira-payload.json. --dry-run performs read-only checks and prints the payload.',
        async run(...args) {
            if (!PROJECT_KEY) die('JIRA_PROJECT_KEY must be set in .cursor/.project/.env')
            const flags = parseFlags(args)

            const summary = flags.summary
            if (!summary) die('--summary required')
            const descFile = flags['desc-file']
            if (!descFile) die('--desc-file required')

            const priority = flags.priority || 'Major'
            const assignee = flags.assignee || ''
            const epicKey = flags.epic || ''
            const labelsCsv = flags.labels || ''
            const dedupKeywords = flags.dedup || ''
            const dryRun = !!flags['dry-run']
            // Assignee defaults to the API token user (`myself`) when no flag is
            // given, so projects that mark assignee required never hit a 400 for
            // an omitted field. `myself` is fetched lazily only on that fallback.
            let assigneeId = ASSIGNEE_MAP[assignee] || assignee || ''
            if (!assigneeId) {
                const me = await jiraApi('myself').catch(() => null)
                assigneeId = me?.accountId || ''
            }

            const descText = tryReadFile(descFile)
            const idemHash = computeIdemHash(summary, descText)
            const descTextWithIdem = `${descText.trimEnd()}\n\n_idem: ${idemHash}_\n`

            // === Pre-flight (read-only, parallel) ===
            const [createMeta, issueTypesResp, assignableCheck, epicData] = await Promise.all([
                jiraApi(`issue/createmeta?projectKeys=${PROJECT_KEY}&issuetypeNames=Story&expand=projects.issuetypes.fields`),
                jiraApi(`issue/createmeta/${PROJECT_KEY}/issuetypes`).catch(() => null),
                assigneeId ? jiraApi(`user/assignable/search?project=${PROJECT_KEY}&accountId=${assigneeId}`).catch(() => []) : Promise.resolve(null),
                epicKey ? jiraApi(`issue/${epicKey}?fields=issuetype,summary,status`).catch(() => null) : Promise.resolve(null)
            ])

            const storyType = createMeta?.projects?.[0]?.issuetypes?.find((t) => t.name === 'Story')
            const metaFields = storyType?.fields || {}
            const validationErrors = []

            if (!storyType) {
                const knownTypes = (issueTypesResp?.issueTypes || issueTypesResp?.values || [])
                    .map((t) => t.name)
                    .filter(Boolean)
                validationErrors.push(
                    knownTypes.length
                        ? `Story issue type is not available for ${PROJECT_KEY}. Valid types: ${knownTypes.join(', ')}`
                        : `Story issue type is not available for ${PROJECT_KEY}`
                )
            }

            const allowedPriorities = (metaFields.priority?.allowedValues || []).map((p) => p.name)
            if (allowedPriorities.length && !allowedPriorities.includes(priority)) {
                validationErrors.push(`priority "${priority}" not in allowed values: ${allowedPriorities.join(', ')}`)
            }

            if (assigneeId) {
                const assignable = Array.isArray(assignableCheck) ? assignableCheck : []
                const matched = assignable.some((u) => u.accountId === assigneeId)
                if (!matched) {
                    validationErrors.push(`assignee accountId "${assigneeId}" is not assignable to ${PROJECT_KEY} project (user inactive, not a project member, or wrong id)`)
                }
            }

            if (epicKey) {
                if (!epicData) {
                    validationErrors.push(`epic "${epicKey}" not accessible (wrong key or no permission)`)
                } else {
                    const epicType = epicData.fields?.issuetype?.name || ''
                    if (epicType.toLowerCase() !== 'epic') {
                        validationErrors.push(`parent "${epicKey}" is ${epicType || 'unknown type'}, expected Epic`)
                    }
                }
            }

            if (validationErrors.length > 0) {
                console.error('Pre-flight validation failed (no Jira write performed):')
                for (const e of validationErrors) console.error(`  - ${e}`)
                process.exit(1)
            }

            // === Idempotency check ===
            const idemSearch = await jiraApi(
                `search/jql?jql=${encodeURIComponent(`project = ${PROJECT_KEY} AND text ~ "idem ${idemHash}"`)}&maxResults=5&fields=summary,status`
            )
            if (idemSearch.issues?.length > 0) {
                console.log('IDEMPOTENT_MATCH_FOUND')
                console.log('An identical Story (matching summary + description) already exists:')
                for (const i of idemSearch.issues) {
                    console.log(`${i.key} | ${i.fields?.summary || 'N/A'} | ${i.fields?.status?.name || 'N/A'}`)
                }
                console.log(`URL: ${baseUrl}/browse/${idemSearch.issues[0].key}`)
                return
            }

            // === Keyword dedup check ===
            if (dedupKeywords) {
                const safeKeywords = dedupKeywords.replace(/\\/g, '\\\\').replace(/"/g, '\\"')
                const dupJql = `project = ${PROJECT_KEY} AND issuetype = Story AND summary ~ "${safeKeywords}"`
                const dupSearch = await jiraApi(
                    `search/jql?jql=${encodeURIComponent(dupJql)}&maxResults=5&fields=summary,status`
                )
                if (dupSearch.issues?.length > 0) {
                    console.log('DUPLICATES_FOUND')
                    console.log(`Results: ${dupSearch.issues.length}`)
                    console.log('---')
                    for (const i of dupSearch.issues) {
                        console.log(`${i.key} | ${i.fields?.summary || 'N/A'} | ${i.fields?.status?.name || 'N/A'}`)
                    }
                    console.log('To proceed anyway, re-run without --dedup.')
                    return
                }
                console.log('No duplicates found — proceeding.')
            }

            // === Build atomic payload ===
            const fields = {
                project: { key: PROJECT_KEY },
                issuetype: { name: 'Story' },
                summary,
                priority: { name: priority },
                description: buildAdfFromText(descTextWithIdem)
            }
            if (assigneeId) fields.assignee = { accountId: assigneeId }
            if (labelsCsv) {
                fields.labels = labelsCsv.split(',').map((l) => l.trim()).filter(Boolean)
            }
            if (epicKey) fields.parent = { key: epicKey }

            if (dryRun) {
                console.log('DRY RUN — no Jira write performed.')
                console.log('---')
                console.log('Pre-flight: PASS')
                console.log(`Idempotency hash: ${idemHash}`)
                console.log(`Epic: ${epicKey || '(none)'}`)
                console.log('---')
                console.log('Payload that would be POSTed to /issue:')
                console.log(JSON.stringify({ fields }, null, 2))
                return
            }

            // === Atomic POST ===
            let data
            try {
                data = await jiraApi('issue', { method: 'POST', body: { fields } })
            } catch (err) {
                dumpFailedPayload(fields, err.message)
                console.error(`ERROR: ${err.message}`)
                console.error(`Payload dumped to ${FAILED_PAYLOAD_PATH}`)
                console.error('Diagnose with read-only endpoints only. Do NOT retry-loop with POST.')
                process.exit(1)
            }

            console.log('SUCCESS')
            console.log(`Key: ${data.key}`)
            console.log(`ID: ${data.id}`)
            console.log(`URL: ${baseUrl}/browse/${data.key}`)
            if (epicKey) console.log(`Epic: ${epicKey}`)
            console.log(`Idempotency hash: ${idemHash}`)
        }
    },

    'create-stories': {
        usage: '<stories.md> [--epic=PROJ-123] [--priority=Major] [--assignee=FE|BE|<id>] [--dry-run] [--refresh]',
        desc: 'Create/reuse Jira Stories straight from a deterministic stories.md (each "### <ID> — <title>" block: summary = title, description = block body verbatim). Assignee defaults to the API token user when --assignee is omitted (override with FE|BE from JIRA_ASSIGNEE_MAP or a raw accountId). Reuses existing issues by an idempotency hash embedded in the description — re-runs never duplicate. On success, backfills stories.md in place: heading ids -> real keys, a "Jira tickets created via /opsx:propose-approved: <keys>" line above ## Stories, and a "- Jira: [<KEY>](<url>)" reference per story. --dry-run: read-only preview. --refresh: search-only (no create) to re-stamp Jira links for already-created tickets.',
        async run(...args) {
            if (!PROJECT_KEY) die('JIRA_PROJECT_KEY must be set in .cursor/.project/.env')
            const flags = parseFlags(args)
            const positional = args.filter((a) => typeof a === 'string' && !a.startsWith('--'))
            const storiesPath = positional[0]
            if (!storiesPath) die('<stories.md> path required')

            const epicKey = flags.epic || ''
            const priority = flags.priority || 'Major'
            const assignee = flags.assignee || ''
            const dryRun = !!flags['dry-run']
            const refresh = !!flags.refresh
            const noLink = !!flags['no-link']

            let md
            try {
                md = readFileSync(storiesPath, 'utf8')
            } catch {
                die(`cannot read stories file: ${storiesPath}`)
            }

            const blocks = parseStoryBlocks(md)
            if (!blocks.length) die(`no story blocks ("### <ID> — <title>") found in ${storiesPath}`)

            // One-time epic pre-flight (skip in refresh mode — refresh never creates).
            if (epicKey && !refresh && !dryRun) {
                const epicData = await jiraApi(`issue/${epicKey}?fields=issuetype,summary,status`).catch(() => null)
                if (!epicData) die(`epic "${epicKey}" not accessible (wrong key or no permission)`)
                const epicType = epicData.fields?.issuetype?.name || ''
                if (epicType.toLowerCase() !== 'epic') {
                    die(`parent "${epicKey}" is ${epicType || 'unknown type'}, expected Epic`)
                }
            }

            // One-time assignee pre-flight (skip in refresh mode — refresh never
            // creates). Assignee defaults to the API token user (`myself`) so
            // projects that mark assignee required never hit a 400 for an omitted
            // field. Validate the resolved account is assignable BEFORE any POST —
            // this runs in --dry-run too, so a dry run cannot claim success for a
            // create the live run would reject.
            let assigneeId = ASSIGNEE_MAP[assignee] || assignee || ''
            if (!refresh) {
                if (!assigneeId) {
                    const me = await jiraApi('myself').catch(() => null)
                    assigneeId = me?.accountId || ''
                }
                if (assigneeId) {
                    const assignable = await jiraApi(`user/assignable/search?project=${PROJECT_KEY}&accountId=${assigneeId}`).catch(() => [])
                    const matched = Array.isArray(assignable) && assignable.some((u) => u.accountId === assigneeId)
                    if (!matched) {
                        die(`assignee accountId "${assigneeId}" is not assignable to ${PROJECT_KEY} project (user inactive, not a project member, or wrong id). Pass --assignee=<assignable accountId> or set JIRA_ASSIGNEE_MAP.`)
                    }
                }
            }

            const results = [] // { id, title, action, key, url, idemHash }
            for (const b of blocks) {
                if (!b.title) {
                    console.error(`WARNING: skipping a block with no parseable title (id "${b.id}")`)
                    continue
                }
                const descText = descriptionForIssue(b.body)
                const idemHash = computeIdemHash(b.title, descText)

                // Reuse an existing issue by idempotency hash. New issues embed the
                // hash in the description as an `_idem: <hash>_` marker, so the
                // `text ~ "idem <hash>"` clause matches on re-runs.
                const idemSearch = await jiraApi(
                    `search/jql?jql=${encodeURIComponent(`project = ${PROJECT_KEY} AND text ~ "idem ${idemHash}"`)}&maxResults=5&fields=summary,status`
                )
                let existing = idemSearch.issues?.[0]

                // In refresh mode, fall back to a summary match so links can be
                // re-stamped even for issues created before the hash existed.
                if (!existing && refresh) {
                    const safe = b.title.replace(/\\/g, '\\\\').replace(/"/g, '\\"')
                    const dupJql = `project = ${PROJECT_KEY} AND issuetype = Story AND summary ~ "${safe}"`
                    const dupSearch = await jiraApi(
                        `search/jql?jql=${encodeURIComponent(dupJql)}&maxResults=5&fields=summary,status`
                    )
                    existing = dupSearch.issues?.[0]
                }

                if (existing) {
                    results.push({ id: b.id, title: b.title, action: 'reuse', key: existing.key, url: `${baseUrl}/browse/${existing.key}`, idemHash })
                    continue
                }
                if (refresh) {
                    results.push({ id: b.id, title: b.title, action: 'missing', key: null, url: null, idemHash })
                    continue
                }
                if (dryRun) {
                    results.push({ id: b.id, title: b.title, action: 'create', key: null, url: null, idemHash })
                    continue
                }

                // === Create ===
                // The idempotency hash is embedded in the description as an
                // `_idem: <hash>_` marker so re-runs find and reuse this issue (see
                // the reuse search above). The hash itself is computed from the
                // clean title + body, so the marker never perturbs it.
                const descTextWithIdem = `${descText.trimEnd()}\n\n_idem: ${idemHash}_\n`
                const fields = {
                    project: { key: PROJECT_KEY },
                    issuetype: { name: 'Story' },
                    summary: b.title,
                    priority: { name: priority },
                    description: buildAdfFromText(descTextWithIdem)
                }
                if (assigneeId) fields.assignee = { accountId: assigneeId }
                if (epicKey) fields.parent = { key: epicKey }

                let data
                try {
                    data = await jiraApi('issue', { method: 'POST', body: { fields } })
                } catch (err) {
                    dumpFailedPayload(fields, err.message)
                    console.error(`ERROR creating story "${b.title}": ${err.message}`)
                    console.error(`Payload dumped to ${FAILED_PAYLOAD_PATH}`)
                    console.error('Diagnose with read-only endpoints only. Do NOT retry-loop with POST.')
                    process.exit(1)
                }
                results.push({ id: b.id, title: b.title, action: 'created', key: data.key, url: `${baseUrl}/browse/${data.key}`, idemHash })
            }

            // === Back-link each story to its FSD/ISD page (Confluence-typed remote
            // link). The page URL/id is parsed from the story block's own AC / FSD
            // reference; the globalId makes re-POSTs an idempotent upsert. Missing
            // config or an unpublished doc (no pageId) is skipped, not fatal. ===
            let linkedCount = 0
            let linkSkipMissingCfg = false
            let linkNoPage = 0
            if (!noLink) {
                const blockById = new Map(blocks.map((b) => [b.id, b.block]))
                for (const r of results) {
                    if (!r.key) continue
                    const { url, pageId, title } = extractDocLink(blockById.get(r.id) || '')
                    if (!pageId) { linkNoPage += 1; continue }
                    if (!CONFLUENCE_APP_ID) { linkSkipMissingCfg = true; continue }
                    if (dryRun) { linkedCount += 1; continue }
                    try {
                        await jiraApi(`issue/${r.key}/remotelink`, {
                            method: 'POST',
                            body: buildConfluenceRemoteLink({ appId: CONFLUENCE_APP_ID, pageId, url, title })
                        })
                        linkedCount += 1
                    } catch (err) {
                        console.error(`WARNING: could not link ${r.key} -> page ${pageId}: ${err.message}`)
                    }
                }
            }

            // === Report ===
            const mode = dryRun ? 'DRY RUN — no Jira write performed.' : refresh ? 'REFRESH — search-only, no Jira write performed.' : 'create-stories'
            console.log(mode)
            console.log('---')
            for (const r of results) {
                const label =
                    r.action === 'created' ? `CREATE -> ${r.key}`
                    : r.action === 'reuse' ? `REUSE ${r.key}`
                    : r.action === 'create' ? 'CREATE (would create a new Story)'
                    : 'MISSING (no existing issue found)'
                console.log(`- ${r.id} "${r.title}": ${label}`)
            }
            if (epicKey) console.log(`Epic: ${epicKey}`)
            if (assigneeId && !refresh) console.log(`Assignee: ${assigneeId}`)
            if (!noLink) {
                const verb = dryRun ? 'would link' : 'linked'
                if (linkedCount) console.log(`Doc links: ${verb} ${linkedCount} story(ies) to their Confluence page`)
                if (linkSkipMissingCfg) console.log('Doc links: SKIPPED — CONFLUENCE_APP_ID not set (run: node jira.mjs link-config <linked-issue>)')
                if (linkNoPage) console.log(`Doc links: ${linkNoPage} story(ies) had no published doc link (page not yet published) — skipped`)
            }

            // === Backfill ===
            if (dryRun) {
                console.log('---')
                console.log('Backfill preview (NOT written): heading ids -> Jira keys, a "Jira tickets created via /opsx:propose-approved: <keys>" line above ## Stories, and a "- Jira: [<KEY>](<url>)" reference per story.')
                return
            }

            const linkable = results.filter((r) => r.key)
            if (linkable.length) {
                const updated = backfillStoriesMd(md, results)
                if (updated !== md) {
                    writeFileSync(storiesPath, updated, 'utf8')
                    console.log('---')
                    console.log(`Backfilled ${linkable.length} Jira link(s) into ${storiesPath}`)
                } else {
                    console.log('---')
                    console.log('stories.md already up to date — no backfill changes.')
                }
            }
            const missing = results.filter((r) => r.action === 'missing')
            if (missing.length) {
                console.log(`NOTE — no existing issue found to refresh for: ${missing.map((m) => m.id).join(', ')}`)
            }
        }
    },

    'get-comments': {
        usage: '<issue_key>',
        desc: 'Get all comments on an issue (newest last). Always check comments when researching a ticket — they often contain critical context, QA notes, and decisions.',
        async run(issueKey) {
            if (!issueKey) die('issue_key required')
            const data = await jiraApi(`issue/${issueKey}/comment?orderBy=created`)
            const comments = data.comments ?? []

            if (comments.length === 0) {
                console.log(`No comments on ${issueKey}`)
                return
            }

            console.log(`Comments: ${comments.length} on ${issueKey}`)
            for (const c of comments) {
                console.log('---')
                console.log(`ID: ${c.id}`)
                console.log(`Author: ${c.author?.displayName || 'Unknown'}`)
                console.log(`Created: ${c.created}`)
                console.log(`Body: ${extractAdfText(c.body)}`)
            }
        }
    },

    'edit-issue': {
        usage: '<issue_key> <fields_json>',
        desc: 'Update issue fields (JSON inline or file path)',
        async run(issueKey, fieldsInput) {
            if (!issueKey || !fieldsInput) die('issue_key, fields_json required')
            const fields = JSON.parse(tryReadFile(fieldsInput))

            await jiraApi(`issue/${issueKey}`, { method: 'PUT', body: { fields } })
            console.log(`SUCCESS — ${issueKey} updated`)
            console.log(`URL: ${baseUrl}/browse/${issueKey}`)
        }
    },

    transition: {
        usage: '<issue_key> [target_status]',
        desc: 'Transition issue. Without status — lists available transitions',
        async run(issueKey, targetStatus) {
            if (!issueKey) die('issue_key required')

            const data = await jiraApi(`issue/${issueKey}/transitions`)

            if (!targetStatus) {
                console.log(`Available transitions for ${issueKey}:`)
                for (const t of data.transitions) {
                    console.log(`  ID: ${t.id} → ${t.name} (${t.to?.name || 'N/A'})`)
                }
                return
            }

            const target = targetStatus.toLowerCase()
            const transition = data.transitions.find(
                (t) => t.name.toLowerCase() === target || t.to?.name.toLowerCase() === target
            )

            if (!transition) {
                die(
                    `No transition for "${targetStatus}". Available: ${data.transitions.map((t) => t.name).join(', ')}`
                )
            }

            await jiraApi(`issue/${issueKey}/transitions`, {
                method: 'POST',
                body: { transition: { id: transition.id } }
            })
            console.log(`SUCCESS — ${issueKey} → ${targetStatus}`)
            console.log(`URL: ${baseUrl}/browse/${issueKey}`)
        }
    },

    'add-to-sprint': {
        usage: '<issue_key> [board_id]',
        desc: 'Move issue into the active sprint. Requires JIRA_BOARD_ID in .cursor/.project/.env or pass board_id explicitly.',
        async run(issueKey, boardId) {
            if (!issueKey) die('issue_key required')
            const board = boardId || (DEFAULT_BOARD_ID ? String(DEFAULT_BOARD_ID) : null)
            if (!board) die('board_id required (or set JIRA_BOARD_ID in .cursor/.project/.env)')
            const sprints = await agileApi(`board/${board}/sprint?state=active`)
            const values = sprints.values ?? sprints
            const active = Array.isArray(values)
                ? values[0]
                : values?.find((s) => s.state === 'active')
            if (!active) {
                die(
                    `No active sprint found for board ${board}. Create or start a sprint on the board first.`
                )
            }
            await agileApi(`sprint/${active.id}/issue`, {
                method: 'POST',
                body: { issues: [issueKey] }
            })
            console.log(`SUCCESS — ${issueKey} added to sprint "${active.name}" (id: ${active.id})`)
            console.log(`URL: ${baseUrl}/browse/${issueKey}`)
        }
    },

    search: {
        usage: '<jql_query> [max_results] [fields]',
        desc: 'JQL search',
        async run(
            jql,
            maxResults = '50',
            fields = 'summary,status,issuetype,priority,assignee,created'
        ) {
            if (!jql) die('jql_query required')
            const encoded = encodeURIComponent(jql)
            const data = await jiraApi(
                `search/jql?jql=${encoded}&maxResults=${maxResults}&fields=${fields}`
            )

            console.log(`Results: ${data.issues.length} of ${data.total || '?'}`)
            console.log('---')
            for (const i of data.issues) {
                const f = i.fields || {}
                console.log(`Key: ${i.key}`)
                console.log(`Summary: ${f.summary || 'N/A'}`)
                console.log(`Type: ${f.issuetype?.name || 'N/A'}`)
                console.log(`Status: ${f.status?.name || 'N/A'}`)
                console.log(`Priority: ${f.priority?.name || 'N/A'}`)
                console.log(`Assignee: ${f.assignee?.displayName || 'Unassigned'}`)
                console.log(`URL: ${baseUrl}/browse/${i.key}`)
                console.log('---')
            }
        }
    },

    'add-comment': {
        usage: '<issue_key> <comment_body>',
        desc: 'Add comment (text or file path). Supports #/##/### headings, - bullet lists, and links.',
        async run(issueKey, bodyInput) {
            if (!issueKey || !bodyInput) die('issue_key, comment_body required')
            const text = tryReadFile(bodyInput)

            const data = await jiraApi(`issue/${issueKey}/comment`, {
                method: 'POST',
                body: {
                    body: buildAdfFromText(text)
                }
            })
            console.log('SUCCESS')
            console.log(`Comment ID: ${data.id}`)
            console.log(`Author: ${data.author?.displayName || 'N/A'}`)
        }
    },

    'edit-comment': {
        usage: '<issue_key> <comment_id> <comment_body>',
        desc: 'Edit comment (text or file path). Supports simple bullet/link formatting.',
        async run(issueKey, commentId, bodyInput) {
            if (!issueKey || !commentId || !bodyInput)
                die('issue_key, comment_id, comment_body required')
            const text = tryReadFile(bodyInput)

            await jiraApi(`issue/${issueKey}/comment/${commentId}`, {
                method: 'PUT',
                body: {
                    body: buildAdfFromText(text)
                }
            })
            console.log('SUCCESS')
            console.log(`Comment ID: ${commentId}`)
        }
    },

    'delete-comment': {
        usage: '<issue_key> <comment_id>',
        desc: 'Delete comment by ID',
        async run(issueKey, commentId) {
            if (!issueKey || !commentId) die('issue_key, comment_id required')

            await jiraApi(`issue/${issueKey}/comment/${commentId}`, { method: 'DELETE' })
            console.log('SUCCESS')
            console.log(`Deleted comment ID: ${commentId}`)
        }
    },

    'add-worklog': {
        usage: '<issue_key> <time_spent> [comment]',
        desc: 'Log work (e.g. "2h", "30m", "4d")',
        async run(issueKey, timeSpent, comment) {
            if (!issueKey || !timeSpent) die('issue_key, time_spent required')

            const payload = { timeSpent }
            if (comment) {
                payload.comment = buildAdfFromText(comment)
            }

            const data = await jiraApi(`issue/${issueKey}/worklog`, {
                method: 'POST',
                body: payload
            })
            console.log('SUCCESS')
            console.log(`Worklog ID: ${data.id}`)
            console.log(`Time spent: ${data.timeSpent}`)
        }
    },

    'list-projects': {
        usage: '[max_results] [search_string]',
        desc: 'List visible projects',
        async run(maxResults = '50', search) {
            let url = `project/search?maxResults=${maxResults}`
            if (search) url += `&query=${encodeURIComponent(search)}`
            const data = await jiraApi(url)

            const projects = data.values || []
            console.log(`Projects: ${projects.length}`)
            console.log('---')
            for (const p of projects) {
                console.log(`Key: ${p.key}`)
                console.log(`Name: ${p.name}`)
                console.log(`ID: ${p.id}`)
                console.log(`URL: ${baseUrl}/browse/${p.key}`)
                console.log('---')
            }
        }
    },

    'issue-types': {
        usage: '[project_key]',
        desc: 'Get issue types for project (defaults to JIRA_PROJECT_KEY from .cursor/.project/.env)',
        async run(projectKey = PROJECT_KEY) {
            if (!projectKey) die('project_key required (or set JIRA_PROJECT_KEY in .cursor/.project/.env)')
            const data = await jiraApi(`issue/createmeta/${projectKey}/issuetypes`)
            const types = data.issueTypes || data.values || []

            console.log(`Issue types for ${projectKey}:`)
            console.log('---')
            for (const t of types) {
                console.log(`ID: ${t.id}`)
                console.log(`Name: ${t.name}`)
                console.log(`Subtask: ${t.subtask || false}`)
                console.log('---')
            }
        }
    },

    'lookup-user': {
        usage: '<search_string>',
        desc: 'Find user by name or email',
        async run(search) {
            if (!search) die('search_string required')
            const data = await jiraApi(
                `user/search?query=${encodeURIComponent(search)}&maxResults=10`
            )

            console.log(`Users found: ${data.length}`)
            console.log('---')
            for (const u of data) {
                console.log(`Account ID: ${u.accountId}`)
                console.log(`Name: ${u.displayName}`)
                console.log(`Email: ${u.emailAddress || 'N/A'}`)
                console.log(`Active: ${u.active}`)
                console.log('---')
            }
        }
    },

    'remote-links': {
        usage: '<issue_key>',
        desc: 'Get remote links for issue',
        async run(issueKey) {
            if (!issueKey) die('issue_key required')
            const data = await jiraApi(`issue/${issueKey}/remotelink`)

            console.log(`Remote links: ${data.length}`)
            console.log('---')
            for (const l of data) {
                console.log(`ID: ${l.id}`)
                console.log(`Title: ${l.object?.title || 'N/A'}`)
                console.log(`URL: ${l.object?.url || 'N/A'}`)
                console.log('---')
            }
        }
    },

    'add-remote-link': {
        usage: '<issue_key> <url> <title> [--confluence --page-id=<id>]',
        desc: 'Add a remote link to an issue. Plain by default; with --confluence --page-id=<id> (and CONFLUENCE_APP_ID set) it creates a first-class Confluence-content link (surfaced by the page Jira Links button + issuesWithRemoteLinksByGlobalId JQL).',
        async run(issueKey, url, title, ...rest) {
            if (!issueKey || !url || !title) die('issue_key, url, and title required')
            const flags = parseFlags(rest)
            let body
            if (flags.confluence) {
                const pageId = flags['page-id']
                if (!pageId) die('--page-id=<id> required with --confluence')
                if (!CONFLUENCE_APP_ID) die('CONFLUENCE_APP_ID must be set (run: node jira.mjs link-config <linked-issue>)')
                body = buildConfluenceRemoteLink({ appId: CONFLUENCE_APP_ID, pageId, url, title })
            } else {
                body = { object: { url, title } }
            }
            const data = await jiraApi(`issue/${issueKey}/remotelink`, { method: 'POST', body })
            console.log(`Remote link added to ${issueKey}: ${data?.id ?? '(created)'}`)
        }
    },

    'link-config': {
        usage: '[sample_issue_key]',
        desc: 'Discover the site cloudId + Confluence appId needed for "Linked Jira Tickets" (the page Jira Issues macro and the create-stories back-links). Pass an issue already linked to a Confluence page for the most reliable appId. Prints ready-to-paste .env lines; does not write .env.',
        async run(sampleIssueKey) {
            let cloudId = ''
            try {
                const res = await fetch(`${baseUrl}/_edge/tenant_info`, {
                    headers: { Authorization: AUTH, Accept: 'application/json' }
                })
                if (res.ok) cloudId = (await res.json())?.cloudId || ''
            } catch { /* best-effort */ }

            // Primary: read the appId out of a sample issue's Confluence remote link.
            let appId = ''
            let appIdSource = ''
            if (sampleIssueKey) {
                const links = await jiraApi(`issue/${sampleIssueKey}/remotelink`).catch(() => [])
                for (const l of Array.isArray(links) ? links : []) {
                    const gm = /appId=([^&"]+)/.exec(l.globalId || '')
                    if (gm) { appId = gm[1]; appIdSource = `remote link on ${sampleIssueKey}`; break }
                }
            }
            // Fallback: the Jira→Confluence application link (needs admin).
            if (!appId) {
                try {
                    const res = await fetch(`${baseUrl}/rest/applinks/2.0/listApplicationlinks`, {
                        headers: { Authorization: AUTH, Accept: 'application/json' }
                    })
                    if (res.ok) {
                        const body = await res.json()
                        const list = body?.list || body?.applicationLinks || []
                        const conf = list.find((x) => (x.application?.typeId || x.typeId) === 'confluence')
                        appId = conf?.application?.id || conf?.id || ''
                        if (appId) appIdSource = 'listApplicationlinks'
                    }
                } catch { /* best-effort */ }
            }

            console.log('Linked Jira Tickets — discovered config:')
            console.log(`  cloudId: ${cloudId || '(not found)'}`)
            console.log(`  Confluence appId: ${appId ? `${appId} (${appIdSource})` : '(not found — pass an issue already linked to a Confluence page)'}`)
            console.log('---')
            console.log('Add to .cursor/.project/.env:')
            if (appId) console.log(`CONFLUENCE_APP_ID=${appId}`)
            if (cloudId) console.log(`CONFLUENCE_CLOUD_ID=${cloudId}`)
            if (!appId || !cloudId) {
                console.log('---')
                console.log('NOTE — for a missing appId, link any Story to the FSD/ISD page once (Jira: Link → Confluence page), then re-run link-config with that issue key.')
            }
        }
    },

    'delete-remote-link': {
        usage: '<issue_key> <link_id>',
        desc: 'Delete a remote link from an issue',
        async run(issueKey, linkId) {
            if (!issueKey || !linkId) die('issue_key and link_id required')
            await jiraApi(`issue/${issueKey}/remotelink/${linkId}`, { method: 'DELETE' })
            console.log(`Remote link ${linkId} deleted from ${issueKey}`)
        }
    },

    'upload-attachment': {
        usage: '<issue_key> <file_path>',
        desc: 'Upload a file as an attachment to an issue',
        async run(issueKey, filePath) {
            if (!issueKey || !filePath) die('issue_key and file_path required')
            const att = await uploadAttachment(issueKey, filePath)
            console.log('SUCCESS')
            console.log(`Attachment ID: ${att.id}`)
            console.log(`Filename: ${att.filename}`)
            console.log(`Size: ${att.size}`)
            console.log(`URL: ${baseUrl}/browse/${issueKey}`)
        }
    }
}

function extractAdfText(node) {
    if (!node) return ''
    let text = node.text || ''
    if (node.content) {
        for (const child of node.content) text += extractAdfText(child)
    }
    return text
}

function tryReadFile(input) {
    try {
        return readFileSync(input, 'utf8')
    } catch {
        return input
    }
}

function parseFlags(args) {
    const flags = {}
    for (const arg of args) {
        if (typeof arg !== 'string' || !arg.startsWith('--')) continue
        const eqIdx = arg.indexOf('=')
        if (eqIdx === -1) {
            flags[arg.slice(2)] = true
        } else {
            flags[arg.slice(2, eqIdx)] = arg.slice(eqIdx + 1)
        }
    }
    return flags
}

function computeIdemHash(summary, descText) {
    // Normalise to make whitespace/case-insensitive within a project: trim, collapse whitespace, lowercase.
    // Use first 800 chars of body so unrelated edits late in the description don't trigger a new ticket.
    const normSummary = summary.replace(/\s+/g, ' ').trim().toLowerCase()
    const normDesc = (descText || '').replace(/\s+/g, ' ').trim().slice(0, 800).toLowerCase()
    return createHash('sha256').update(`${normSummary}\n${normDesc}`).digest('hex').slice(0, 12)
}

function dumpFailedPayload(fields, errorMessage) {
    try {
        mkdirSync(TMP_DIR, { recursive: true })
        const dump = {
            error: errorMessage,
            timestamp: new Date().toISOString(),
            endpoint: 'POST /rest/api/3/issue',
            fields
        }
        writeFileSync(FAILED_PAYLOAD_PATH, JSON.stringify(dump, null, 2))
    } catch (writeErr) {
        // Don't mask the original error if the dump itself failed.
        console.warn(`Warning: failed to dump payload — ${writeErr.message}`)
    }
}

async function uploadAttachment(issueKey, filePath) {
    const fileBuffer = readFileSync(filePath)
    const fileName = basename(filePath)
    const form = new FormData()
    form.append('file', new Blob([fileBuffer]), fileName)

    const res = await fetch(`${baseUrl}/rest/api/3/issue/${issueKey}/attachments`, {
        method: 'POST',
        headers: {
            Authorization: AUTH,
            Accept: 'application/json',
            'X-Atlassian-Token': 'no-check'
        },
        body: form
    })

    if (!res.ok) {
        const errData = await res.json().catch(() => null)
        const msg = errData?.errorMessages?.join(', ') || res.statusText
        throw new Error(`HTTP ${res.status}: ${msg}`)
    }

    const attachments = await res.json()
    if (!Array.isArray(attachments) || attachments.length === 0) {
        throw new Error('Upload succeeded but API returned no attachment metadata')
    }
    return attachments[0]
}

const [command, ...args] = process.argv.slice(2)

const wantsHelp =
    !command ||
    command === 'help' ||
    command === '--help' ||
    command === '-h' ||
    !COMMANDS[command]

if (wantsHelp) {
    console.log('Usage: node jira.mjs <command> [args]\n')
    console.log('Commands:')
    for (const [name, { usage, desc }] of Object.entries(COMMANDS)) {
        console.log(`  ${name} ${usage}`)
        console.log(`    ${desc}\n`)
    }
    process.exit(command && command !== 'help' && command !== '--help' && command !== '-h' ? 1 : 0)
}

if (args.includes('--help') || args.includes('-h')) {
    const { usage, desc } = COMMANDS[command]
    console.log(`Usage: node jira.mjs ${command} ${usage}`)
    console.log(`  ${desc}\n`)
    process.exit(0)
}

try {
    await COMMANDS[command].run(...args)
} catch (err) {
    console.error(`ERROR: ${err.message}`)
    process.exit(1)
}
