# Atlassian scripts — architecture

This is the map for the Confluence tooling under `scripts/`. Read this before
editing: it says where each concern lives, where the boundaries are, and — most
importantly — **where to make a given change** so you extend the system instead
of bolting on a workaround.

The Jira side (`jira.mjs`, `auth.jiraApi`/`agileApi`) is intentionally out of
scope here; see *Out of scope / known debt* at the end.

## Mental model: one agnostic core, a thin typed layer on top

The system publishes **any** markdown file to Confluence storage XHTML. On top
of that generic pipeline sits an optional **typed** layer (FSD / ISD) that adds
chrome: a header card, approval rosters, a Reference Materials table, a TOC,
status lozenges, and resolved @mentions.

The code lives under `lib/` in five domain folders (`util/`, `atlassian/`,
`doc/`, `diagrams/`, `jira/`); paths below are relative to `lib/`.

```
                 ┌─────────────────────────── TYPED LAYER (opt-in via --type) ────────────────────────────┐
                 │  doc/types/index.mjs  registry: getDocType(type) → DocType facade                       │
                 │  doc/types/base.mjs   shared behaviour (deriveCard/Approvals, validate,                 │
                 │                        validateFormat, collectMentionNames, parseRequirements)          │
                 │  doc/types/fsd.mjs     FSD vocabulary (headings, requiredH2, requirement gate)           │
                 │  doc/types/isd.mjs     ISD vocabulary                                                    │
                 │  doc/render.mjs       model → chrome via Nunjucks templates/confluence/*.njk             │
                 │  doc/profiles.mjs     per-type profile (parent env, toc, badgeMap, page width)           │
                 └───────────────────────────────────────────────┬───────────────────────────────────────┘
                                                                  │ builds on
┌─────────────────────────────────────── AGNOSTIC CORE (no FSD/ISD knowledge) ─────────────────────────────┐
│  doc/model.mjs           parseDoc / serializeDoc / parseBody / collectHeadings (markdown ⇄ generic model) │
│  doc/md-to-storage.mjs   body markdown → storage XHTML (lists, code, tables, links, badges, mentions)     │
│  doc/storage-to-doc.mjs  reverse: page STORAGE → generic model → canonical markdown (strips wiki chrome)  │
│  diagrams/attachment-sync.mjs  hash-gated file → attachment push (content-type agnostic)                  │
│  doc/publish.mjs         orchestration: parse → (validate) → render/convert → create/update → attachments │
│  atlassian/attachments.mjs / atlassian/url.mjs / atlassian/users.mjs   API helpers                        │
│  atlassian/auth.mjs      env + fetchJson/api client                                                       │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
      shared primitives (pure): util/{cli-args,xhtml,node-io} · doc/status-vocab · atlassian/html-to-markdown
```

**The one rule that keeps this clean:** the agnostic core NEVER imports a concrete
doc type. It only ever touches `doc/types/index.mjs` (the registry) and the
uniform `DocType` interface. FSD/ISD strings live in exactly one place —
`doc/types/fsd.mjs` / `doc/types/isd.mjs` (vocabulary) and the `*.njk` templates
(chrome markup). Everything else is generic.

## Module map

### Entry points (CLI, at `scripts/`)
- **`publish.mjs`** — publish/update one markdown file. Parses flags, builds a
  `mentionMap`, delegates to `doc/publish.publishDoc`.
- **`confluence.mjs`** — the low-level Confluence CLI (get/create/update/copy/
  delete pages, attachments, comments, search, `pull-diagrams`, `modify-table`).
- **`merge-baseline.mjs`** — the safe-re-publish gate: a git-native, body-scoped
  3-way merge of the working copy against a freshly-pulled (gitignored) baseline,
  with the ancestor read from git history. Delegates to `doc/merge.mergeDocBody`.
- **`export-confluence-to-markdown.mjs`** — bulk export (storage/export_view →
  markdown). **`validate-confluence-export.mjs`** — QA-only validator for that
  bulk export (compares exported `.md` against live pages); not part of the
  FSD/ISD publish path.

### `doc/` — the templated-document system (agnostic core + typed layer)
- **`doc/model.mjs`** — generic markdown ⇄ model. Knows the *shape* of a
  templated page (h1 title, a header card H2, H3 sections, a Reference Materials
  table, then a body) but not which strings are FSD vs ISD — the caller passes
  `bodySections`. Round-trip stable: `serializeDoc(parseDoc(md)) ≈ md`.
