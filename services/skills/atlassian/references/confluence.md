# Confluence Reference

## Key Info

- **Space key / IDs:** project-specific — `CONFLUENCE_SPACE_KEY` in `.env`. Cloud ID and numeric space/page IDs are per-instance; discover them from the page URL or the space settings, never hard-code.
- **Updating with markdown replaces ALL content** — destroys @mentions, status badges, dates, macros
- **ADF format preserves everything** — use it when page has rich elements

## Publishing Markdown Documents

Use `publish.mjs` when a workflow needs markdown converted to Confluence storage. A document profile can
add macros, badges, mentions, attachments, layout, and inline-comment preservation. Publish an FSD/ISD
page to Confluence only when the user explicitly asks; create the Jira stories and back-link them to the
published doc as a follow-up step when requested.

```bash
# Create with a document profile:
node services/skills/atlassian/scripts/publish.mjs <doc.md> \
  --type=<profile> --title="<title>" --mention="<Name>=<accountId>"

# Update in place:
node services/skills/atlassian/scripts/publish.mjs <doc.md> \
  --type=<profile> --page-id=<ID> --mention="<Name>=<accountId>" --skip-attachments

# Publish without a document profile:
node services/skills/atlassian/scripts/publish.mjs <doc.md> \
  --parent=<parent_page_id> --title="<title>"

# Preview storage output without an API write:
node services/skills/atlassian/scripts/publish.mjs <doc.md> \
  --type=<profile> --title="preview" --dry
```

## Commands Reference

Run from repo root: `node services/skills/atlassian/scripts/confluence.mjs <command>`

### Core CRUD

| Command | Args | Description |
|---------|------|-------------|
| `get-page` | `<page_id> [storage\|adf] [--type <fsd\|isd>] [--body-only] [--into <path>]` | Read page. Default `storage` returns raw storage XHTML; `adf` returns `atlas_doc_format` JSON. `--type=<fsd\|isd>` instead returns the page reversed to canonical authored markdown (the typed reverse pull — strips wiki chrome; use for a clean baseline/re-sync). `--body-only` for piping to file; `--into` writes a stamped working copy |
| `pull-diagrams` | `<page_id> --into <doc.md>` | Reverse pull: rebuild ` ```drawio:<type>:<id> ` blocks + `./assets` from the page's `.drawio` attachments |
| `create-page` | `<space_id> <title> <body_or_file> [parent_id] [format]` | Create page |
| `update-page` | `<page_id> <body_or_file> [format] [version_message]` | Update (replaces ALL body) |
| `delete-page` | `<page_id>` | Delete page |
| `copy-page` | `<source_id> <parent_id> [new_title]` | Copy (API v1, preserves everything) |
| `list-attachments` | `<page_id>` | List page attachments |
| `upload-attachment` | `<page_id> <file_path> [comment]` | Upload attachment (fails on duplicate filename) |
| `upload-attachment-update` | `<page_id> <file_path> [comment]` | Upload or update attachment by filename |

### Diagrams — images, never editable macros

Diagrams are NOT embedded as editable draw.io macros. The diagrams skill generates a
`.png` (rendered image) plus its `.drawio` source, and the publish pipeline uploads BOTH
as hash-gated attachments (`attachment-sync.mjs` via `diagram-publish.mjs`); the page body
references the `.png` as a normal `<ac:image>`. This protects diagrams from being edited
in Confluence and makes the round-trip authoritative: the `.drawio` companion is the
durable source `pull-diagrams` reads back. There is deliberately no `insert-drawio` command
and no draw.io macro anywhere in the pipeline.

### Reverse pull (Confluence -> markdown)

`pull-diagrams` reconstructs generated diagrams back into an authored doc. For each `.drawio`
attachment on the page it downloads the `.drawio` (+ its sibling `.png`) into `<doc-dir>/assets`,
decodes the embedded spec via the diagrams skill's `reverse.mjs`, and upserts the
` ```drawio:<type>:<id> ` block + image into `<doc.md>`. Remote wins: an existing block for the
same `id` is replaced in place (idempotent re-pull), so it never duplicates. Only the embedded
`data-spec` is honored — draw.io app geometry edits that don't update it are ignored.

