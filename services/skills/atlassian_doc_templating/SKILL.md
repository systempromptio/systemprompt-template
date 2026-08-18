# Atlassian Doc Templating (FSD / ISD → Confluence)

Renders canonical, human-readable **markdown** specification documents (FSD, ISD)
into **classic Confluence storage XHTML** (`<ac:...>` macros, tables, lozenges), and
is designed to round-trip back (Confluence ADF → canonical markdown). The
markdown file is the single source of truth and visually mirrors the wiki page.

For operational publish/fetch commands see the `atlassian` skill; this skill is
the architecture + format contract for the rendering subsystem.

## Core Principles

- **Markdown-native, system-free source.** No YAML front-matter, no page id /
  space / parent in the document. Structure is encoded as normal markdown
  headings and tables so the file reads like the wiki page. Systemic values live
  in `.env` / CLI flags; the target page id is a `--page-id` flag on update.
- **Plain names, resolved at publish.** People are written as plain display names
  (`Viktor Durnev`). The publisher resolves them to Confluence account ids on the
  fly (`lib/atlassian/users.mjs`) and passes a `mentionMap` to the renderer.
  Without a match (dry render), the name renders as plain text.
- **Plain status words → lozenges.** Statuses are canonical lowercase words
  (`approved`, `in progress`, …); the renderer maps them to Confluence status
  macro colours.
- **Chrome vs body.** "Chrome" is produced by Nunjucks templates, matching the
  wiki. FSD and ISD share the SAME chrome skeleton; only the header card label and
  the body section names differ (each type's vocabulary lives in
  `lib/doc/types/{fsd,isd}.mjs`, built on the shared `lib/doc/types/base.mjs`): an
  FSD opens with `## General FSD Information` and its body starts at `## In Scope
  Functional Requirements`, while an ISD opens with `## General ISD Information`
  and its body starts at `## Requirements`. Both H1 page titles end with the doc-type token
  (`… FSD` / `… ISD`) so the page tree is self-describing; the publisher appends
  the token when the authored title omits it. The header is ONE generic container: the card H2
  opens a combined 3-column table, and every following `### ` H3 (approval groups,
  or anything else) adds another section. Each section's heading becomes a merged (`colspan`) grey
  (`#f4f5f7`) header row and its table rows are the white cells below; a 2-col row
  (card) has its value cell span the remaining columns. `base.njk` walks
  `model.header.sections` generically — no hard-coded field or group names — so
  adding an H3 + table adds a section. A cell whose value is a status word renders
  as a lozenge; any other cell goes through @mention resolution (plain text when
  unresolved). The whole table is wrapped in the native Content Properties
  (`details`) macro keyed by the type's `propertiesId` (`fsd-header` / `isd-header`),
  which is invisible on the page but turns column 1 into a property key and column 2
  into its value — that is what lets a publish refresh the parent page's approval
  matrix (`lib/doc/matrix.mjs`). Approval rows are therefore authored
  `role | status | name`: a key must be a short stable token and must never hold a
  macro (a mention). Then the status legend, then Reference Materials (its own typed
  table with real column headers). No two-column layout, no TOC by default.
  Markdown tables cannot merge cells, so md preview is close but not pixel-identical
  (merges/lozenges/@mentions only exist on the wiki). The requirement "body" is
  authored markdown converted to storage by `lib/doc/md-to-storage.mjs`.
- **Round-trip stability.** `parseDoc → serializeDoc → parseDoc` is stable; the
  reverse extractor (ADF → markdown) must reproduce the canonical markdown.

## Canonical Document Format

Parsed by `lib/doc/model.mjs` (`parseDoc`). Section titles are matched
case-insensitively. Anything the parser does not recognise as chrome and that
precedes the first body section is dropped — keep unrecognised content inside the
body.

Chrome tables use a blank placeholder header row (`|  |  |`): GFM requires a
header row, but the column labels are redundant with the merged section title and
are absent on the wiki. Reference Materials keeps its real headers.