- **`doc/md-to-storage.mjs`** — body markdown → storage XHTML. `thematicBreak` and
  `blankParagraphs` are opt-in; `badgeMap` turns backtick status words into
  lozenges; `mentionMap` turns `@Name` into user links.
- **`doc/storage-to-doc.mjs`** — the reverse of publish: `parseStorageToModel`
  (agnostic engine) turns a page's STORAGE into the generic model, and
  `storageToDoc({ type, … })` (async driver) resolves mentions + serializes. Powers
  the typed reverse pull (see *The type-aware reverse pull*).
- **`doc/pull.mjs`** — the shared typed reverse-pull core over `storageToDoc`:
  STORAGE → canonical markdown + download the body's non-diagram images (skipping
  the generated-diagram PNGs). Used by BOTH `confluence.mjs get-page --type` and
  `export-confluence-to-markdown --type`; diagram reconstruction stays one call up
  (both callers run `diagrams/reconstruct.mjs` over the final markdown).
- **`doc/publish.mjs`** — the orchestrator (see *Publish flow*).
- **`doc/merge.mjs`** — body-scoped git-native 3-way merge (`mergeDocBody`,
  `splitHeadBody`, `chromeDrift`) over `git merge-file`. Merges only the document
  body; chrome is kept from the working copy and wiki drift is surfaced, not
  merged. Pure (no repo needed) — the `merge-baseline.mjs` CLI supplies the
  git-history ancestor.
- **`doc/render.mjs`** — `renderDoc(model, opts)` → chrome storage via Nunjucks.
- **`doc/profiles.mjs`** — per-type publish profile (parent env, toc, badgeMap,
  content appearance / page width).
- **`doc/status-vocab.mjs`** — `STATUS_VALUES` + `STATUS_COLOURS` (one source of
  truth for the status words and their lozenge colours), plus the `<client> review`
  placeholder and `deriveClientName` that resolves it from a document's approval
  group label — no client name is ever written into the library.
- **`doc/meta.mjs`** — read/stamp the Confluence URL + Page ID header in a
  working-copy markdown file.
- **`doc/types/index.mjs`** — the registry (`getDocType`, `listDocTypes`). The
  ONLY door from the core into a concrete type.
- **`doc/types/base.mjs`** — `makeDocType(config)`: all shared typed behaviour.
- **`doc/types/fsd.mjs`, `doc/types/isd.mjs`** — vocabulary only.
- **`templates/confluence/*.njk`** — `base.njk` (shared chrome), `fsd.njk`/
  `isd.njk` (per-type extension points), `components/atlas.njk` (macros).

### `atlassian/` — REST transport + Atlassian formats/helpers
- **`atlassian/auth.mjs`** — `.env` load + `fetchJson` and the
  `api`/`jiraApi`/`agileApi` clients.
- **`atlassian/url.mjs`** — `${baseUrl}/wiki/...` URL shapes + space-key lookup.
  Depends on `auth.mjs`, so it is lazy-imported after the `--dry` early return.
- **`atlassian/attachments.mjs`** — upload/update attachment + `mimeForFile`.
- **`atlassian/users.mjs`** — user resolution both ways: `resolveMentions`
  (display name → account id, CQL, for publish) and `makeAccountNameResolver`
  (account id → display name, for the reverse pull).
- **`atlassian/adf.mjs`** — markdown → ADF JSON (Confluence comments + Jira
  descriptions).
- **`atlassian/html-to-markdown.mjs`** — `makeTurndown` (export_view → markdown,
  shared by export/validate) + `makeStorageTurndown` (STORAGE-body converter used
  by the typed reverse pull) + export-header parsing.

### `util/` — pure primitives (no domain knowledge)
- **`util/cli-args.mjs`** — one argv parser (`--k v`, `--k=v`, booleans, repeatable).
- **`util/xhtml.mjs`** — `escHtml`/`escAttr`/`escapeRegExp`: the ONE place raw
  strings become safe storage text/attributes.
- **`util/node-io.mjs`** — `tryReadFile`, `withConcurrency`, `fileExists`, and the
  shared filename helpers `sanitizeFileName` / `safeDecodeURIComponent`.

