#!/usr/bin/env node
/**
 * Publish / update a markdown document to Confluence.
 *
 * TYPED mode (--type=fsd|isd):
 *   - Applies the doc-type profile: status badges, content-appearance (Narrow/full-width).
 *   - Renders the FSD/ISD chrome (General FSD/ISD Information card, approval roster, References, TOC,
 *     status lozenges, resolved @mentions, Document Change Log footer) BY DEFAULT — a typed publish
 *     implies `--render=template`. Pass `--render=markdown` to opt out (plain conversion).
 *   - Enforces a self-describing page title: the created page's title always carries the doc-type
 *     token ("… FSD" / "… ISD"); when the title/h1 omits it, the token is appended.
 *   - Parent page resolves from CONFLUENCE_<TYPE>_PARENT_ID in .env, or --parent override.
 *
 * UNTYPED / generic mode (no --type):
 *   - Publishes any markdown file as plain converted storage XHTML; no macros applied.
 *   - --parent is required when creating a new page.
 *
 * Usage:
 *   # Typed — first publish (create):
 *   node publish.mjs <doc.md> --type=fsd --title="<title>" --comment="<msg>" [--parent=<id>] [--mention="Name=id"] [--dry]
 *
 *   # Typed — update in place:
 *   node publish.mjs <doc.md> --type=fsd --page-id=<id> --comment="<msg>" [--mention="Name=id"] [--skip-attachments] [--dry]
 *
 *   # Untyped — create any page:
 *   node publish.mjs <doc.md> --parent=<id> --title="<title>" [--mention="Name=id"] [--dry]
 *
 *   # Untyped — update any existing page:
 *   node publish.mjs <doc.md> --page-id=<id> [--mention="Name=id"] [--skip-attachments] [--dry]
 *
 * Flags:
 *   --type=<type>            doc-type profile (fsd, isd). Optional.
 *   --page-id=<id>           update an existing Confluence page
 *   --title="<title>"        page title (required on create)
 *   --parent=<id>            parent page ID override (or env fallback for typed)
 *   --mention="Name=id"      @mention mapping; repeat for multiple
 *   --skip-attachments       skip image attachment uploads
 *   --skip-validation        bypass the format canon check (template mode only)
 *   --skip-matrix            leave the parent page's approval matrix alone (a typed
 *                            publish otherwise refreshes it — see lib/doc/matrix.mjs)
 *   --comment="<msg>"        Confluence version-history message for this update (<=50 chars).
 *                            REQUIRED for a typed publish (unless --dry); for an
 *                            untyped publish it falls back to the doc-type profile message.
 *   --dry                    convert only, write preview HTML, no API calls
 *   --space=<key>            Confluence space key override (default: CONFLUENCE_SPACE_KEY)
 *   --render=template|markdown
 *                            render canonical FSD/ISD markdown via the Nunjucks
 *                            chrome (General FSD/ISD Information card, approvals, references, status
 *                            lozenges, resolved @mentions) as plain wiki-style
 *                            tables. Enforces the format canon (required sections)
 *                            unless --skip-validation. Title defaults from the
 *                            document's h1. Defaults to `template` for a typed
 *                            publish (--type set) and `markdown` otherwise; pass
 *                            `--render=markdown` to force plain conversion.
 */
import { resolve } from 'node:path'
import { publishDoc } from './lib/doc/publish.mjs'
import { parseArgs } from './lib/util/cli-args.mjs'

const { flags, positional } = parseArgs(process.argv.slice(2), {
  booleans: ['skip-attachments', 'skip-validation', 'skip-matrix', 'dry'],
  repeatable: ['mention'],
})
const mdPath = positional[0]

function usage(msg) {
  if (msg) console.error(`ERROR: ${msg}\n`)
  console.error('Usage:')
  console.error('  # Typed (profile: TOC, status badges, Narrow page width). --comment required (unless --dry):')
  console.error('  node publish.mjs <doc.md> --type=fsd|isd --title="<t>" --comment="<msg>" [--parent=<id>] [--mention="Name=id"] [--dry]')
  console.error('  node publish.mjs <doc.md> --type=fsd|isd --page-id=<id> --comment="<msg>" [--mention="Name=id"] [--skip-attachments] [--dry]')
  console.error('')
  console.error('  # Untyped (generic, no macros):')
  console.error('  node publish.mjs <doc.md> --parent=<id> --title="<t>" [--mention="Name=id"] [--dry]')
  console.error('  node publish.mjs <doc.md> --page-id=<id> [--mention="Name=id"] [--skip-attachments] [--dry]')
  process.exit(1)
}

if (!mdPath) usage('A source markdown file is required as the first argument.')

// Parse --mention (repeatable)
const mentionMap = {}
for (const raw of [].concat(flags.mention || [])) {
  const str = String(raw)
  const eq = str.indexOf('=')
  if (eq > 0) mentionMap[str.slice(0, eq).trim()] = str.slice(eq + 1).trim()
}

const docType = flags.type ? String(flags.type) : undefined
const pageId = flags['page-id'] ? String(flags['page-id']) : undefined
const title = flags.title ? String(flags.title) : undefined
const parent = flags.parent ? String(flags.parent) : undefined
const skipAttachments = Boolean(flags['skip-attachments'])
const skipValidation = Boolean(flags['skip-validation'])
const skipMatrix = Boolean(flags['skip-matrix'])
const dry = Boolean(flags.dry)
const spaceKey = flags.space ? String(flags.space) : undefined
const comment = flags.comment != null ? String(flags.comment).trim() : undefined
if (comment !== undefined && comment.length > 50) {
  usage(`--comment must be <=50 characters (got ${comment.length}).`)
}
// A typed (FSD/ISD) publish must carry a human-readable version-history message so
// the Confluence page history stays reviewable and rollback is easy. --dry writes
// no version, so it is exempt (the preview pass runs without a comment).
if (docType && !dry && !comment) {
  usage('--comment="<msg>" is required for a typed publish (<=50 chars) — it becomes the Confluence version-history message.')
}
// Leave render undefined when the caller does not set it, so a typed publish can
// default to the template chrome (decided in confluence-publish). Only an explicit
// --render=markdown opts a typed publish out of the chrome.
const render = flags.render ? String(flags.render) : undefined

// A typed publish renders the template chrome by default; an untyped one is plain
// markdown unless --render=template is passed.
const willTemplate = render === 'template' || (Boolean(docType) && render !== 'markdown')

// In template mode the title defaults from the document's h1, so it is not required.
if (!pageId && !title && !willTemplate) {
  usage('--title is required when creating a new page (no --page-id).')
}

try {
  await publishDoc({
    mdPath: resolve(mdPath),
    docType,
    pageId,
    title,
    parent,
    mentionMap,
    render,
    skipAttachments,
    skipValidation,
    skipMatrix,
    dry,
    spaceKey,
    comment,
  })
} catch (err) {
  console.error(`ERROR: ${err.message}`)
  process.exit(1)
}
