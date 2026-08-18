/**
 * Shared JSDoc type definitions — the cross-boundary shapes for the whole generator.
 *
 * Responsibility: one place that names every shape passed between the pipeline stages
 *   (spec -> validate -> layout -> emit -> render). Editors and LLMs get real hints via
 *   `@typedef {import('../lib/types.mjs').Foo}` without any build step.
 * Inputs/Outputs: none — this module is types only (no runtime values).
 * Edit here when: you add a field to a spec/model shape, or add a new diagram type whose
 *   spec/model shapes differ. Keep names stable; they are referenced across modules.
 * Do NOT: put runtime logic here. It must stay a pure type surface.
 */

// ----------------------------- Spec (authoring) ------------------------------

/**
 * @typedef {Object} Participant
 * @property {string} id            Stable identifier referenced by messages/notes.
 * @property {string} [title]       Primary label (defaults to `id`); char-budgeted (R3).
 * @property {string} [subtitle]    Optional secondary label; char-budgeted (R3).
 * @property {'actor'|'box'} [kind] Visual kind (defaults to `box`).
 */

/**
 * @typedef {Object} Message
 * @property {'call'|'return'|'self'} kind
 * @property {string} from           Source participant id.
 * @property {string} [to]           Target id (required for call/return; omitted/self==from).
 * @property {string} [text]         Label; char-budgeted per kind (R4).
 */

/**
 * @typedef {Object} Note
 * @property {string} under          Id of the single participant this note sits under (R5).
 * @property {string} text           Body; char-budgeted (R5), wrapped by the layout.
 */

/**
 * A parsed diagram spec: a common envelope (`type`, `id`, `title?`) plus a per-type body.
 * The `sequence` body is `participants` + `messages` + optional `notes`.
 * @typedef {Object} Spec
 * @property {string} type
 * @property {string} id
 * @property {string} [title]
 * @property {Participant[]} [participants]
 * @property {Message[]} [messages]
 * @property {Note[]} [notes]
 */

// ----------------------------- Layout model ----------------------------------

/** A rectangle in diagram coordinates. @typedef {{x:number,y:number,w:number,h:number}} Rect */

/**
 * A participant after layout: geometry + the pre-positioned header text cells.
 * @typedef {Object} PLayout
 * @property {string} id
 * @property {string} title
 * @property {string|null} subtitle
 * @property {Rect} titleCell
 * @property {Rect|null} subtitleCell
 * @property {'actor'|'box'} kind
 * @property {number} index
 * @property {boolean} isActor
 * @property {boolean} hasBar
 * @property {number} xCenter
 * @property {number} width
 * @property {number} left
 * @property {number} right
 * @property {number} headerTop
 * @property {number} headerH
 * @property {number} [lifelineBottom]
 */

/**
 * A message after layout. Call/return carry endpoint + relative label position; self carries
 * the hook geometry and a pre-wrapped, pre-positioned label box.
 * @typedef {Object} MsgLayout
 * @property {'call'|'return'|'self'} kind
 * @property {number} y
 * @property {string} [text]              call/return label
 * @property {number} [x1]                call/return source x
 * @property {number} [x2]                call/return target x
 * @property {number} [labelX]            relative label position in [-1,1]
 * @property {string[]} [lines]           self label, pre-wrapped
 * @property {number} [startX]            self hook start x (main bar edge)
 * @property {number} [outX]              self hook outward x
 * @property {number} [endX]              self hook arrowhead x
 * @property {number} [hookY]             self hook top y
 * @property {number} [drop]              self hook vertical drop
 * @property {number} [labelW]            self label box width
 * @property {number} [labelY]            self label box y
 * @property {number} [labelH]            self label box height
 * @property {number} [nestH]             self nested-activation height
 */

/**
 * The full deterministic layout consumed by `emit`.
 * @typedef {Object} LayoutModel
 * @property {PLayout[]} participants
 * @property {MsgLayout[]} messages
 * @property {Rect[]} activations
 * @property {(Rect & {lines:string[]})[]} notes
 * @property {number} width
 * @property {number} height
 */

// ----------------------------- Flow spec (authoring) -------------------------

