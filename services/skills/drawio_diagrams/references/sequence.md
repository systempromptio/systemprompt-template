# Sequence diagram — authoring reference

The cross-type authoring rules (audience, high-level/collapse, conciseness, scope,
reading order) live in the skill's **Authoring philosophy** — read that first. This reference
adds only what is specific to `sequence`, in the standard CONCEPT -> CONTRACT shape.

## CONCEPT

**Overview.** A sequence diagram shows **synchronous calls between participants over time**: who
calls whom, in what order, and when control returns. Columns are participants (left->right); rows
are messages (top->bottom).

**Mental model — a call stack.** `call X -> Y` hands control to `Y` and pushes a frame (`Y` is now
executing); `return Y -> X` hands control back and pops it; `self X` is internal work by whoever
is currently executing. Only the participant on top of the stack acts at any moment. Every rule
below exists to keep the picture a clean, strictly nested synchronous sequence.

**The story is more than the calls.** When you select from the notes, the diagram-worthy set is
not only participant-to-participant `call`s: an internal step worth surfacing is a `self`, and a
constraint or outcome the arrows cannot show (a trust boundary, a decline path) is a `note`. Treat
both as first-class parts of the picture — reached for with the same restraint as a `call`, never
defaulted away because they are not an arrow between two columns.

**Pick me when** the story is an *ordered exchange over time* between a few participants and the
**order of calls and the return of control** is the point. If the story branches on a decision,
or you only need "who connects to whom" with no time axis, use `flow` instead.

**Scope stance.** Third-party participants sit at the far (right) end of the row, outside the
spine of participants we build; the call to them stays in the picture. Position carries the
boundary — a reader sees a system we consume, not one we own.

## CONTRACT

### Format

Author inline with a ` ```drawio:sequence:<id> ` block: the header carries the `type`
(`sequence`) and the `id`; the body is the **scenario only** — no `type:`/`id:` keys. Add the
`![](./assets/<id>.png)` line yourself after the block; on publish the block is stripped. Quote
every human-readable string value (`text`, `title`, `subtitle`) — the compact one-line `{ … }`
style is comma-sensitive.

### Vocabulary

`participants` — the columns (ordered left->right):

| field | required | budget | meaning |
| --- | --- | --- | --- |
| `id` | yes | — | stable key referenced by `messages` / `notes` |
| `title` | recommended | <= 15 chars | primary bold label (defaults to `id`) |
| `subtitle` | no | <= 22 chars | second line, smaller grey — role/detail |
| `kind` | no | — | `actor` (stick figure in the accent colour — the human / initiator) or `box` (default) |

`messages` — the rows (ordered top->bottom):

| `kind` | fields | text budget | meaning |
| --- | --- | --- | --- |
| `call` | `from`, `to`, `text` | <= 20, single line | solid arrow `from -> to`; opens an activation on `to`. Always left->right (`to` right of `from`) |
| `return` | `from`, `to`, `text` | <= 20, single line | dashed, open-headed arrow; closes the most recent activation on `from`. Always right->left |
| `self` | `from`, `text` (no `to`) | <= 40, wraps | internal work on `from`; renders a small nested activation |

Use `self` for internal work worth surfacing to the reader — a non-obvious or important step —
not for every routine action; skip it when nothing meaningful would be lost.

`notes` — optional callouts:

| field | required | budget | meaning |
| --- | --- | --- | --- |
| `under` | yes | — | id of the ONE participant the note sits under |
| `text` | yes | <= 70, wraps | callout body — a non-obvious constraint |

### Rules & limits

An invalid spec produces **no files**; the error names the rule id.

- **R1 — one call in flight.** Only the participant on top of the stack may act: after A calls B,
  A is blocked until B returns — no second `call`, `self`, or `return` from A until then.
- **R2 — every call returns.** Each `call` is closed by a matching `return` to its caller once the
  callee has nothing more to call. No fire-and-forget / dangling activations.
- **R3 — participant text fits (soft / warning).** `title` <= 15, `subtitle` <= 22 (the header box
  is fixed-width). Over-budget text overflows the header — the generator **warns** and still
  renders; keep within budget so nothing clips.
- **R4 — message budgets (soft / warning).** `call`/`return` <= 20 single line; `self` <= 40
  (wraps). Same as R3: a warning, not a hard failure.
- **R5 — notes.** `under` one participant (a hard rule); `text` <= 70 is a **soft** budget (warns,
  still renders). Add a note only when the flow doesn't already show it.
- **R6 — calls flow left->right.** A `call`'s `to` sits to the right of its `from`; only a
  `return` travels right->left. Order participants so the initiator (usually the `actor`) is far
  left. A leftward call means the order is wrong or it is really a `return`.

Also enforced: `kind` is exactly `call | return | self`; every `from`/`to` references a declared
participant; a `self` has no distinct `to`; participant ids are unique.

**Cannot express:** branching / decision logic, alternative paths, or async fire-and-forget — the
model is a strictly nested synchronous call stack. If the story branches, use `flow`.

### Self-check

Before emitting: the messages form a strictly nested call stack (R1/R2); every arrow points the
right way (R6); all text is within budget (R3/R4, soft — over-budget warns and overflows); any third-party participant sits at the far
end of the row, outside our spine; and you have captured the diagram-worthy internal steps as
`self` and the non-obvious constraints/outcomes the arrows cannot show as `notes` — not defaulted
to participant-to-participant `call`s alone (holding the sparing bar above).

### Example

````markdown
```drawio:sequence:consent-manager-init
title: Consent Manager initialisation
participants:                                       # columns, left->right
  - { id: shopper, title: Shopper, kind: actor }
  - { id: browser, title: Web Browser }
  - { id: cmp,     title: CMP, subtitle: Consent platform }
messages:                                           # rows, top->bottom
  - { kind: call,   from: shopper, to: browser, text: Open storefront }
  - { kind: call,   from: browser, to: cmp,     text: Load CMP SDK }
  - { kind: self,   from: cmp,                   text: Read stored consent }
  - { kind: return, from: cmp,     to: browser,  text: Show banner }
  - { kind: return, from: browser, to: shopper,  text: Rendered page }
notes:                                              # optional, sparingly
  - under: cmp
    text: CMP SDK is loaded once per session.
```

![Consent Manager initialisation](./assets/consent-manager-init.png)
````
