# Requirements Register — Delivery Status

**Basis:** the 43-row register (*[Enterprise AI] SystemPrompt requirements register*,
REQ-001–009 from a.nagornyy, REQ-010–043 from O. Manatskyi), assessed against this
repository at core 0.38.0, **assuming the current delivery pass completes**. That pass
adds: spend forecasting + early-warning alerts, scheduled cost digests, web CSV export,
configurable latency SLOs, persisted anomaly detection, share-token/grant expiry,
SSN/phone PII patterns, governance + quota enablement, Salesforce deprovisioning
reconciliation, and the AI Kit / AI SDLC Delivery organization split.

**Documentation & evidence (2026-08-25):** the full register is covered by nine
documentation pages under `/documentation/enterprise-*` (bucketed, with a per-page
"Verified evidence" section embedding the screenshot pack and `just e2e-req REQ-0NN`
replication commands), every REQ-001–043 carries a tagged Playwright spec (019/032/035/039
as honest `test.fixme` placeholders), and gateway-level proofs live in
`tests/integration/gateway` (`req_0nn_*`). REQ-037/038 are delivered via `governance:`
model metadata + route `requires:` (core change, patch-active — a core release is needed
before the patch can be retired).

Statuses use the vocabulary requested by b.gulyaev: **Testing/Delivered** means built and
evidenced here, awaiting Astound's acceptance testing. Evidence conventions are in
`requirements/compliance-register.md` (tagged Playwright specs + screenshot pack).

---

## 1. Completed (Testing/Delivered)

| REQ | Requirement | What is delivered |
|---|---|---|
| 001 | Admin user management | Full CRUD, role/status edit, disable/delete, invites, session + PAT revocation, last-active — all from the Admin UI. Three security defects fixed alongside (role-escalation guard, cross-org enumeration, unbound search). |
| 002 | Controlled registration | Enrolment closed on all three doors (core `allow_registration`, passkey self-reg, SSO auto-provision). Only path in is an admin invite (hashed, single-use, 7-day TTL). |
| 003 | Usage analytics dashboard | 5-tab SSR dashboard: request volume, error rate, active users, daily/weekly series, selectable periods (15m–30d + custom). |
| 004 | AI cost & model analytics | Total spend, daily cost trend, cost/request, token counters, model distribution, cost-by-model series, month-end P&L by org/provider/model. Microdollar integers end to end. |
| 005 | User adoption analytics | WAU with deltas, requests/user/day, top-user leaderboard, inactive-seat report with configurable window (7/14/30/90d). |
| 009 | Spend limits & budget monitoring | Hard cap **enforced** at the gateway (429 at org monthly cap), soft threshold recorded + Slack-alerted on first crossing, spend meters, burn-up chart, 12-month crossing history. |
| 010 | Real-time spend attribution | Every request carries user, org, department, model, provider, cost; near-real-time dashboards and reports slice on all of them. (Team/product/project = same open Hub decision as REQ-006.) |
| 011 | Spend overrun early warning | **New:** linear month-end projection computed on the request path; first projected-overrun crossing per org per month recorded and Slack-alerted. |
| 012 | Soft budget alerts | Soft threshold per org plan, non-blocking, Slack notification on transition (not per request). |
| 013 | Hard budget cutoffs | Dollar-based monthly hard cap per org plan, enforced with 429 + retry-after at the month boundary. Scope is organization (team/project pending Hub decision). |
| 014 | Burndown forecasting | **New:** burn-up chart now paired with projection-vs-cap early-warning alerts (see 011). |
| 015 | Self-service cost reporting | CLI reports with `--since/--until` + CSV export; **new:** CSV export endpoints in the admin web UI for cost/request reports, org-scoped. |
| 016 | Scheduled cost digests | **New:** scheduled digest job (weekly/monthly) delivering cost + budget-utilization summaries to Slack via the existing alerting transport. |
| 017 | Provider cost comparison | Cost/margin breakdown by provider and by model (internal report + CLI). Quality-normalization needs an agreed quality baseline — flagged in acceptance criteria. |
| 020 | Provider abstraction | Provider-neutral gateway (`/v1/messages`, `/v1/responses`), 4 wire protocols, `upstream_model` rewrite; live Cerebras/OpenAI/Gemini redirects prove substitution without app changes. |
| 021 | Central model access policy | `gateway_route` ACL grants per role/org/department, enforced at the gateway (403), auditable and revocable; admin UI for the access matrix. |
| 023 | Enterprise SSO provisioning | Salesforce OIDC + PKCE with gated JIT provisioning; **new:** scheduled reconciliation job — a user deactivated/removed in Salesforce is disabled here with sessions + PATs revoked. (SCIM is deliberately not built: Salesforce does not push standards-based SCIM and Odoo has none — documented as a future option for Okta/Entra.) |
| 024 | Shadow AI detection & blocking | Governed-side complete: model allowlist (`allow_unlisted_models: false`), route entitlements, full audit. Detection of traffic that never touches the gateway is a network-control matter outside the platform (noted for Astound IT). |
| 026 | Immutable AI audit trail | Every model call → `ai_requests` row (actor, model, provider, tokens, cost, latency, trace ids) with DB-locked actor attribution; governance decisions audited on the same spine. Caveat noted: append-only by convention, no WORM/hash-chain (see §3). |
| 027 | End-to-end traceability | One trace id correlates request → policy decision → model call → MCP/tool execution → cost; universal chain resolver in the admin UI. |
| 033 | Per-consumer rate limiting | Quota-window machinery (requests/tokens/cost per user or org) **now configured and enabled** in this pass, plus role-tier multipliers and per-org caps. |
| 034 | Private/self-hosted routing | Any OpenAI/Anthropic-compatible private endpoint is a config entry; `Backend` surface keeps private providers un-advertised; routes validated at boot. |
| 041 | Pre-execution tool governance | 4-stage synchronous chain (scope → secret scan → blocklist → rate limit) **enabled in this pass**, first-deny-wins, every decision audited with trace linkage. |
| 042 | Governed MCP server registry | Declarative registry with per-server auth/scopes/audience, plan-based entitlement, immediate revocation, JTI token revocation. |
| 043 | Tool schema validation | Registration fails fast on missing/invalid manifests and DB schema sync; tool input schemas captured at discovery. Caveat: no JSON-Schema meta-validation of tool contracts (see §2). |
| 037 | Data residency routing | **New:** provider/model YAML carries `governance: {european, no_retain}`; routes declare `requires:` and the server refuses to boot a route whose provider/model cannot satisfy it; dispatch re-checks and audits (`requires:` in the route-match descriptor). |
| 038 | No-train/no-retain enforcement | **New:** same `governance.no_retain` flag + route `requires: {no_retain: true}` — classified routes can only reach providers contractually marked no-retain; enforced at boot and at dispatch. Local route `claude-star-4203d1` demonstrates it. |
| — | Two-project split (b.gulyaev) | **New:** `ai-kit` and `ai-sdlc-delivery` organizations with own plans, seat limits, budget caps; org-scoped rosters/analytics/ACLs guarantee non-overlap. Onboarding = admin invites per group. |

