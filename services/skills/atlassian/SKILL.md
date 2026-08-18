# Atlassian (Jira + Confluence)

## Key Constraints (read first)

1. **Jira & Confluence API** via scripts in `services/skills/atlassian/scripts/` (native fetch + API token).
2. **No Atlassian MCP** for Jira/Confluence — use scripts only (stable, no OAuth expiration).
3. **Credentials/config** are loaded by the scripts from environment variables / the project `.env`.
   Report missing key names only; never print credential values.
4. **Project config** — runtime values (`JIRA_PROJECT_KEY`, `JIRA_BOARD_ID`, `CONFLUENCE_SPACE_KEY`, `ATLASSIAN_*`) come from the environment; ask the user for project data (key, board, links, statuses/transitions) when it is not configured

## Jira Decision Tree

Run from repo root: `node services/skills/atlassian/scripts/jira.mjs <command>`

```
User wants to...
├── GET issue + comments     → jira.mjs get-issue <key>        ← comments included automatically
├── GET comments only        → jira.mjs get-comments <key>
├── VIEW screenshots/images → jira.mjs get-attachments <key>  ← then Read each printed path
├── CREATE issue            → jira.mjs create-issue <type> <summary> [desc] [priority]
├── CREATE Story            → jira.mjs create-story --summary=<title> --desc-file=<file> [--epic=<key>]
├── CREATE Stories (bulk)   → jira.mjs create-stories <stories.md> [--epic=<key>] [--assignee=FE|BE|<id>] [--dry-run] [--refresh]  ← whole stories.md, backfills keys/links
├── UPDATE fields           → jira.mjs edit-issue <key> <fields_json>
├── TRANSITION              → jira.mjs transition <key> [target_status]
├── ADD TO SPRINT           → jira.mjs add-to-sprint <key> [board_id]
├── SEARCH                  → jira.mjs search '<jql>'
├── ADD comment             → jira.mjs add-comment <key> <body>
├── LOG work                → jira.mjs add-worklog <key> <time>
├── LIST projects           → jira.mjs list-projects
├── ISSUE types             → jira.mjs issue-types [project]
├── FIND user               → jira.mjs lookup-user <name>
└── REMOTE links            → jira.mjs remote-links <key>
```

**Creating stories in bulk:** to create a whole backlog from a deterministic
`stories.md` file, prefer `create-stories <stories.md>`
— it parses every `### <ID> — <title>` block itself (summary = heading, description = block body verbatim),
creates/reuses issues by idempotency hash, and backfills the keys + `- Jira:` links into `stories.md`. No
temp description files. Preview with `--dry-run`; re-stamp links for already-created issues with `--refresh`.
Use the single-block `create-story` (with `--desc-file`) only for one-off Story creation outside that flow.
For either, ask whether an Epic parent is required and pass `--epic=<key>` only when the user provides one.
Assignee defaults to the API token user; both commands pre-flight that the resolved account is assignable
before any write, so a project that marks assignee required never fails mid-run. Override the default with
`--assignee=FE|BE` (via `JIRA_ASSIGNEE_MAP`) or a raw accountId when a different owner is needed.

`jira.mjs add-comment` converts triple-backtick fences with an optional language into Jira ADF `codeBlock` nodes. Use fences for multi-line code/log snippets; use backticks or `{{text}}` only for inline code.

Markdown pipe tables (`| col | col |` with a `|---|---|` separator row) convert to ADF `table` nodes. See `references/jira.md` → Comment Formatting Rules.

### Researching a Jira Ticket

To understand a ticket's scope, acceptance criteria, decisions, history, and related discussion:

1. **`get-issue <key>`** — current status, description, all comments, assignee, and the attachment list.
2. **`search '<jql>'`** and **`remote-links <key>`** — related tickets and linked Confluence requirements pages; read the linked pages with `confluence.mjs get-page`.
3. **`get-attachments <key>`** — when you need visual context; downloads images to a temp directory and prints their paths, then Read each printed path.

**Read every comment carefully.** Comments contain QA acceptance notes, architecture decisions, scope changes, blockers — critical context missing from the description.

### Viewing Jira Attachments (Screenshots)

When the user asks to analyse visuals or mentions attachments on a ticket, run `get-attachments <key>` immediately. Do NOT ask for confirmation — it is a read-only operation.

When the user asks to analyse a bug visually, review screenshots, or mentions "look at attachments":

1. Run `get-issue <key>` for text context
2. Run `get-attachments <key>` — downloads images to a temp directory, prints absolute paths
3. Read each printed path using the Read tool — images load directly into context
4. Files are cleaned automatically at the start of the next `get-attachments` run

**Notes:**

- Only image attachments are downloaded (`image/*` MIME types)
- The download directory is outside the source tree — files never appear in git changes
- The folder is created automatically on first run — no setup needed

For commands reference, JQL examples, workflows, and project config see `references/jira.md`.

## Confluence Decision Tree

Run from repo root: `node services/skills/atlassian/scripts/confluence.mjs <command>`