```bash
node services/skills/atlassian/scripts/confluence.mjs pull-diagrams <page_id> --into path/to/doc.md
```

The bulk `export-confluence-to-markdown.mjs` shares this exact reconstruction (via the shared
`lib/diagrams/reconstruct.mjs`): any exported image backed by a `.drawio` attachment is pulled as an
editable ` ```drawio ` block in place of the flat image, and its `.png`/`.drawio` are saved under
their clean slug names (`<slug>.png` / `<slug>.drawio`, no URL-hash suffix) so the pair stays paired
and re-publishes idempotently through the hash-gated attachment sync. So a downloaded page comes back
fully re-editable — text, images, and diagrams (edit the block, regenerate the PNG, publish). Generic
(non-diagram) images keep a content-hash suffix for collision safety.

#### Typed reverse pull (`export --type=fsd|isd`)

`export-confluence-to-markdown.mjs` has two modes on the one command (mirroring `publish.mjs`'s
`--type`):

| Invocation | Body pipeline |
|------------|---------------|
| `export <root_id>` (no `--type`) | Generic `export_view` -> Turndown dump (unchanged default; good for a raw markdown mirror of the space) |
| `export <root_id> --type=fsd\|isd` | Typed reverse: fetch STORAGE, strip the wiki chrome the publish templates add (`<ac:layout>` wrapper, TOC macro, status-badge legend, structural `<hr/>`/`<p/>` dividers, Document Change Log footer), and rebuild the canonical authored markdown via `serializeDoc` |

The typed path reconstructs the header card + approval groups, the Reference Materials table, and the
body; it rehydrates typed values (status lozenges -> status words, `<ri:user account-id>` -> display
names via the user API, `<ac:image>` -> `![](./assets/<file>)`). Type selection is **explicit only** —
no label/title auto-detect. A page that isn't the given type (missing the type's header card) prints a
`WARN` and falls back to the generic dump for that page. Details in `scripts/ARCHITECTURE.md` ->
*The type-aware reverse pull*.

```bash
node services/skills/atlassian/scripts/export-confluence-to-markdown.mjs <root_id> --type=isd
```

### Search & Browse

| Command | Args | Description |
|---------|------|-------------|
| `search` | `<cql_query> [limit]` | CQL search |
| `list-spaces` | `[limit] [global\|personal]` | List spaces |
| `list-pages` | `<space_id> [limit] [sort] [title]` | Pages in space |
| `list-children` | `<page_id> [limit] [depth]` | Child pages |

### Comments

| Command | Args | Description |
|---------|------|-------------|
| `comments` | `<page_id> [footer\|inline] [limit]` | Read comments |
| `add-comment` | `<page_id> <body_or_file> [footer\|inline] [parent_comment_id]` | Add comment (ADF JSON required — string or .json file path) |
| `delete-comment` | `<comment_id> [footer\|inline]` | Delete a comment |

### ADF Manipulation

| Command | Args | Description |
|---------|------|-------------|
| `modify-table` | `<input.json> <output.json> <heading> <col_name> <notes_json>` | Add column to ADF table after heading |

## Safe Editing Workflow

**NEVER update a page with markdown if it contains @mentions, status badges, dates, or macros.**

### Step 1: Read and assess
```bash
node services/skills/atlassian/scripts/confluence.mjs get-page <page_id> storage
```
Check for `<custom data-type="mention"`, `<custom data-type="status"`, `<custom data-type="date"`. If present → rich elements.

### Step 2: Choose strategy

**No rich elements** → update with hand-built storage XHTML (or, preferably, `publish.mjs`):
```bash
node services/skills/atlassian/scripts/confluence.mjs update-page <id> <body_or_file> storage "Version message"
```

**Has rich elements** → use ADF:
```bash
node services/skills/atlassian/scripts/confluence.mjs get-page <id> adf --body-only > /tmp/body.json
# Modify JSON with node script
node services/skills/atlassian/scripts/confluence.mjs update-page <id> /tmp/body-modified.json adf "Version message"
```

## Common CQL Queries

```bash
node services/skills/atlassian/scripts/confluence.mjs search 'space = <SPACE_KEY> AND type = page'
node services/skills/atlassian/scripts/confluence.mjs search 'title ~ "meeting" AND type = page'
node services/skills/atlassian/scripts/confluence.mjs search 'space = <SPACE_KEY> AND type = page ORDER BY lastModified DESC' 10
```

## Comments Best Practices

1. **Always use ADF format** — `add-comment` expects ADF JSON (not HTML/markdown). Write the ADF to a temp `.json` file and pass the path.
2. **Always share a clickable anchor link** in chat after creating a comment using markdown link syntax:
   ```
   [Comment description](<ATLASSIAN_BASE_URL>/wiki/spaces/<SPACE_KEY>/pages/{page_id}?focusedCommentId={comment_id})
   ```
   The URL is printed by the script automatically. Wrap it in `[text](url)` so it's clickable in Cursor chat.
3. **Prefer inline comments** over footer comments — attach to the relevant section using `INLINE_TEXT_SELECTION` env var.
4. **Use @mentions** in ADF via `mention` node with the user's `accountId`:
   ```json
   { "type": "mention", "attrs": { "id": "<accountId>", "text": "@Name", "accessLevel": "" } }
   ```

### Inline Comment Example

```bash
INLINE_TEXT_SELECTION="Section Title" INLINE_MATCH_COUNT=1 INLINE_MATCH_INDEX=0 \
  node services/skills/atlassian/scripts/confluence.mjs add-comment <page_id> /tmp/comment.json inline
