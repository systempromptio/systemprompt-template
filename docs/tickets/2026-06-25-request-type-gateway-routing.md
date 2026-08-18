# Ticket: Request-type-aware gateway routing in core

**Component:** `systemprompt-core` — gateway / inference routing (consumed by astound via `[patch.crates-io]`)
**Type:** Feature / RFC (planning)
**Priority:** Medium — forward-compatibility seam; no user-visible behavior change on landing
**Status:** Proposed
**Author:** Ed (ed@systemprompt.io), 2026-06-25
**Related:** gateway redirect (Cowork `claude-*` → Cerebras `zai-glm-4.7`); GLM cost/speed/sovereignty analysis

---

## Problem

The gateway routes inference requests to a backend **purely on the model name** (a glob
`model_pattern`, first-match-wins). We want to send different *types* of request to different
backends — e.g. **reasoning-heavy** traffic to Cerebras (fast GLM decode, ~914 tok/s) and
**tool-heavy / bulk / background** traffic to a cheaper EU self-host backend (~130–300 tok/s,
data-sovereign, cheap at scale) — without forking behavior per client.

Today this is impossible: `GatewayRoute` has no way to express "match when the request looks like
X", so all `claude-*` traffic collapses to a single backend.

## Key finding (de-risks the work)

The data needed to classify is **already parsed and in scope at the routing decision** — we are not
adding new plumbing, just widening one function's input.

At `resolve_upstream()` (`crates/entry/api/src/services/gateway/service/resolve.rs`) the full
`CanonicalRequest` exists, including `tools`, `tool_choice`, `thinking`, `reasoning_effort`,
`stream`, `max_tokens`, `system`, `messages`. But the matcher only receives the model string:

- `GatewayRoute::matches(&self, model: &str)` — `crates/shared/models/src/profile/gateway/route.rs`
- `GatewayConfig::resolve_route(&self, registry, model: &str)` — `.../gateway/config.rs`

There is a **direct precedent** to mirror for idiom: `SystemPromptRule` / `system_prompt_overrides`
(declarative request-attribute rules) + the inventory-extensible `OverrideEngine`
(`.../gateway/override_rule.rs`, `service/finalize.rs`).

## Proposed design

### 1. Declarative conditions (primary) — mirror `SystemPromptRule`

Add an optional `when` block to `GatewayRoute`:

```rust
pub struct GatewayRoute {
    // ...existing: id, model_pattern, provider, upstream_model, extra_headers, pricing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<RouteMatch>,
}

#[serde(deny_unknown_fields)]
pub struct RouteMatch {
    pub requires_tools: Option<bool>,
    pub min_tools: Option<usize>,
    pub thinking: Option<bool>,
    pub min_reasoning_effort: Option<ReasoningEffort>,
    pub stream: Option<bool>,
    pub min_input_tokens: Option<u32>,       // estimated over messages
    pub response_format: Option<ResponseFormatKind>,
}
```

- New `GatewayRoute::matches_request(&self, req: &CanonicalRequest) -> bool` = model glob AND all
  present predicates (absent predicate = wildcard; `when: None` = model-only, unchanged).
- `GatewayConfig::resolve_route` takes `&CanonicalRequest` instead of `&str`.
- One call-site edit in `resolve_upstream` (the `request` is already there).
- First-match-wins already gives priority ordering, so the taxonomy lives entirely in profile YAML.

### 2. Programmatic escape hatch (secondary) — mirror `OverrideEngine`

For classification that declarative predicates can't express (accurate token counting, a small
classifier, heuristic task detection), add an inventory seam:

```rust
#[async_trait]
pub trait RouteSelector: Send + Sync {
    async fn refine<'a>(&self, matched: &'a GatewayRoute, req: &CanonicalRequest)
        -> Option<Cow<'a, GatewayRoute>>;
}
inventory::collect!(RouteSelectorRegistration);
```

Called in `resolve_upstream` immediately after the declarative match. Extensions in `extensions/`
register a selector and own the policy in Rust — no further core change once the seam exists.

## Scope / files

Core fork (`../systemprompt-core`):
- `crates/shared/models/src/profile/gateway/route.rs` — `RouteMatch`, `matches_request`
- `crates/shared/models/src/profile/gateway/config.rs` — widen `resolve_route` signature
- `crates/entry/api/src/services/gateway/service/resolve.rs` — pass `request` (1 line)
- (phase 2) new `RouteSelector` trait + inventory call in `resolve_upstream`

Astound (`extensions/web/admin`):
- `GatewayRouteView` + `yaml_io` round-trip so the admin UI and profile serializer carry `when`

Unchanged: `is_model_exposed` stays model-only (exposure ≠ variant selection); authz hook still
runs after route resolution.

## Out of scope / explicit non-goals

- **Client changes.** Cowork / Codex / OpenCode are not yet emitting reliably distinguishable
  requests. This ticket only builds the server seam so routing is config-only when they mature.
- Changing the inbound parser or adapter-selection model (adapter stays provider-`wire`-bound).

## Important caveat — what actually discriminates

In real agent loops the **full tool catalog is sent on every step**, so `requires_tools: true`
matches ~all requests and does *not* separate a "tool step" from a "reasoning step". The signals
that genuinely vary per call are **`thinking` / `reasoning_effort`**, **`stream`** (background calls),
and **the model name the client chose**. Ship with these as the trustworthy discriminators; treat
tool-presence as a weak signal. Categorization ultimately comes from either (A) the gateway
inferring request shape or (B) the client selecting a per-role model name — both are supported by
this design.

## Rollout

1. Land core change with a single `claude-*` catch-all route → byte-for-byte unchanged behavior.
2. Re-comment the `[patch.crates-io]` before any published build (per existing patched-core process).
3. Add real split routes in profile YAML opportunistically as clients begin to vary requests — no
   code change required at that point.

## Acceptance criteria

- [ ] A route with `when:` omitted behaves exactly as today (model-only match).
- [ ] A route with `when: { thinking: true }` matches only requests with thinking enabled; a later
      catch-all handles the rest, verified by an audit-row backend/provider check.
- [ ] `min_reasoning_effort` and `stream` predicates match as specified.
- [ ] Unknown keys under `when:` error loudly (`deny_unknown_fields`).
- [ ] Admin UI lists and round-trips `when:` without dropping fields.
- [ ] (phase 2) An extension-registered `RouteSelector` can override the matched route.

## Open questions

- Token estimation for `min_input_tokens`: cheap char/word heuristic vs real tokenizer? (lean cheap.)
- Should `RouteSelector` be able to *deny* (vs only re-route)? Authz hook already owns deny.
- Do we want per-route observability (which predicate matched) in the audit spine?