```markdown
# <Page Title> FSD                     ← H1, the page title (ends with FSD/ISD)

## General FSD Information             ← header container; opens the combined table
|  |  |                                  (blank placeholder header — no Field/Value)
| --- | --- |
| WBS-Feature Name | ... |
| Project Name | ... |
| Package/Set/Batch | ... |
| Author/Owner | ... |                 ← plain name(s); resolved on publish
| FSD Status | on review |             ← canonical status word
### Astound Approval                   ← H3 subsection; Astound (delivery sign-off)
|  |  |  |                               is required. First H3: no blank line before
| --- | --- | --- |
| SA | approved | <name> |              ← role FIRST: column 1 is the wiki property key

### <Client> Approval                  ← later H3: one blank line before it
|  |  |  |
| --- | --- | --- |
| PO | in progress | <name> |

## Reference Materials                 ← separate chrome H2; real 3-col headers
| Material | Link / reference | Notes |
| --- | --- | --- |
| SRD | [label](url) | ... |            ← labeled link, bare URL (→ inline card), or text


---
## In Scope Functional Requirements    ← FIRST body section; body = verbatim md
### <WBS code> — <Title>               ← ONE story per requirement (no summary table)
#### User Story
#### Visualization                     ← OPTIONAL; State | Desktop | Mobile table
#### Acceptance Criteria
#### Key behaviours


---
## Deferred Requirements
## Change Requests
## Open Questions                      ← empty on generation; Q# | Question | Owner | Notes | Decision
```

- **Body sections** (`lib/doc/types/<type>.mjs` `bodySections`): everything from the first
  body-section H2 to EOF is the body, stored verbatim as markdown. FSD bodies start
  at *In Scope Functional Requirements* (then *Deferred Requirements*, *Change
  Requests*, *Open Questions*); ISD bodies start at *Requirements* (then
  *Integration Specification*, *Out Of Scope / Limitations*, *Deferred
  Requirements*, *Change Requests*, *Open Questions*). Never author a *Change
  History* section — the Document Change Log footer is appended by the chrome.
- **Open Questions is the review surface.** Both types end with it, carrying only
  its sentence and the bare `Q# | Question | Owner | Notes | Decision` header on a
  generated document; reviewers add rows on the published page.
- **In Scope = one story per requirement.** No summary/requirements table — each
  requirement is its own `### ` story (User Story + Acceptance criteria, plus
  optional Key behaviours). Do not group several requirements under one heading.
- **Optional `#### Visualization`.** After User Story, a requirement may carry a
  `State | Desktop | Mobile` table (states down the rows, screenshots/placeholders
  in the cells). Include it only for visually meaningful requirements; omit it
  where nothing grounds it (per the "optional sub-sections omitted when ungrounded"
  rule). Unlike the chrome tables, this table keeps its real column header.
- **Requirement heading codes** (`REQ_HEADING_RE` in `lib/doc/types/base.mjs`): use the clean **WBS code**
  (`GH.NAV`, `GH.PRO`, `SLS.SRCH`) — one per story, **no** `RQ<n>_` / ID prefix —
  then a **whitespace-surrounded dash** (`-`, `–`, `—`), then the title. The
  surrounding whitespace distinguishes the separator from a hyphen inside a code.
  (The regex still tolerates comma-grouped codes, but the canon is one per story.)
- **Status vocabulary** (`STATUS_VALUES` in `lib/doc/status-vocab.mjs`): `draft`,
  `not started`, `in progress`, `on review`, `in review`, `astound review`,
  `<Client> review`, `pending answers`, `approved`. The client slot resolves per
  document: write the client exactly as its approval group names it
  ("Acme Retail Approval" → `Acme Retail review`), and a drifted spelling fails
  validation.

## Body layout is renderer-owned (do NOT author dividers)