```
User wants to...
├── READ page    → confluence.mjs get-page <id> [storage|adf]
├── CREATE page  → confluence.mjs create-page <space_id> <title> <body>
├── UPDATE page  → see "Safe Editing Workflow" in references/confluence.md
├── COPY page    → confluence.mjs copy-page <source_id> <parent_id> [title]
├── DELETE page  → confluence.mjs delete-page <id>
├── LIST attachments → confluence.mjs list-attachments <page_id>
├── UPLOAD file  → confluence.mjs upload-attachment <page_id> <file_path> [comment]
├── UPLOAD/UPDATE file → confluence.mjs upload-attachment-update <page_id> <file_path> [comment]
├── PULL diagrams → confluence.mjs pull-diagrams <page_id> --into <doc.md>  (image + .drawio round-trip; export-confluence does this automatically per page)
├── SEARCH       → confluence.mjs search '<cql>'
├── LIST spaces  → confluence.mjs list-spaces
├── LIST pages   → confluence.mjs list-pages <space_id>
├── LIST children→ confluence.mjs list-children <page_id>
├── APPROVAL MATRIX → confluence.mjs doc-matrix --type <fsd|isd> [--dry]  (manual/dry-run only — a typed publish refreshes the parent's matrix automatically)
├── READ comments→ confluence.mjs comments <page_id> [footer|inline]
└── ADD comment  → confluence.mjs add-comment <page_id> <body> [footer|inline]
```

For commands reference, CQL examples, safe editing, content format rules, and macros see `references/confluence.md`.

**Structured markdown publishing:** use `scripts/publish.mjs` when a workflow needs markdown converted to
Confluence storage with a document profile, attachments, @mentions, and inline-comment preservation.
Publish an FSD/ISD page only when the user explicitly asks for it.

```bash
# Create under the profile's configured parent, or an explicit parent when provided
# (--comment is REQUIRED for a typed publish — it becomes the Confluence version-history message):
node services/skills/atlassian/scripts/publish.mjs <doc.md> \
  --type=<profile> --title="<title>" --comment="<msg>" --mention="<Name>=<accountId>"

# Update in place:
node services/skills/atlassian/scripts/publish.mjs <doc.md> \
  --type=<profile> --page-id=<ID> --comment="<msg>" --mention="<Name>=<accountId>" --skip-attachments

# Generic markdown without a document profile:
node services/skills/atlassian/scripts/publish.mjs <doc.md> \
  --parent=<id> --title="<title>"

# Dry run:
node services/skills/atlassian/scripts/publish.mjs <doc.md> --type=<profile> --title="preview" --dry
```

Use `jira.mjs lookup-user <name>` for account IDs used in `--mention`.

**Approval matrix — automatic.** A typed publish wraps the header table in the native Content Properties
macro, which makes column 1 a property key and column 2 its value (that is why an approval row is authored
`role | status | name`), and then refreshes ONE Page Properties Report on the **parent** page. So a new
FSD/ISD joins the matrix the moment it is published and nobody has to run a follow-up step; the report
cannot drift from the documents. Columns run document, status, author, the approver roles, then
package/batch — the remaining card fields (WBS, project name, Jira reference) are left out. The roles come
from the documents, so no role list or client vocabulary is configured anywhere.

- Scope is the `fsd`/`isd` label plus `ancestor = <parent>`, so a matrix only aggregates its own subtree.
- Nothing is written when the rebuilt report is identical, so re-publishing does not churn the parent's
  version history. A failure never fails the publish (it prints a NOTE).
- A page published before this wrapper existed contributes nothing until it is republished.
- `--skip-matrix` on `publish.mjs` leaves the parent alone; `confluence.mjs doc-matrix --type <fsd|isd>
  [--dry]` is the manual entry point for previewing columns or repairing a parent page by hand.

**Bulk export (Confluence → markdown):** `scripts/export-confluence-to-markdown.mjs`
walks a page subtree to markdown. Two modes, one command:

```bash
# Generic render dump (default) — good for a raw markdown mirror:
node services/skills/atlassian/scripts/export-confluence-to-markdown.mjs <root_id> --out <dir>

# Typed reverse pull — reconstruct canonical authored FSD/ISD markdown from a
# published page's STORAGE (strips wiki chrome; inverse of publish). Mirrors
# publish.mjs's --type. Pages that don't parse as the type fall back to the
# generic dump for that page (with a WARN):
node services/skills/atlassian/scripts/export-confluence-to-markdown.mjs <root_id> --type=isd
```

Diagrams round-trip automatically in both modes (image + `.drawio` → editable
` ```drawio ` block). See `scripts/ARCHITECTURE.md` → *The type-aware reverse pull*.

Assets round-trip too. `get-page --type --assets-dir <dir>` localizes body images +
diagram sources into `<dir>` and KEEPS them (discovery writes the working copy's
`./assets`); omit the flag and they go to a throwaway temp dir (a baseline pull that
never clobbers `./assets`). On publish, referenced assets sync hash-gated (unchanged
skipped) and **managed** orphans (our `sha256:`-gated attachments no longer
referenced — a rename is remove+add) are deleted, while attachments we do not own
are only warned. A referenced file missing on disk is skipped with a WARN, never
crashing the publish. See `scripts/ARCHITECTURE.md` → *Asset round-trip*.

## Anti-Patterns (DO NOT)

- **DO NOT** use Atlassian MCP tools for Jira/Confluence — OAuth expiration issues
- **DO NOT** hardcode account IDs — use `lookup-user` first
- **DO NOT** guess Jira transitions — use `transition` without status to list them
- **DO NOT** create Jira tickets without checking for duplicates — search first
- **DO NOT** update Confluence pages with rich elements using markdown without warning
- **DO NOT** pass raw markdown syntax as Confluence storage format body — always use XHTML
- **DO NOT** create Confluence comments without sharing the anchor link in chat