### `diagrams/` — image + source, never editable macros
- **`diagrams/attachment-sync.mjs`** — batch, hash-gated attachment push. Stores
  `sha256:<hex>` in the attachment comment; unchanged bytes are skipped so links
  stay stable. Fully content-type agnostic (the doc core's attachment layer). With
  `prune`, it also deletes **managed** orphans — attachments carrying our `sha256:`
  gate whose file is no longer referenced (`decidePrune`, unit-tested; a rename is
  remove+add) — while unmanaged orphans are only warned, never deleted. A referenced
  file missing on disk is skipped (`action:'missing'`) so a dangling link never
  crashes a publish. Delete goes through `attachments.deleteAttachment` (v1 REST).
- **`diagrams/publish.mjs`** — decides which referenced images are generated
  diagrams (an image with a sibling `<slug>.drawio`) so the `.drawio` source is
  uploaded alongside the `.png`.
- **`diagrams/pull.mjs`** — pure reverse primitives: pick the `.drawio`/`.png`
  pairs from a page's attachments (`collectDrawioAttachments`) and splice a
  reconstructed ` ```drawio ` block into the doc (`upsertDiagramBlock`, replacing
  an existing block or a bare exported image in place).
- **`diagrams/reconstruct.mjs`** — the shared reverse orchestrator over
  `pull.mjs`: download the `.drawio` (+ `.png`) under clean slug names, run the
  diagrams `reverse.mjs` to decode the embedded spec, and splice block + image.
  Used by BOTH `confluence.mjs pull-diagrams` and `export-confluence-to-markdown`
  so a downloaded page comes back fully re-editable (text, images, diagrams).
- **`diagrams/reverse-cli.mjs`** — the ONE resolved path to the diagrams skill's
  `reverse.mjs`, imported by the confluence CLI, the exporter, and the tests
  (previously hand-resolved three times).

### `jira/` — Jira-only helpers
- **`jira/stories-md.mjs`** — pure parsing/backfilling of the deterministic
  `stories.md` (no I/O), used by `jira.mjs`.

## The agnostic-core ↔ typed boundary

`DocType` (returned by `getDocType(type)`) is the entire contract between the
core and a type:

```
{ type, cardHeading, bodySections, requiredH2, requirementSectionTitle,
  parse(md)                 → model (+ documentCard, approvals)
  validate(model)           → { ok, errors, warnings }         (content rules)
  validateFormat(md)        → { ok, errors, warnings }          (required H2s)
  collectMentionNames(model)→ string[]                          (author + approvers)
  deriveCard(sections) / deriveApprovals(sections) / parseRequirements(body) }
```

`doc/publish.mjs` and `doc/render.mjs` only ever call these methods; they
never branch on `type` or spell an FSD/ISD string. Untyped publishes skip the
typed layer entirely (plain `doc/md-to-storage.mjs`).

## Publish flow (PUSH: markdown → Confluence)

`publish.mjs` → `doc/publish.publishDoc`:

1. Resolve the profile (`doc/profiles.mjs`) and the `DocType` (`getDocType`).
2. `useTemplate` = `--render=template` OR (a profile exists AND not
   `--render=markdown`).
3. **Validate format** (template mode, unless `--skip-validation`):
   `docType.validateFormat(md)` — required H2s present.
4. **Parse** (template mode): `docType.parse(md)` → model with `documentCard` +
   `approvals`. Title defaults from the h1.
5. **`--dry`** returns here after writing a preview HTML (no credentials loaded).
6. Load `atlassian/auth.mjs` (credentials) + `atlassian/url.mjs` lazily.
7. **Resolve mentions** (template mode): `docType.collectMentionNames(model)` →
   `resolveMentions` → account ids merged with explicit `--mention`.
8. **Build body**: template mode → `renderDoc(model, { type, mentionMap, badgeMap,
   includeToc })`; markdown mode → `mdToStorage` (+ TOC macro when a profile is set).
9. **Create or update** the page (v2 API). On update, open inline comments are
   re-anchored first.
10. Set content appearance (page width) when the profile defines it.
11. **Attachments** (unless `--skip-attachments`): `collectDiagrams` finds images
    + their `.drawio` companions; `syncAttachmentsToPage` pushes them hash-gated.

## Diagrams: images + `.drawio` source, no macros

There is **no draw.io macro** anywhere. Diagrams are generated as a `.png`
(rendered image) plus a `.drawio` source; publish uploads BOTH as hash-gated
attachments and the page body embeds the `.png` as a normal `<ac:image>`. This
protects diagrams from in-Confluence editing and makes the round-trip
authoritative.

**Durable signal:** the marker that an image is a generated diagram is the
sibling attachment — a `<slug>.png` whose page also carries `<slug>.drawio` of
the same basename. (An earlier HTML-comment anchor was dropped because
Confluence's sanitizer strips comments on save.) `diagrams/pull.mjs` keys off this
sibling to rebuild the authored diagram blocks.

## Pull flow (Confluence → markdown), today

- **`confluence.mjs get-page <id> [storage|adf] [--into <path>]`** — read a page
  body (raw storage XHTML or ADF). `--into` writes a working copy and stamps the
  Confluence URL + Page ID header (`doc/meta.mjs`).
- **`confluence.mjs get-page <id> --type=<fsd|isd> [--into <path>]`** — the
  single-page **typed reverse pull**: reverse the page's STORAGE into canonical
  authored markdown via the shared `doc/pull.mjs` core, reconstruct diagram blocks,
  and (with `--into`) stamp the meta header. Body images / diagram sources download
  to a throwaway temp dir so a baseline pull never clobbers the working copy's
  `./assets`, while the markdown still links `./assets/...` for a clean diff. This
  is what a re-publish workflow uses to fetch the `<feature>.baseline.md`.
- **`confluence.mjs pull-diagrams <id> --into <doc.md>`** — reverse-pull generated
  diagrams from the page's `.drawio` attachments into the doc (idempotent). Thin
  wrapper over `diagrams/reconstruct.mjs`.
- **`export-confluence-to-markdown.mjs`** — bulk export to markdown. Two modes,
  one command:
  - **generic (default)** — the `export_view` → Turndown dump
    (`atlassian/html-to-markdown.mjs`), keeping the generic markdown-mirror use
    case intact.
  - **typed (`--type=fsd|isd`)** — the **type-aware reverse pull** (below): fetch
    STORAGE, invert the publish chrome/transforms into the canonical model, and
    serialize back to authored markdown. A page that doesn't parse as the given
    type falls back to the generic dump for that page (with a `WARN`).

  In both modes diagrams round-trip automatically: any exported image backed by a
  `.drawio` attachment is pulled as an editable ` ```drawio ` block (via
  `diagrams/reconstruct.mjs`), and its `.png`/`.drawio` are saved under clean slug
  names (no URL-hash suffix) so the pair re-publishes idempotently through the
  hash-gated attachment sync.

## Invariants (do not break these)

1. **Escaping choke point.** Every dynamic value entering storage XHTML goes
   through `util/xhtml.mjs` (`escHtml`/`escAttr`). Never hand-roll escaping.
2. **One status vocabulary.** Status words + colours come from
   `doc/status-vocab.mjs`. Never redefine a colour map elsewhere.
3. **Core stays type-free.** No file outside `doc/types/` and `templates/` may
   contain an FSD/ISD-specific string. Reach types only via `getDocType`.
4. **Renderer owns body layout.** `doc/render.mjs` strips body blank lines + `---`
   and re-applies layout from fixed template rules; authored dividers are
   cosmetic. Body-part conversion runs with `thematicBreak`/`blankParagraphs` OFF.
5. **Round-trip stability.** `serializeDoc(parseDoc(md))` is idempotent
   (`serialize → parse → serialize` is byte-identical). Guarded by tests.
6. **Hash-gated attachments.** Unchanged bytes must skip re-upload (stable links).
7. **Diagrams are images.** No draw.io macros; upload `.png` + `.drawio` source.

## Edit X → change it here

| You want to… | Edit |
| --- | --- |
| Add a new doc type | `doc/types/<type>.mjs` (vocabulary) + register in `doc/types/index.mjs` + add `templates/confluence/<type>.njk` + a `doc/profiles.mjs` entry |
| Change FSD/ISD headings or required sections | that type's `doc/types/<type>.mjs` |
| Change shared typed behaviour (card/approvals/validation) | `doc/types/base.mjs` — add a config knob, don't branch on `type` |
| Add/rename a status or change a lozenge colour | `doc/status-vocab.mjs` (one place) |
| Change how markdown body becomes storage | `doc/md-to-storage.mjs` |
| Change chrome markup (card/approvals/legend/TOC) | `templates/confluence/base.njk` (+ `components/atlas.njk`) |
| Change escaping rules | `util/xhtml.mjs` |
| Change how flags parse | `util/cli-args.mjs` |
| Change the publish sequence | `doc/publish.mjs` |
| Change URL shapes / space lookup | `atlassian/url.mjs` |
| Change the export→markdown conventions | `atlassian/html-to-markdown.mjs` (`makeTurndown`) |
| Change the typed reverse pull (storage→canonical md) | `doc/storage-to-doc.mjs` (engine + driver) + `makeStorageTurndown` in `atlassian/html-to-markdown.mjs`; the shared entry-point core is `doc/pull.mjs` |

## The type-aware reverse pull — BUILT (`get-page --type` / `export --type`)

Goal: pull a published FSD/ISD page back into its canonical authored markdown so a
human edit on the wiki can be re-synced. This is the inverse of the publish flow
and reuses the same registry + serializer, so it stays type-free in the core.

Two entry points share the `doc/pull.mjs` core (`pullTypedMarkdown`, which wraps
`storageToDoc` + body-image download):

```
node confluence.mjs get-page <id> --type=isd --into <feature>.baseline.md   # one page
node export-confluence-to-markdown.mjs <root_id> --type=isd                 # a subtree
```

The single-page pull is storage-only (you name the type). The bulk export also
fetches `body.export_view` so a page that isn't the given type can fall back to
the generic dump per page (`NotDocTypeError`).

Flow (per page, inverse of the forward path):

1. **Fetch** `body.storage` (alongside `body.export_view` for fallback).
2. **Explicit type only** — no label/title auto-detect. `--type=isd` reconstructs
   every exported page under the ISD profile; `getDocType(type)` resolves it.
3. **`parseStorage(storageXhtml) → model`** — a `DocType` method
   (`doc/types/base.mjs`) that delegates to the doc-type-agnostic engine
   `parseStorageToModel` in **`lib/doc/storage-to-doc.mjs`**. Driven by this type's
   `cardHeading`, it reconstructs the generic model `serializeDoc` consumes:
   - the combined chrome table → `header.sections` (card + approval groups),
   - the `Reference Materials` table → `references` (inverting `parseLinkCell`),
   - the body (from the chrome→body `<hr/>` to the Document Change Log footer,
     with the template's structural `<hr/>`/`<p/>` dividers stripped) → markdown
     via `makeStorageTurndown` in `atlassian/html-to-markdown.mjs`.
   - **Detection gate:** absence of the `cardHeading` table throws
     `NotDocTypeError`, which the export catches to fall back per page.
4. **Rehydrate typed values**: status lozenges → status words (via the
   `<ac:parameter ac:name="title">`), `<ri:user account-id>` → display names
   (async, resolved through the v1 user API + cache in the export — the reverse of
   the publish `mentionMap`), `<ac:image><ri:attachment>` → `![](<assets>/<file>)`
   (paired with the `.drawio` companion `diagrams/reconstruct.mjs` restores).
5. **`serializeDoc(model)`** → canonical markdown; the export prepends the same
   `# Title` + Confluence/Page ID/Version metadata header the generic dump uses.

Boundary preserved: the driver calls `getDocType(type).parseStorage(...)` and
`serializeDoc(...)`; the core never parses FSD/ISD chrome directly. `makeStorageTurndown`
is a sibling of `makeTurndown` that (unlike it) converts tables to GFM pipes and
adds the macro-inversion rules.

Fidelity notes (call-outs, not blockers): the reverse target is
**canonical-equivalence**, i.e. `serializeDoc(reverse(storage))` equals
`serializeDoc(parse(authored))` (layout/spacing is renderer-owned, per invariant
5) — not byte-equality with a hand-authored file. Code macros round-trip as fenced
blocks; diagrams come back as `drawio:*` blocks (the current canonical), not a
`mermaid` source an author may have first written.

## Out of scope / known debt

- **Jira (`jira.mjs`, `lib/jira/*`, `auth.jiraApi`/`agileApi`).** The Jira CLI is
  intentionally not mapped above. It carries its own concerns (issue CRUD, story
  creation/backfill, and the Story↔doc remote-link surface: `create-stories`
  linking by default, `link-config`, `add-remote-link`/`delete-remote-link`, and
  the doc footer's "Linked Jira Tickets" macro). See `references/jira.md`.
- **`export_view` fetch in typed bulk export.** In `--type` mode the exporter
  fetches both `body.storage` (typed reverse) and `body.export_view` (generic
  per-page fallback). When the typed parse succeeds the `export_view` body is
  fetched-but-unused; it is kept because the per-page fallback needs it and
  fetching lazily would cost a second request on every fallback page. The
  single-page `get-page --type` pull has no fallback and is storage-only.
- **Asset round-trip.** `get-page --type` writes body images / diagram sources to
  `--assets-dir` when given (KEPT — discovery localizes them into the working copy's
  `./assets`) or a throwaway temp dir otherwise (a baseline pull stays a pure text
  artifact and never clobbers `./assets`). At submit the baseline pull stages wiki
  binaries in a gitignored dir; after the body merge, `merge-baseline --assets-from`
  copies in only the newly-referenced (wiki-added) images (`doc/assets.mjs`, pure/
  unit-tested) and warns on dangling links. Publish then hash-syncs referenced
  assets and prunes managed orphans (above). Binaries are never text-merged; the
  markdown decides what is referenced and these steps keep the bytes in sync.