```

## Content Format Rules

### Always use Confluence Storage Format (XHTML), never raw markdown

When creating or updating page bodies, never pass raw markdown as `storage` format — Confluence renders
`##` and `|---|` as literal characters.

**Preferred path — use `publish.mjs`:** converts markdown → storage automatically (see Publishing section above).

**Low-level fallback — hand-build XHTML** (use only when `publish.mjs` does not fit the use-case):
- Write XHTML storage format (`<h1>`, `<h2>`, `<p>`, `<table>`, `<ul>`, `<strong>`, `<code>`, etc.)
- Write to `/tmp/*.html`, then pass that file to `update-page` or `create-page`

### Page structure for multi-section guides

Any page with 3+ sections (headings) **must include a Table of Contents macro** as the first element in the body, so readers can jump to sections directly.

**ToC in XHTML storage format:**
```html
<ac:structured-macro ac:name="toc">
  <ac:parameter ac:name="minLevel">1</ac:parameter>
  <ac:parameter ac:name="maxLevel">3</ac:parameter>
  <ac:parameter ac:name="type">list</ac:parameter>
  <ac:parameter ac:name="printable">true</ac:parameter>
</ac:structured-macro>
```

**Full page template (storage format):**
```html
<ac:structured-macro ac:name="toc">
  <ac:parameter ac:name="minLevel">1</ac:parameter>
  <ac:parameter ac:name="maxLevel">3</ac:parameter>
</ac:structured-macro>

<h2>Introduction</h2>
<p>...</p>

<h1>Section 1</h1>
<h2>Subsection</h2>
<p>...</p>
```

### Confluence macros in storage format

| Macro | Storage format |
|-------|---------------|
| Table of Contents | `<ac:structured-macro ac:name="toc">...</ac:structured-macro>` |
| Info panel | `<ac:structured-macro ac:name="info"><ac:rich-text-body><p>text</p></ac:rich-text-body></ac:structured-macro>` |
| Note panel | `<ac:structured-macro ac:name="note"><ac:rich-text-body><p>text</p></ac:rich-text-body></ac:structured-macro>` |
| Tip panel | `<ac:structured-macro ac:name="tip"><ac:rich-text-body><p>text</p></ac:rich-text-body></ac:structured-macro>` |
| Warning panel | `<ac:structured-macro ac:name="warning"><ac:rich-text-body><p>text</p></ac:rich-text-body></ac:structured-macro>` |
| Expand (collapsible) | `<ac:structured-macro ac:name="expand"><ac:parameter ac:name="title">Title</ac:parameter><ac:rich-text-body><p>text</p></ac:rich-text-body></ac:structured-macro>` |
| Code block | `<ac:structured-macro ac:name="code"><ac:parameter ac:name="language">json</ac:parameter><ac:plain-text-body><![CDATA[...]]></ac:plain-text-body></ac:structured-macro>` |
