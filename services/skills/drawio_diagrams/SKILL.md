# Draw.io Diagrams

Turn a compact, human-reviewable **YAML spec** into a `.drawio` source file **and** a rendered
`.png`, deterministically and offline. You author the spec; the generator produces the files and
prints their paths; you insert the image reference. Use it whenever the diagram should be
**re-generatable from source in git** rather than a hand-drawn one-off.

This skill is about **how** to draw. Whether a given document section needs a diagram at all is
decided by the workflow that calls the skill, not here.

## Authoring principles (every type)

This is the single home for the cross-type rules;
Read this first, whatever type you draw.

- **Communicate scope.** A diagram exists to make obvious — at a glance, to business *and*
  engineering readers — *what happens, between whom, and where the boundary of our work sits*. If
  an element does not help a reader understand the scope, it is noise.
- **Write for both audiences.** Labels must read for a business stakeholder *and* an engineer:
  plain, recognizable names (real system/vendor names, plain verb phrases), short enough that the
  picture carries the meaning. Searchable beats clever.
- **Stay high-level; collapse to the logic.** Show systems and meaningful interactions, not
  private method names, header names, or payload fields. If a detail does not change the
  architecture or the scope, collapse it — describe one level higher, compromise toward the reader.
- **Be concise.** The fewest elements that still carry the meaning; a crowded diagram hides the
  scope it should reveal. Keep the canvas that small too — reuse the positions the picture already
  occupies rather than spreading elements out, and never leave an empty row or column behind.
