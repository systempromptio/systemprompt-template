# Design note — the register rows that were scoped, not built

Companion to [`compliance-register.md`](compliance-register.md), which records what shipped.
This note holds the items whose right next step is a decision or a design, with what each
would take. Estimates are engineering effort once the named decision lands.

## REQ-028 — Observability egress (OTEL/SIEM)

What exists: OTLP **ingest** (core decodes and persists traces/logs/metrics), a live JSON
audit stream over SSE (`/admin/api/sse/audit`, fed by `pg_notify` triggers on
`governance_decisions`, `ai_requests`, `plugin_usage_events`), and the structured Postgres
audit itself.

What is missing: any *push* exporter. The clean shape is a fork-side job/daemon consuming
the same `audit_events` channel the SSE bus reads and forwarding to one sink:

- **OTLP exporter** (Datadog/LGTM both ingest OTLP): map audit rows to OTLP log records with
  trace ids carried through. ~1–2 wks.
- **Splunk HEC** is a simpler HTTP JSON post if Splunk is the stack. ~1 wk.

Decision needed: which stack. Building all three unprompted is waste.

## REQ-019 — Tenant-isolated semantic cache

Nothing exists. A real design needs: an embedding source (which itself is an AI call with
cost), a per-organization vector store partition, similarity + staleness thresholds, an
invalidation story, and a leakage proof (cross-tenant tests). It also changes response
semantics — a cached answer is not the model's answer to *this* prompt. Recommend an
explicit build-vs-integrate evaluation (e.g. against off-the-shelf semantic caches) before
committing. Build estimate if chosen: 4–6 wks.

## REQ-032 — Gateway provider failover

No failover exists in the gateway dispatch path (the agent AI service's fallback is a
different plane). Core change: `GatewayRoute` gains an ordered `fallback_providers` list;
dispatch retries eligible errors (connect/5xx/429-with-retry) against the next provider,
stamping the audit row with the provider actually used. Health-check-driven proactive
failover is a second step. Est 2–3 wks in core. Interim mitigation: flipping a route's
provider is a one-line profile edit.

## REQ-035 — A/B traffic splitting

Route resolution is strictly first-match. An experiment needs: a percentage/assignment
field on routes, sticky assignment (per user or per session, so a conversation does not
straddle variants), and variant stamped into `ai_requests` for measurement. Est 2 wks in
core. P3 — defer until a concrete experiment question exists.

## REQ-037/038 — Data classification, residency, and no-train enforcement

Two halves of one design:

1. A **classification label** on the request (header or route-match predicate) — Astound
   must define the taxonomy (e.g. `public | internal | customer | regulated`).
2. **Provider metadata** (`region`, `no_train: true`) on `ProviderEntry` — needs a core
   schema change (`deny_unknown_fields` blocks a YAML-only addition) — and a routing
   validator: a request labelled `regulated` may only resolve to providers whose metadata
   satisfies the policy, else 403 with an audit row.

Est 2 wks in core once the taxonomy lands. Until then residency is enforced by route
convention, which is real but unprovable.

## REQ-039/040 — Prompt registry (versioning, rollback, distribution)

Treat as one piece of work. The signed-manifest channel (skills) already solves
*distribution and revocation*; what is missing is the lifecycle object: a `prompt_templates`
table (id, version, body, params schema, created_by, activated_at), an activation pointer
per template (rollback = repoint), audit of which version served which request, and an
admin surface. The gateway's `SystemPromptOverride` engine is the natural enforcement
point. Design first; est 2–3 wks build after sign-off.

## REQ-022 — Per-key governance (virtual keys)

Keys collapse to their owner (`ApiKeyPrincipal { user_id }`) and `ai_gateway_policies`
rows are global, so no per-key budget/rate/model scope exists. Core change: make the key
a quota subject (`subject: api_key`, bucket keyed by key id) and let a key carry an
optional grants scope narrower than its owner's. Est 2 wks in core. First question back to
Astound: given org budgets, org rate windows, and revocable expiring PATs already bound the
blast radius, is per-key policy still required?

## REQ-025 — Grant expiry (`valid_until`)

`access_control_rules` is core-owned; adding `valid_until` touches its schema, the typed
`Rule`, the resolver (an expired rule stops matching), and the ACL loaders. Small, clean
core change (~3–4 days) — recorded here rather than forked so the fork does not diverge on
a security-critical table. Share-token expiry (the external-facing half of REQ-025) shipped
fork-side.

## REQ-026 — Audit immutability hardening

Options, cheapest first: (1) a dedicated DB role for the app with `REVOKE UPDATE/DELETE` on
audit tables + a separate maintenance role for retention; (2) hash-chaining a running
digest column per table; (3) periodic export to WORM object storage. Also needed
regardless: a retention policy for `ai_requests` (currently unbounded). Decision needed:
which compliance regime this must satisfy — it picks the option.

## REQ-007/008 — IDE acceptance telemetry and SCM ingest

Unchanged from [`design-org-hierarchy-and-scm.md`](design-org-hierarchy-and-scm.md): tab
acceptance has no data source on this platform; SCM commit ingest is blocked on naming the
authoritative SCM and the identity mapping.
