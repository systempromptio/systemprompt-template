# Jira Reference

For project-specific config: runtime values (project key, board, credentials) come from the environment / project `.env`; ask the user for project data that is not configured.

## Commands Reference

Run from repo root: `node services/skills/atlassian/scripts/jira.mjs <command>`

### Core CRUD

| Command | Args | Description |
|---------|------|-------------|
| `get-issue` | `<issue_key> [fields]` | Get issue details |
| `create-issue` | `<type> <summary> [desc] [priority] [assignee_id] [parent]` | Create issue (desc: text file or ADF .json) |
| `create-story` | `--summary=<...> --desc-file=<...> [--priority=Major] [--assignee=FE\|BE\|<id>] [--epic=<key>] [--labels=csv] [--dedup=<keywords>] [--dry-run]` | Create ONE Story from a single `stories.md` block with preflight, idempotency, optional epic parent, and dry-run |
| `create-stories` | `<stories.md> [--epic=<key>] [--priority=Major] [--assignee=FE\|BE\|<id>] [--dry-run] [--refresh]` | Create/reuse ALL Stories from a `stories.md` (parses each block itself; idempotency via an `_idem: <hash>_` marker in the description; assignee defaults to the API token user; backfills heading keys + `- Jira:` links into the file). `--dry-run` = read-only preview; `--refresh` = re-stamp links only |
| `edit-issue` | `<issue_key> <fields_json>` | Update fields (JSON or file). For description use ADF. |
| `transition` | `<issue_key> [target_status]` | Transition. Without status — lists options |
| `add-to-sprint` | `<issue_key> [board_id]` | Move issue to active sprint (default board from `JIRA_BOARD_ID`) |

### Search & Browse

| Command | Args | Description |
|---------|------|-------------|
| `search` | `<jql> [max] [fields]` | JQL search |
| `list-projects` | `[max] [search]` | List projects |
| `issue-types` | `[project_key]` | Issue types (default: from JIRA_PROJECT_KEY env) |
| `lookup-user` | `<search>` | Find user by name/email |
| `remote-links` | `<issue_key>` | Remote links |

### Description format (create-issue / edit-issue)

- **Text file** (e.g. `desc.txt`): converted to ADF. Use `#`, `##`, `###` for headings and `- item` for bullet lists. Prefer "what to do" (action items) for PM clarity.
- **JSON file** (`.json`): full ADF document (`type: "doc", version: 1, content: [...]`). Use for ordered lists, panels, or complex layout. Example: `edit-issue PROJ-XXX fields.json` where file contains `{"description": { "type": "doc", "version": 1, "content": [...] }}`.

### Creating Stories from `stories.md`

Use this contract when creating Jira Story issues from an approved `stories.md` backlog
file. Prefer `create-stories` — it parses the file, creates/reuses, and backfills
in one pass; there are no hand-off temp description files.

MUST:
- Use only existing story blocks from `stories.md`. Do not rewrite, expand, or invent story content.
- `create-stories` reads each `### <STORY-ID> — <title>` block itself: Summary = `<title>`; description =
  the block body verbatim (user story, the acceptance-criteria LINK, implementation notes, references).
  Note acceptance criteria is a single Confluence anchor link to the FSD's `#### Acceptance Criteria`
  section — not copied criteria text.
- Ask the user whether these stories need an Epic parent. If yes, require the Epic key and pass
  `--epic=<ISSUE-KEY>`. If no, omit `--epic`.
- Assignee defaults to the API token user (`myself`). Override with `--assignee=FE|BE` (from
  `JIRA_ASSIGNEE_MAP`) or a raw accountId. Pre-flight validates the resolved account is assignable to the
  project before any write (also under `--dry-run`), so a required-assignee project never 400s mid-run.
- Run `create-stories <stories.md> --dry-run` before real creation. Treat dry-run failures as blocking.
- Creation is idempotent (an `_idem: <hash>_` marker embedded in each Story description): re-runs reuse
  existing issues instead of duplicating. On success
  the command backfills `stories.md` itself — heading `TBD-n` -> real key, a
  `Jira tickets created: <keys>` line above `## Stories`, and a
  `- Jira: [<KEY>](<browse-url>)` reference per story. Use `--refresh` (search-only) to re-stamp links for
  already-created issues.
- Use single-block `create-story` (with `--desc-file`) only for one-off Story creation outside this flow.

DO NOT:
- Do not create Jira Stories from local `tasks.md`, specs, FSD/ISD prose, or agent summaries.
- Do not create a new Epic automatically.
- Do not add a Story to a sprint unless the user explicitly asks after the Story exists.
- Do not hand-edit `stories.md` keys/links — the script owns that backfill.
- Do not retry-loop failed POST requests. Diagnose with read-only calls and inspect the
  `last-jira-payload.json` dump the script prints the path to on failure.

Example:

```bash
# Preview, then create the whole backlog and backfill keys/links into stories.md:
node services/skills/atlassian/scripts/jira.mjs create-stories \
  path/to/add-store-locator/stories.md --epic="SFPA-123" --dry-run

node services/skills/atlassian/scripts/jira.mjs create-stories \
  path/to/add-store-locator/stories.md --epic="SFPA-123"
```

### Doc ↔ Story linking

Stories and their FSD/ISD Confluence page are linked bidirectionally so the doc footer can list its
Stories and each Story points back at the doc. `create-stories` does this automatically; the commands
below are the setup + manual counterparts.