## 2. Partially completed (delivered with named gaps)

| REQ | Delivered | Remaining gap |
|---|---|---|
| 006 | Drill-down by organization, department, individual user — applied identically across all dashboards. | Hub is now **defined as geographic** (Ed, 2026-08-25); the dimension is implementable and scheduled as the next pass (design in `requirements/design-org-hierarchy-and-scm.md`, read "Hub" as geographic). Team/project remain undefined. |
| 007 | AI-authored LOC, applied edits, permission-grant rate, commit lines — shipped, honestly labelled as proxies. | Tab-acceptance rate is **not feasible as specified**: Claude Code emits no accept/reject signal; no manual-LOC baseline exists. Needs an IDE-level integration to ever exist. |
| 008 | Commits observed through Claude Code sessions, deduped, rolled up daily, plotted beside AI usage. | Commits made outside Claude Code are invisible. Blocked on Astound nominating the authoritative SCM + identity mapping. |
| 018 | Declarative routing on request shape; programmatic `RouteSelector` seam; full cost/latency telemetry to decide with. | No automatic optimizer that scores cost/quality/latency. Needs an agreed quality baseline first (same dependency as 017). |
| 022 | PATs/API keys: revocable, optional expiry, prefix-identified; org budgets + quota buckets bound the blast radius of any key. | A key is not its own governance subject — no per-key budget/rate/model scope (keys collapse to the owning user). Real fix is a core change; scoped in the design note. |
| 025 | **New:** share-token expiry + grant `valid_until` handling; invites/setup-tokens/JWTs already time-bound. | No forced maximum PAT lifetime policy; contractor governance still leans on account-level controls. |
| 028 | OTLP **ingest** (traces/logs/metrics), live JSON audit stream over SSE, structured Postgres audit. | No push egress to Datadog/Splunk/OTEL collector. Scoped as an integration design (`requirements/design-finops-and-observability.md`). |
| 029 | **New:** configurable latency SLO threshold with breach % reporting, p50/p95, per-model latency data. | No per-use-case SLO taxonomy or error-budget alerting — needs Astound's SLO definitions. |
| 030 | Central safety chain **enabled**: jailbreak heuristics, PII (email/CC/**new:** SSN/phone), secret egress scanning; buffered responses can block. | No toxicity/brand-risk classifier; streamed egress is audit-only by design; full taxonomy needs a decision on which categories Astound requires blocked. |
| 031 | **New:** persisted anomaly job over cost/volume/error spikes with Slack alerts + dashboard surface. | Simpler thresholds than a learned baseline; tuning iterates with real traffic. |
| 036 | 34 secret patterns + entropy backstop; PII email/CC/**new:** SSN/phone; display-layer redaction in transcripts. | Enforcement **blocks rather than redacts** in-flight; no PHI taxonomy (health identifiers). Full redaction pipeline is a scoped design item. |
| 040 | Signed, centrally-revocable distribution exists (skills/plugins over Ed25519-signed manifests) and can carry prompt content. | No versioned prompt-template object with parameters/pinning/rollback (see 039). |

## 3. Not delivered (scoped only — needs a product decision or larger design)

| REQ | Why not built now | What it would take |
|---|---|---|
| 019 | Tenant-isolated semantic caching | No semantic cache exists anywhere in the gateway. Needs embedding infrastructure, similarity thresholds, and a tenant-isolation proof — a project of its own. Evaluate build vs. integrate. |
| 032 | Automatic provider failover | **Register correction: this is a Gap, not Partial** — no failover exists in the gateway dispatch path (the "fallback" the earlier assessment saw belongs to the agent AI service). Needs secondary-provider fields on routes + health/error-triggered retry in core. |
| 035 | A/B model testing on live traffic | Route resolution is strictly first-match; no percentage split or experiment assignment exists. Moderate core change; deprioritized as P3. |
| 039 | Prompt versioning & rollback | No prompt registry exists; the nearest primitive (system-prompt overrides) has no version history. Needs a first-class lifecycle object — design proposed before building. |
| 026 (hardening) | True immutability | Audit rows are append-only by convention, not by mechanism. Options scoped: `REVOKE UPDATE/DELETE` + append-only role, hash-chaining, or export-to-WORM. Also: no retention policy on `ai_requests` yet. |
| 023 (SCIM) | Generic IdP + SCIM | Deliberately deferred: neither Salesforce (as IdP) nor Odoo speaks SCIM push, so an endpoint would have no caller. Revisit if Okta/Entra fronts the instance. |

## Decisions Astound owes (blocking the remaining rows)

1. ~~Define "Hub"~~ Decided: geographic. Implementation is the next pass; team/project dimensions remain open.
2. **Nominate the authoritative SCM + identity mapping** — unlocks full commit analytics (008).
3. ~~Sign off the safety category set~~ Decided: ship the enabled default set (jailbreak, secrets, PII email/CC/SSN/phone; ingress-block + secret/SSN egress) and the enabled quota sizes; Astound tunes from real traffic rather than pre-approving (030/033/036).
4. ~~Provide a quality baseline~~ Decided: sensible defaults now, revisit with Astound when a quality signal exists; cost/latency comparison ships without normalization (017/018).
5. ~~Data-classification scheme~~ Decided: classification is carried as `governance:` booleans (`european`, `no_retain`) on gateway provider/model YAML, with route-level `requires:` enforced at boot validation and at dispatch (audited). Landed in core (patch-active) + both profiles — 037/038 unblocked.
6. **Confirm the deployed build** — several rows the register marks Partial were assessed against an older deployment.
7. **Fresh sheet export** — the Google Sheet has been extended past REQ-043; new rows need a re-export to be assessed.

---
*Companion documents: `requirements/compliance-register.md` (per-row evidence),
`requirements/design-org-hierarchy-and-scm.md` (Hub + SCM design),
`requirements/design-finops-and-observability.md` (scoped not-built items).*