Vertical spacing and dividers in the body are NOT taken from the markdown — the
renderer strips every blank line and `---` from the body (`stripBodyLayout`),
splits it into H2/H3 parts, and re-applies a deterministic layout (a divider
before each body H2, a gap before each H3). So authored `---` and extra blank
lines are cosmetic in the `.md` and ignored on the wiki. Author clean markdown:

- Correct **heading hierarchy**: body H2 sections (In Scope / Requirements,
  Deferred Requirements, Change Requests), `### ` requirement stories under them.
- The **required H2 sections** for the type must be present (see *Validation*).
- Chrome H2s (the General FSD/ISD Information card, Reference Materials) and their
  H3 approval groups live above the first body H2 and render into the header
  layout, not as body sections.

Fenced code blocks are preserved verbatim (their blank lines and `---` survive).

A body table's first row publishes as a bold header (`<th><strong>…</strong></th>`,
the shape Confluence's own editor writes) and comes back plain on pull — author the
header text without `**`.

`<br>` is the one piece of inline HTML the renderer honours, for cells that need
more than one line; it publishes as a real break and comes back as `<br>` on pull.
Any other raw HTML is escaped and shows up as literal text on the page.

## Cross-references inside a document

Link to another section with a GitHub-style slug of its heading —
`[the bag entry](#object-bag-entry)` for `#### Object: \`bag[]\` entry` (lowercase,
backticks dropped, every other run of non-alphanumerics collapsed to `-`). The
renderer turns it into a Confluence anchor link and stamps an anchor macro on the
target heading, so a heading's own generated id (unstable for headings with inline
code or punctuation) never matters. Only referenced headings get a macro, and the
reverse pull restores the markdown link, so the round-trip is lossless.

## Rendering Pipeline

`renderDoc(model, opts)` in `lib/doc/render.mjs`:

1. `stripBodyLayout(model.body)` removes body blank lines + `---` (fence-aware),
   then `splitBodyParts` cuts the body into ordered H2/H3 parts.
2. Each part is converted independently with
   `mdToStorage(part, { thematicBreak: false, blankParagraphs: false })` — the
   converter emits NO `<hr/>` and NO empty paragraphs; the template owns all
   dividers/gaps between parts.
3. Nunjucks renders `<type>.njk` with the model + `bodyParts`. Autoescape is
   **off**; every dynamic value is escaped with the `x` (text) or `xa` (attribute)
   filter. Globals: `statusColor(text)` (status word → colour), `isStatus(text)`
   (is the value a status word → lozenge vs @mention), `mentionId(name)` (name →
   account id from `mentionMap`).

Status colours (`STATUS_COLOURS` in `lib/doc/status-vocab.mjs`): `approved`→Green,
`in/on review`/`astound review`→Blue, `<Client> review`→Purple,
`in progress`/`pending answers`→Yellow, `not started`/`draft`→Grey.

## File Structure

```text
skills/operate/atlassian/scripts/
  publish.mjs                      CLI: markdown → publish (flags below)
  lib/
    doc/                           the templated-document system
      model.mjs                    GENERIC core: parseDoc / serializeDoc /
                                   parseBody / collectHeadings (no FSD/ISD strings)
      types/
        index.mjs                  registry: getDocType(type) / listDocTypes()
        base.mjs                   makeDocType(): shared typed behaviour
                                   (deriveCard/Approvals, validate, validateFormat,
                                   collectMentionNames, parseRequirements)
        fsd.mjs / isd.mjs          per-type VOCABULARY only (headings, requiredH2)
      render.mjs                   renderDoc: model → classic <ac:> storage
      md-to-storage.mjs            body markdown → storage (thematicBreak +
                                   blankParagraphs options)
      publish.mjs                  publish orchestration + content-appearance
      matrix.mjs                   approval matrix: read the children's property
                                   keys, upsert the parent's aggregated report
      profiles.mjs                 per-type profile (parent, toc, badges, width)
      status-vocab.mjs             STATUS_VALUES + STATUS_COLOURS (one source)
      meta.mjs                     stamp Confluence URL + Page ID in working copy
    atlassian/                     REST transport + Atlassian helpers
      auth.mjs                     env + api() client (baseUrl trailing-slash safe)
      users.mjs                    resolveMentions: display name → account id (CQL)
      url.mjs / attachments.mjs / adf.mjs / html-to-markdown.mjs
    util/                          pure primitives
      xhtml.mjs                    escHtml / escAttr / normalizeDashes (escaping)
      cli-args.mjs / node-io.mjs
    diagrams/                      attachment-sync.mjs, publish.mjs, pull.mjs
  templates/confluence/
    base.njk                       shared chrome: one combined table built from
                                   model.header.sections, legend, References, body
    fsd.njk / isd.njk              extend base; per-type divergence point
    components/atlas.njk           components: status, mention, toc, link,
                                   colgroup, propertiesOpen/Close
```