| Command | Args | Description |
|---------|------|-------------|
| `link-config` | `[sample_issue_key]` | Discover + print the `.env` lines needed for linking (`CONFLUENCE_APP_ID`, `CONFLUENCE_CLOUD_ID`). Pass an issue already linked to a Confluence page for the most reliable appId. Prints only; does not write `.env`. |
| `add-remote-link` | `<issue_key> <url> <title> [--confluence --page-id=<id>]` | Add a remote link. Plain web link by default; with `--confluence --page-id` (and `CONFLUENCE_APP_ID` set) it creates a first-class Confluence-content link (surfaced by the page's Jira Links button + the footer JQL). |
| `delete-remote-link` | `<issue_key> <link_id>` | Remove a remote link (ids from `remote-links`). |

**How it flows (do not break this):**

- `create-stories` back-links each created Story to its FSD/ISD page **by default** (opt out with
  `--no-link`). It parses the page URL/id from the story block's Acceptance-criteria/FSD reference and
  POSTs a Confluence-typed remote link with `globalId=appId=<..>&pageId=<..>` (an idempotent upsert). A
  missing `CONFLUENCE_APP_ID` or an unpublished doc (no page id yet) is **skipped, not fatal** — the
  command prints a hint to run `link-config`.
- The doc footer **"Linked Jira Tickets"** is driven by those links: `publish.mjs` → `doc/render.mjs`
  builds the JQL `issuesWithRemoteLinksByGlobalId("appId=<..>&pageId=<..>")`, and the
  `templates/confluence` Jira Issues macro renders the matching Stories. So the footer only populates
  once the Stories carry the Confluence-typed remote link, which is exactly what `create-stories` adds.

Config: `CONFLUENCE_APP_ID` / `CONFLUENCE_CLOUD_ID` in the project `.env` (discover via `link-config`).

### Comments & Worklog

| Command | Args | Description |
|---------|------|-------------|
| `add-comment` | `<issue_key> <body>` | Add comment (text or file). Same formatting as descriptions: `#`/`##`/`###` headings, `- item` bullets, links, fenced code blocks. |
| `edit-comment` | `<issue_key> <comment_id> <body>` | Edit comment (text or file). Same formatting support. |
| `delete-comment` | `<issue_key> <comment_id>` | Delete comment by ID |
| `add-worklog` | `<issue_key> <time> [comment]` | Log work ("2h", "30m", "4d") |

### Comment Formatting Rules

Comments go through `buildAdfFromText()` (in `lib/adf.mjs`) which converts text to ADF. The parser uses **simplified Markdown-like** syntax (NOT Jira Wiki Markup):

| Element | Syntax | Gotcha |
|---------|--------|--------|
| Heading 1 | `# Title` | Single `#` = h1 (NOT ordered list!) |
| Heading 2 | `## Title` | |
| Heading 3 | `### Title` | |
| Bold | `*text*` or `**text**` | Both work |
| Monospace | `{{code}}` or `` `code` `` | Both work |
| Code block | Triple backtick fences, optionally with language | Converts to Jira ADF `codeBlock` |
| Bullet list | `- item` | One item per line |
| Table | Pipe rows + separator | Markdown pipe syntax; first row is header when followed by `\|---\|` |
| Ordered list | `1. item` / `2. item` | Use numeric prefix, NOT `#` |
| Wiki link | `[title\|url]` | Pipe separator |
| Markdown link | `[title](url)` | Also supported |
| Color | `{color:red}text{color}` | Maps red/green to hex |

**CRITICAL:** `# text` is a HEADING, not an ordered list item. For numbered lists use `1.` `2.` `3.` prefix.

**Correct ordered list:**
```
1. First item
2. Second item
3. Third item
```

**WRONG** (renders as three h1 headings):
```
# First item
# Second item
# Third item
```

**Table** (pipe markdown — NOT wiki markup):

```
| Source | What it says |
|--------|--------------|
| FSD | No URL in AC |
| Code | Three segments |
```

- Each row must start with `|`. Include a separator row (`|---|---|`) so the first row becomes `tableHeader` cells.
- Inline formatting (`**bold**`, `` `code` ``) works inside cells.
- Without a separator row, all rows render as `tableCell` (no header styling).

## Common JQL

Replace `PROJECT` with your Jira project key (`JIRA_PROJECT_KEY` in the project `.env`).

```bash
node services/skills/atlassian/scripts/jira.mjs search 'project = PROJECT AND status != Closed'
node services/skills/atlassian/scripts/jira.mjs search 'project = PROJECT AND assignee = currentUser()'
node services/skills/atlassian/scripts/jira.mjs search 'project = PROJECT AND status = "In Progress"'
node services/skills/atlassian/scripts/jira.mjs search 'project = PROJECT AND priority IN (Blocker, Critical, Major)'
node services/skills/atlassian/scripts/jira.mjs search 'project = PROJECT AND updated >= -7d ORDER BY updated DESC' 20
```

## Common Workflows

### Create and assign
```bash
node services/skills/atlassian/scripts/jira.mjs lookup-user "Jane Doe"
node services/skills/atlassian/scripts/jira.mjs create-issue Task "Fix login" "Description" Major <accountId>
```

### Move through pipeline
```bash
node services/skills/atlassian/scripts/jira.mjs transition PROJ-123                # list transitions
node services/skills/atlassian/scripts/jira.mjs transition PROJ-123 "In Progress"  # transition
```

### Update fields
```bash
node services/skills/atlassian/scripts/jira.mjs edit-issue PROJ-123 '{"priority":{"name":"Critical"}}'
node services/skills/atlassian/scripts/jira.mjs edit-issue PROJ-123 '{"assignee":{"accountId":"<id>"}}'
```