/**
 * A node in a `type: flow` diagram: a box (default) or a decision rhombus, placed on a manual
 * grid. `row` is the band (top->bottom); the column is the node's index within its row unless
 * an explicit `col` overrides it (so a lone node can sit under a specific node above).
 * @typedef {Object} FlowNode
 * @property {string} id             Stable identifier referenced by edges.
 * @property {number} row            Band index (integer >= 0), top to bottom.
 * @property {number} [col]          Optional explicit column override (integer >= 0).
 * @property {'box'|'decision'} [kind] Visual kind (defaults to `box`). A `decision` is a bare
 *                                   rhombus: `title`/`subtitle` are ignored (its meaning is on
 *                                   the branch arrows).
 * @property {string} [title]        Primary label (defaults to `id`); fixed box, so char-budgeted (F3). Boxes only.
 * @property {string} [subtitle]     Optional secondary label; char-budgeted (F3). Boxes only.
 */

/**
 * A directed edge between two flow nodes. `text` is an optional label (required as a guard on a
 * `decision`'s outgoing edges, F4). `type` selects the arrow style: `sync` (default, solid) or
 * `async` (dashed — an out-of-band / background relationship, e.g. a scheduled feed; F6).
 * @typedef {Object} FlowEdge
 * @property {string} from           Source node id.
 * @property {string} to             Target node id.
 * @property {string} [text]         Optional label; char-budgeted (F3), wraps.
 * @property {'sync'|'async'} [type] Arrow style (defaults to `sync`); `async` renders dashed (F6).
 */

/**
 * A parsed flow spec: the common envelope plus a `nodes` grid and `edges`.
 * @typedef {Object} FlowSpec
 * @property {'flow'} type
 * @property {string} id
 * @property {string} [title]
 * @property {FlowNode[]} nodes
 * @property {FlowEdge[]} [edges]
 */

// ----------------------------- Flow layout model -----------------------------

/**
 * A flow node after layout: its cell rectangle plus pre-positioned header text cells.
 * @typedef {Object} FNodeLayout
 * @property {string} id
 * @property {'box'|'decision'} kind
 * @property {number} row
 * @property {number} col
 * @property {Rect} rect              The node box/rhombus rectangle (diagram coords).
 * @property {Rect|null} titleCell    Null for a decision (a bare diamond has no text cell).
 * @property {Rect|null} subtitleCell
 * @property {string} title
 * @property {string|null} subtitle
 * @property {string[]} titleLines
 * @property {string[]} subtitleLines
 */

/**
 * A flow edge after layout: absolute endpoints, optional explicit waypoints, and a
 * pre-positioned single-line-or-wrapped label with its side/anchor resolved.
 * @typedef {Object} FEdgeLayout
 * @property {string} from           Source node id (endpoint identity for the F8 geometry check).
 * @property {string} to             Target node id (endpoint identity for the F8 geometry check).
 * @property {number} x1
 * @property {number} y1
 * @property {number} x2
 * @property {number} y2
 * @property {{x:number,y:number}[]} [waypoints]
 * @property {string[]} labelLines
 * @property {Rect|null} labelCell
 * @property {'left'|'right'|'center'} labelAlign
 * @property {boolean} [async] True when the edge is `type: async` (emitted dashed).
 */

/**
 * The full deterministic flow layout consumed by `emit`.
 * @typedef {Object} FlowLayoutModel
 * @property {FNodeLayout[]} nodes
 * @property {FEdgeLayout[]} edges
 * @property {number} width
 * @property {number} height
 */

// ----------------------------- Type contract ---------------------------------

/**
 * The uniform interface every diagram type module must expose (the extensibility seam).
 * @typedef {Object} DiagramType
 * @property {string} type                        Discriminator used in `spec.type`.
 * @property {string} title                       Human description for listings.
 * @property {(spec: Spec) => string[]} validate  Returns error strings ([] = valid).
 * @property {(spec: Spec) => LayoutModel} layout  Derives all geometry from the spec.
 * @property {(spec: Spec, opts?: {specYaml?: string}) => string} emit  Render-safe mxGraph XML.
 */

export {}