- **Keep the picture connected.** A diagram is one unbroken story: it has a single starting point,
  and every element joins the chain from there — nothing appears detached or starts acting on its
  own out of nowhere. If a node has nowhere to connect, it does not belong on this diagram. (A
  background *source* — e.g. a data feed — is the one exception: it may originate on its own, but it
  must still attach to what it feeds. Each type's reference states how it expresses this.)
- **Keep the layout clean — nothing crosses, nothing overlaps.** Spread a node's targets across
  different axes so an arrow is never drawn across another box, and keep boxes and labels from
  colliding. Placement is the lever; each type's reference states how its layout expresses this.
- **Let layout show scope — what we build vs what we consume.** Third-party systems sit at the
  periphery (side of the main flow, far end of the participant row), never woven into the centre.
  A reader should trace the "we build this" spine with their eye and see the third-party systems
  sitting visibly outside it. Show them; never drop them or imply we own them.
- **Order for reading.** Put the initiator/actor first (left for sequence, top for flow) and keep
  related nodes aligned so the eye follows the story; each type's reference states its exact ordering.

## Workflow

The recommended path is an in-markdown diagram block — the source lives next to where it renders
(see [Block format](#block-format)).

1. **Pick the type and read its reference** ([Diagram types](#diagram-types)) — it defines that
   type's spec fields and rules. Read it before writing.
2. **Author** a ` ```drawio:<type>:<id> ` block in the doc.
3. **Validate**: `node scripts/validate.mjs --md <doc.md>` (or `<spec.yaml>`). An invalid spec
   produces no files; fix the reported rule id.
4. **Generate**: `node scripts/generate.mjs --out-dir <assets-dir> --md <doc.md>`. For a doc
   published to Confluence, point `--out-dir` at an `assets/` folder **next to the doc** so both
   files sit beside it. Output filenames are `<id>.drawio` / `<id>.png`.
5. **Insert the image yourself**, e.g. `![Title](./assets/<id>.png)`, right after the block. The
   generator never edits your markdown.
6. **Tell the user how to preview it.** A markdown image does not render inline in a plain source
   editor (they see only the `![…](…)` text — expected). Prompt them to open their editor's Markdown
   preview, or open the generated `.png` directly.

**On Confluence publish** the ` ```drawio ` block is **stripped** (never reaches the wiki) and the
sibling `<id>.drawio` is uploaded automatically as the downloadable source — you only reference the
`.png`.

## Block format

The source-of-truth in a doc is a fenced block whose info string carries the diagram `type` and a
unique `id`; the body is the **scenario only**. Do not repeat `type:`/`id:` in the body — the
header is authoritative and the generator injects them.

````markdown
```drawio:sequence:integration-flow-overview
participants:
  - { id: shopper, title: Shopper, kind: actor }
  - { id: browser, title: Web Browser }
messages:
  - { kind: call,   from: shopper, to: browser, text: "Open storefront" }
  - { kind: return, from: browser, to: shopper, text: "Rendered page" }
```

![Integration Flow Overview](./assets/integration-flow-overview.png)
````

- `drawio:<type>:<id>` — `type` is a registered type (e.g. `sequence`); `id` is a stable slug that
  drives the output filenames (`<id>.drawio` / `<id>.png`).
- **Always quote free text.** Wrap every human-readable value (`text`, `title`, `subtitle`) in
  double quotes — the compact one-line `{ … }` style is comma-sensitive, so an unquoted comma
  splits the value. (Generated/round-tripped blocks are auto-quoted.)
- `generate.mjs --md <doc.md>` processes **every** ` ```drawio:… ` block in the doc.

Each generated `.drawio` embeds the full spec as base64 `data-spec`, so it is self-describing for
round-tripping.

## CLI

```bash
# Validate specs (exit 1 on errors)
node scripts/validate.mjs <spec.yaml>
node scripts/validate.mjs --md <doc.md>        # validate ```drawio:<type>:<id> blocks in a doc
cat spec.yaml | node scripts/validate.mjs --stdin

# Generate .drawio + .png (prints a JSON manifest to stdout)
node scripts/generate.mjs --out-dir <dir> --spec-file <spec.yaml>
node scripts/generate.mjs --out-dir <dir> --spec-stdin < spec.yaml
node scripts/generate.mjs --out-dir <dir> --md <doc.md>   # every ```drawio:<type>:<id> block
#   optional: --name <slug> (single spec)  --scale 1.5  --background '#ffffff'|transparent  --no-png

# Render an existing .drawio to PNG
node scripts/render.mjs <input.drawio> [output.png] [--scale=1.5] [--background=#ffffff]

# Reverse: reconstruct the authoring block from a generated .drawio (prints JSON)
node scripts/reverse.mjs --drawio <file.drawio>   # { id, type, title, png, block }
```

## Round-trip (reverse)

A generated `.drawio` embeds the full spec (base64 `data-spec`), so `reverse.mjs` reprints the
` ```drawio:<type>:<id> ` block with no geometry parsing. This powers the Confluence -> markdown
pull: the Atlassian skill's `confluence.mjs pull-diagrams <pageId> --into <doc.md>` downloads each
`.drawio` attachment, calls `reverse.mjs`, and upserts the block + image back into the doc. Spec is
source of truth — hand-edits in the draw.io app that don't update `data-spec` are not honored.

## Diagram types

Pick the type you need and **read its reference before authoring** — it defines the spec vocabulary
and rules. Do not invent fields or styles a reference does not define.

| type | use it for | reference |
| --- | --- | --- |
| `sequence` | synchronous calls/returns between systems **over time** — who calls whom, in what order, when control returns | [references/sequence.md](references/sequence.md) |
| `flow` | boxes-and-arrows on a manual grid — **activity/decision logic** (steps with branching) or a **plain "who talks to whom"** picture (no decisions) | [references/flow.md](references/flow.md) |

Choosing between them:

- **`sequence`** when the story is an *ordered exchange over time* (a call stack: call -> return)
  and the return of control is the point.
- **`flow`** when the story is *structure or logic* rather than a timed exchange — components and
  how they connect, or a process with steps and decisions. Within `flow`, an **activity** diagram
  adds decision diamonds and branches; a **plain** flow just shows connections/scope. Same `flow`
  type — pick the shape that fits; don't add decisions the context doesn't have.

**How a reference is structured** (same skeleton for every type):

- **CONCEPT** (read to *confirm the type*): a short overview + mental model, a **pick-me-when**
  discriminator against the other types, and the type's **scope stance** — how *this* type places
  third-party systems at the periphery.
- **CONTRACT** (read to *author the chosen type*): the **vocabulary** (block format + spec fields,
  with text budgets in the field tables), the **layout** section — the mechanics of how that type's
  spec becomes geometry plus the UX practice that follows from it, the **rules & limits** (what the
  validator enforces, plus what the type deliberately *cannot* express), a one-line **self-check**,
  and one worked **example** with its rendered image.