### Approval matrix (`lib/doc/matrix.mjs`)

The header table is wrapped in the Content Properties (`details`) macro, so Confluence
reads it as key-value pairs. At the end of every typed publish the publisher rebuilds
ONE Page Properties Report on the **parent** page: it reads the children's property
keys (so the approver columns are whatever the documents carry — nothing per-project is
configured) and scopes the report by the `fsd`/`isd` label plus `ancestor`. Columns run
document, status, author, approver roles, package/batch. A no-op rebuild writes nothing,
so re-publishing does not churn the parent's version history, and a failure only prints
a NOTE. `--skip-matrix` opts out; `confluence.mjs doc-matrix` is the manual entry point.

### Document profiles (`lib/doc/profiles.mjs`)

Per doc type: `parentEnv` (env var for parent page id), `toc` min/max,
`badgeMap`, `versionMessage`, and `contentAppearance` (`'default'` = Narrow /
fixed width, or `'full-width'`). The publisher always writes `contentAppearance`
so page width can be switched between publishes. Add a new doc type by adding a
profile entry plus a `<type>.njk`.

### Publish flags (`publish.mjs`)

`--type=<profile>` (fsd|isd), `--render=template|markdown`, `--page-id=<id>`
(update; omit to create), `--title=<t>` (create; defaults to the H1 in template
mode), `--parent=<id>`, `--space=<key>`, `--mention="Name=id"` (repeatable),
`--skip-attachments`, `--skip-validation`, `--skip-matrix`, `--dry`.

## Validation

Two complementary checks, both on the typed facade (`getDocType(type)` →
`lib/doc/types/base.mjs`):

- **`validate(model)`** — content: missing title, invalid status word,
  missing approver name, duplicate requirement code (errors); empty card fields,
  unusual approver role (warnings). `renderDoc` throws on errors unless
  `validate: false`.
- **`validateFormat(md)`** — the structural canon the model cannot express, run
  on the raw markdown: every required base H2 present (for FSD: General FSD
  Information, Reference Materials, In Scope Functional Requirements, Deferred
  Requirements, Change Requests; for ISD: General ISD Information, Reference
  Materials, Requirements, Integration Specification — approvals are H3, not
  required H2s). Extra H2s are allowed. Layout (dividers / blank-line spacing) is
  renderer-owned and deliberately NOT checked. Template-mode `publishDoc` runs it
  before doing anything and aborts on errors unless `--skip-validation`.

## Status & Next

- **Done:** forward render (markdown → classic storage), publish + mention
  resolution, Narrow/full-width control, renderer-owned body layout, generic
  header container (card H2 + H3 sections → one merged table), pre-publish format
  validator (`validateFormat`), agnostic core (`doc/model.mjs`) + typed registry
  (`doc/types/`) so FSD and ISD share one chrome via `isd.njk` extending
  `base.njk`, typed publish (`--type`) rendering the chrome by default.
- **Pending:** the type-aware reverse pull (storage → canonical markdown
  round-trip) — see `scripts/ARCHITECTURE.md` for the documented seam.
