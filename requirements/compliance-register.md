# Enterprise Requirements Register — Compliance Status

Traceability for the requirements register supplied by Astound Digital
(*[Enterprise AI] SystemPrompt requirements register*): REQ-001…009 requested by
a.nagornyy@astounddigital.com, REQ-010…043 by O. Manatskyi. The updated register
CSV with per-row status and comments is `register-updated-2026-08-25.csv`; the
executive bucketing is `../REQUIREMENTS-DELIVERY-STATUS.md`.

This file is the answer to that register. It is scoped to the **admin and governance
surface**; [`requirements.md`](requirements.md) is the earlier POC scope (skills, harness
delivery, Salesforce, RAG) and is deliberately not renumbered into REQ ids.

**How a row is signed off.** A requirement is `Met` only when a tagged Playwright spec
asserts it against the deterministic seed *and* a named screenshot exists. "It looks right
in a browser" is not evidence — the register exists because that standard already produced
a search box that did nothing and a test that passed anyway.

```bash
just start && just e2e-seed --reset
just e2e -- --grep @REQ-003      # prove one row
just e2e-screens                 # regenerate the evidence pack
```

## Summary

| REQ | Requirement | Priority | Register's assessment | Status here |
|-----|-------------|----------|-----------------------|-------------|
| 001 | Admin user management | P0 | Partial | **Met** |
| 002 | Controlled user registration | P0 | Requires configuration | **Met** |
| 003 | Usage analytics dashboard | P1 | Partial | **Met** |
| 004 | AI cost & model analytics | P1 | Partial | **Met** |
| 005 | User adoption & utilization | P1 | Partial | **Met** |
| 006 | Organizational drill-down | P1 | Gap / unclear | **Partial** — org, department and user only |
| 007 | AI development productivity | P2 | Gap | **Not feasible as specified** |
| 008 | Commit activity analytics | P2 | Not native | **Partial** — Claude Code commits only |
| 009 | Spend limits & budget monitoring | P1 | Partial | **Met** |


Rows 010–043 (assessed 2026-08-25 against core 0.38.0; sections below):

| REQ | Requirement | Status here |
|-----|-------------|-------------|
| 010 | Real-time spend attribution | **Met** (org/department/user/model/provider) |
| 011 | Spend overrun early warning | **Met** — forecast projection + Slack alert (new) |
| 012 | Soft budget alerts | **Met** |
| 013 | Hard budget cutoffs | **Met** (organization scope) |
| 014 | Burndown forecasting | **Met** (new, with 011) |
| 015 | Self-service cost reporting | **Met** — CLI + new web CSV export |
| 016 | Scheduled cost digests | **Met** — new `cost_digest` job |
| 017 | Provider cost comparison | **Met** (cost side; quality baseline pending) |
| 018 | Cost/quality/latency optimization | **Partial** — routing seam, no optimizer |
| 019 | Tenant-isolated semantic caching | **Gap** |
| 020 | Provider abstraction | **Met** |
| 021 | Central model access policy | **Met** |
| 022 | Governed virtual keys | **Partial** — no per-key budget/rate (core change) |
| 023 | Enterprise SSO provisioning | **Met** — new deprovisioning reconciliation |
| 024 | Shadow AI detection | **Met** governed-side; non-gateway traffic is a network control |
| 025 | Time-bound external access | **Partial** — share-token expiry new; grant expiry needs core |
| 026 | Immutable audit trail | **Met** with caveat — append-only by convention, no WORM |
| 027 | End-to-end traceability | **Met** |
| 028 | Observability export (OTEL) | **Partial** — ingest + SSE stream, no SIEM egress |
| 029 | Latency SLOs | **Met** — configurable threshold + breach share (new) |
| 030 | Content safety guardrails | **Partial, enabled** — jailbreak/PII/secrets on; no toxicity taxonomy |
| 031 | Usage anomaly detection | **Met** — persisted hourly sweep + alerts (new) |
| 032 | Automatic provider failover | **Gap** (register correction: was "Partial") |
| 033 | Per-consumer rate limiting | **Met** — quota windows configured + enabled (new) |
| 034 | Private/self-hosted routing | **Met** |
| 035 | A/B model testing | **Gap** |
| 036 | PII/PHI detection & redaction | **Partial, enabled** — +SSN/phone (new); no PHI taxonomy |
| 037 | Data residency routing | **Met (new)** — `governance:` model metadata + route `requires:` enforced at boot and dispatch |
| 038 | No-train/no-retain enforcement | **Met (new)** — `governance.no_retain` + `requires: {no_retain: true}` on routes |
| 039 | Prompt versioning & rollback | **Gap** |
| 040 | Central prompt distribution | **Partial** — signed skills channel, no template lifecycle |
| 041 | Pre-execution tool governance | **Met** — 4-stage chain now enabled |
| 042 | Governed MCP registry | **Met** |
| 043 | Tool schema validation | **Met** with caveat — no JSON-Schema meta-validation |

> **The register's "Current Assessment" column describes a deployed environment, not this
> repository.** Most of the analytics and user-management surface it calls missing was built
> in `c403510d` and `c057bf28`. Where Astound cannot see a feature listed here as Met, the
> first question is which build is deployed.

---

## REQ-001 — Admin User Management (P0) · **Met**

> Admin can search/list users, create users, modify role/status, disable/delete access,
> review basic account activity, and revoke sessions without CLI or database access.

| Capability | Where |
|---|---|
| List / group by department | `GET /admin/access/users` → `handlers::ssr::ssr_users`, `users.hbs` |
| Search | `#user-search` in `users.hbs`, bound by `admin-users.js` |
| Create (with department + validated roles) | `POST /api/public/admin/users` → `handlers::users_bootstrap` |
| Update, disable/enable | `PUT /api/public/admin/users/{user_id}` → `handlers::users::update_user_handler` |
| Delete | `DELETE /api/public/admin/users/{user_id}` |
| Invite / revoke / regenerate | `/api/public/admin/invites*` → `handlers::invites` |
| Org & department moves | `PUT /api/public/admin/management/users/{user_id}/{organization,department}` |
| Account activity | `GET /admin/access/user?id=…`, `GET /admin/analytics/users/{user_id}` |
| **Revoke one session** | `DELETE /api/public/admin/users/{user_id}/sessions/{session_id}` |
| **Revoke every session** | `DELETE /api/public/admin/users/{user_id}/sessions` |
| **Revoke a device token** | `DELETE /admin/devices/pats/{id}`, now surfaced on user detail |

**What changed to close this row.** Session revocation existed only as
`systemprompt admin users session end` — the `user_sessions.revoked_at` column and its index
were present in core and nothing in this fork ever wrote them. It is now
`repositories::users::sessions`, mounted beside the user routes and surfaced on the user
detail page. Revoking is an `UPDATE`, never a `DELETE`: the row carries that session's
request counts and cost, which the analytics rollups read.

Four adjacent defects were fixed in the same pass, none of which the register asked about:

1. **Privilege escalation.** Create and invite both refused to assign a non-`user` role
   unless the caller was a platform admin; `update_user_handler` did not, and
   `mutations.rs` wrote `roles = COALESCE($4, roles)` unconditionally. A customer's own org
   admin could therefore grant `admin` by *editing* a user they could not have *created*.
2. **Cross-organization enumeration.** `list_users` had no org filter, so any `admin` — on a
   pooled multi-tenant instance — saw every user on it. Scoping now runs through
   `util::org_scope::listing_scope`, which returns an empty slug rather than `None` for an
   unattached admin: `None` means "every organization", so falling back to it would have
   widened the very listing the function narrows.
3. **A search box wired to nothing.** `users.hbs` rendered `#user-search`; no JavaScript
   bound it. `users-roster.spec.ts` filled it and asserted the matching row was still
   visible — which passes whether or not filtering happens. The control is now bound and the
   spec asserts non-matching rows *disappear*.
4. **Two dead surfaces on user detail.** The template rendered `user_tokens`, which nothing
   produced, so the tokens card always showed its empty state — while `user_devices` was
   loaded on every request and never rendered. The card now renders devices, with the revoke
   button whose endpoint already existed.

Also: `last_active` was `COALESCE(…, u.created_at)`, so a user who had never done anything
reported their join date as their last activity — making "provisioned but never used"
indistinguishable from "used the day they joined", which is exactly the population REQ-005
reports on. It is now nullable and renders "Never".

And the roster had **no sidebar link at all** for an org admin — the surrounding nav block
was platform-admin-only, leaving a breadcrumb on a page they had no way to reach. There is
now a *People* section.

- **Specs:** `@REQ-001` in `playwright/tests/requirements/req-001-user-management.spec.ts`, plus
  `users-roster.spec.ts`, `user-org-membership.spec.ts`
- **Screenshots:** `evidence/req-001-users-roster.png`, `evidence/req-001-user-detail.png`

---

## REQ-002 — Controlled User Registration (P0) · **Met**

> An unapproved user cannot create an account or connect a Bridge.

**This row was *not* met before this change**, despite the register's more optimistic
"authentication and role controls exist". Three doors stood open:

- `POST /admin/auth/passkey/register` was public and gated **only** by an email-domain
  suffix match, with `login.hbs` rendering a "New here? Create an account" button onto it.
  Anyone holding an `@astounddigital.com`, `@astoundcommerce.com` or `@systemprompt.io`
  address could self-create an account with no approval step.
- `auto_provision: true` meant a first Salesforce login self-provisioned too.
- `security.allow_registration: true` in the local profile re-opened core's own WebAuthn
  register endpoints.

An allow-listed domain establishes that an address *could* belong to someone who should have
access. It never establishes that anyone approved them. Enrolment is now closed:

| Control | Setting |
|---|---|
| Passkey self-registration | `allow_self_registration: false` — `services/web/config/salesforce.yaml` |
| SSO just-in-time provisioning | `auto_provision: false` — same file |
| Core's own WebAuthn register | `security.allow_registration: false` — profile |
| The only remaining path in | Admin-issued invite: hashed, single-use, 7-day TTL |

The invite carries its own authorization, so it bypasses the domain list entirely
(`extensions/web/schema/18_user_invites.sql`). Admin-created accounts already mint a
seven-day sign-in link, and a lost one is recoverable by regenerating, so no one is
stranded. The login page renders "Ask an administrator for an invite link" rather than
advertising a door the server now refuses.

**Onboarding consequence, stated plainly:** every new user needs an admin to invite them.
This is the deliberate trade the requirement asks for.

**Bridge connect.** The mechanics were already sound — single-use codes, 10-minute TTL,
stored as SHA-256. The problem was *when* they were issued: `GET /admin/profile` minted a
fresh one into the HTML on **every page load**, for any authenticated user. Redeeming a code
yields a durable PAT signing in as its owner, so that put a live bearer credential on a page
people leave open, reload and screen-share, and burned a code on the majority of views that
never connect a machine. Issuance is now an explicit act:
`POST /admin/api/profile/bridge-code`, scoped to the caller with no target-user parameter.

> **One plan item deliberately not implemented.** The plan also proposed putting
> `/bridge-auth/device-link/approve` behind an admin gate. With enrolment closed, every
> account holder *is* an approved user, so the transitive argument the requirement relies on
> now holds — and gating it would break the documented `just connect` onboarding for every
> ordinary developer, which the POC scope explicitly wants friction-free. Raised for
> Astound's decision rather than changed unilaterally.

**Deployment step (not tracked in git):** `.systemprompt/profiles/local/profile.yaml` is
gitignored and generated by `setup-local`. New clones must set
`security.allow_registration: false`; the production profile already has it.

- **Specs:** `@REQ-002` in `playwright/tests/requirements/req-002-controlled-registration.spec.ts` — out-of-domain
  email refused, in-domain email refused, invite path succeeds
- **Screenshots:** `evidence/req-002-login-invite-only.png`

---

## REQ-003 — Usage Analytics Dashboard (P1) · **Met**

> Dashboard shows daily/weekly requests, total request volume, active users, and historical
> usage trends with selectable time periods.

`GET /admin/analytics` — `handlers::ssr::ssr_analytics_dashboard`, `analytics-dashboard.hbs`.

| Required | Where |
|---|---|
| Total request volume, error rate, active users | `analytics::site::kpis::get_site_kpis` (with prior-window deltas in the same statement) |
| Daily / weekly series | `analytics::site::series::list_daily_usage_series` — `DATE_TRUNC` over a zero-filled `generate_series` spine |
| Selectable period | Preset picker `15m · 1h · 24h · 7d · 30d · custom` (`util::time_range`) |
| Daily/weekly toggle | `SeriesBucket`, `urls::bucket_links` |

Charts are server-rendered SVG — geometry computed in Rust, no client charting library.

> **Open question for Astound.** `MAX_CUSTOM_WINDOW_DAYS = 30` clamps every query, so there
> is no quarter or year view. Confirm 30 days satisfies "historical usage trends"; widening
> it is a deliberate decision about query cost, not an oversight.

- **Specs:** `@REQ-003` · **Screenshots:** `evidence/req-003-usage-trends.png`, `evidence/req-003-overview.png`

---

## REQ-004 — AI Cost & Model Analytics (P1) · **Met**

> Total spend, spend trend, cost/request, token/request activity, and usage distribution by
> AI model.

| Required | Where |
|---|---|
| Total spend, cost per request | `site::kpis::get_site_kpis`, rendered by `view::kpi_strip` |
| Daily cost trend | `site::series` → `view::spend_chart` |
| Token counters | `SUM(input_tokens + output_tokens)` across kpis, distribution, leaderboard |
| Distribution by model | `site::distribution::list_model_distribution` (pie) |
| Cost by model over time | `site::model_series::list_model_cost_series` — top 6 by cost, tail folded to `Other` |
| Month-end P&L, cost by provider/model | `/admin/reports/internal` → `repositories::reports::internal` |

Costs are `BIGINT` microdollars end to end — no floating point anywhere in aggregation.

- **Specs:** `@REQ-004` · **Screenshots:** `evidence/req-004-spend.png`, `evidence/req-004-model-mix.png`

---

## REQ-005 — User Adoption & Utilization (P1) · **Met**

> WAU, requests/user/day, top users, and inactive users over a configurable period.

| Required | Where |
|---|---|
| Active seats vs limit | `organizations::metrics::list_organization_metrics` |
| Weekly active users | `site::kpis` — `COUNT(DISTINCT user_id) FILTER (created_at >= NOW() - 7d)`, with a 14-day prior window for the delta chip |
| Requests per user per day | `view::kpi_strip`, and per-row on the leaderboard |
| Top users | `site::leaderboards::list_top_users_by_requests` — paged, sortable by requests/cost/tokens/last-active |
| **Inactive users, configurable window** | `site::seats` — `?inactive_days=` (7/14/30/90 picker on the Seats tab), default 30 |

**What changed.** The window was an `INTERVAL '30 days'` literal in *both* seat queries and
again in three places of template copy, so "a configurable period such as 30 days" was
false. It is now a bound day count multiplied into an interval via `make_interval(days => $3)`
— a caller-supplied interval *string* would be an injection seam for no benefit — clamped to
1…365 so an edited URL shows the default rather than an error page. The copy interpolates the
resolved window, so it can no longer contradict the query.

- **Specs:** `@REQ-005` · **Screenshots:** `evidence/req-005-seats.png`, `evidence/req-005-inactive-seats.png`

---

## REQ-006 — Organizational Drill-Down (P1) · **Partial**

> Filter and aggregate by team/group, individual user, Hub, and potentially project.

**Available today — three dimensions,** via `SiteScope` (`analytics::site::mod`), applied
identically in every dashboard query and reflected in removable filter chips:

| Dimension | Parameter | Source |
|---|---|---|
| Organization | `?org=` | `organizations.slug` (platform admins only; others are pinned to their own) |
| Department | `?department=` | `user_profile_ext.department` |
| Individual user | `?user_id=`, or `/admin/analytics/users/{user_id}` | `ai_requests.user_id` |

**Not available: team/group, project, Hub.** No such table, column, parameter or widget
exists. Adding one means a new band in the resolver
(organization 300 → role 200 → department 100 → user 0), touching `authz/mod.rs`,
`services/access-control/plans.yaml` and the ACL loader. **This is blocked on Astound
defining what a Hub is** — whether it nests inside an organization, whether a user belongs to
one or many, and whether it grants entitlements or is only a reporting label. A design note
and estimate follow that definition; building before it would guess at the identity model.

**`tenant_id` is not an available dimension, and this is deliberate upstream.** Core
migration `003_drop_runtime_tenancy.sql` dropped it from `ai_requests` because it was never
populated with a real value. Any reporting model Astound proposes should key on organization,
not tenant.

Two scoping defects found and fixed while auditing this row:

- `list_department_names` was not org-scoped, so an org admin's department dropdown listed
  every *other* customer's department names. The rows behind the filter were correctly
  scoped; the option labels leaked structure.
- `resolve_scope` fell back to `None` — meaning *every* organization — for an admin with no
  organization membership. The widest possible scope was reached by having the least
  attachment. It now resolves to an empty slug, which matches nothing.

Still open: `/admin/entities/requests` filters by user, agent, model, provider and status but
**not** by org or department, so request-log drill-down is dashboard-only. The `analytics`
CLI has no org dimension at all.

- **Specs:** `@REQ-006` (the three real dimensions) · **Screenshots:** `evidence/req-006-drilldown.png`, `evidence/req-006-user-drilldown.png`

---

## REQ-007 — AI Development Productivity Metrics (P2) · **Not feasible as specified**

> AI suggestion/Tab acceptance rate, total AI accepts, AI-generated LOC versus manual LOC.

**Tab-acceptance rate has no data source on this platform, and cannot be given one from
here.** Claude Code emits no accept/reject signal. `bridge/src/` is a 60-line white-label
shim over core's bridge; all telemetry is captured server-side from Claude Code's hook
events, and no hook carries an acceptance decision. The product surface already says so
verbatim in `view_code.rs`:

> *"Applied edits — Claude Code emits no accept/reject signal, so there is no acceptance rate
> to report."*

**What is real, and is shipped labelled as a proxy:**

| Measurement | Source |
|---|---|
| AI lines added / removed | `hooks_track::loc` — counts Write/Edit/MultiEdit/NotebookEdit deltas *pre-sanitize*, into `plugin_usage_events.loc_added/loc_removed` |
| Applied edit operations | same event stream |
| Permission-grant rate | `kpis::get_permission_grant_stats` — a `LEAD()` window over PreToolUse/PostToolUse pairs |
| Committed lines | `user_commits` (see REQ-008) |

AI lines and committed lines are **different measurement frames and are never subtracted**.
The Code tab labels them as such rather than presenting a synthetic "AI vs manual" ratio,
which would be a number with no defensible definition.

Closing this row properly requires an IDE-level integration that emits accept/reject events —
a separate piece of work against a different data source, not a gap in this platform.

- **Screenshot:** `evidence/req-007-008-code-tab.png` (evidence of the labelling, not of compliance)

---

## REQ-008 — Commit Activity Analytics (P2) · **Partial**

> Commit activity viewable for the same user/team/project/time period as AI usage.

**Working today:** `user_commits` (`extensions/web/schema/17_usage_metrics.sql`) holds one row
per commit observed through a Claude Code Bash tool call — hash, message, branch, cwd, files
changed, insertions, deletions — parsed by `handlers::hooks_track::commits`, deduped on the
`(user_id, cwd, commit_hash)` unique index, and rolled into `admin_usage_daily_rollups` by the
hourly `usage_daily_rollup` job. The Code tab plots it against AI line deltas over the same
window and scope.

**The gap:** commits made *outside* Claude Code are invisible. Ingesting them means a webhook
upserting into `user_commits` on that same unique index, so hook-observed and SCM-observed
commits converge rather than double-count. The hard part is not the ingest — it is identity:
mapping an SCM login or commit-author email to `users.id`. That mapping needs a decision from
Astound about which SCM is authoritative and how developers' accounts are linked, and is
scoped in the design note alongside REQ-006.

- **Screenshot:** `evidence/req-007-008-code-tab.png`

---

## REQ-009 — Spend Limits & Budget Monitoring (P1) · **Met**

> Current spend versus budget, threshold utilization, and alerts/actions when configured
> limits are reached.

More is enforced here than the register assumes — this is not display-only.

| Required | Where |
|---|---|
| Hard cap **enforcement** | `gateway_org_budget::OrgBudgetGuard` — denies with **429** at the org's monthly cap |
| Model-tier enforcement | `RouteEntitlementGuard` — 403 for models outside the plan's grants |
| Soft threshold | `plans.monthly_cost_warn_usd`, recorded in `org_budget_warnings` |
| Spend vs budget | Per-org meters (`spend-meter.hbs`), fill = MTD % of cap, tick = soft cap |
| Burn-up vs both caps | `view_spend::burndown_chart`, with a linear pace projection |
| Crossing history | `budget_warnings::list_budget_warning_history` — 12 months |
| **Alerting** | `slack_alerts::send_alert`, fired on the first crossing each month |

**What changed.** `slack_alerts::send_alert` was written but had **zero call sites**, and
`send_to_slack` was a stub that only logged "integration not yet implemented" — so nothing
notified anyone. It now posts to Slack `chat.postMessage`, and is wired to the soft-cap
crossing.

Two details that matter more than the wiring:

- **It alerts on the transition, not the state.** The guard calls the recorder on *every*
  request once spend is past the threshold, so alerting unconditionally would post once per
  request for the rest of the month. The upsert now returns `(xmax = 0)` — true only on its
  INSERT arm — which is precisely the first crossing.
- **A Slack 200 is not success.** Slack answers `200` with `{"ok": false, "error": …}` for
  bad channels and revoked tokens, so the response body is checked. Delivery failures warn
  and never propagate: this runs on a detached task off the gateway's request path, and a
  Slack outage must not look like a budget-guard failure.

Configuration is `services/access-control/plans.yaml` (`monthly_cost_cap_usd`,
`monthly_cost_warn_usd`, validated so warn < cap). Alerting needs `slack_bot_token` and
`activity_report_slack_channel` in profile secrets; a `REPLACE_WITH_*` placeholder counts as
absent, so an unconfigured install stays quiet rather than logging a 401 per alert.

**Scope of budgets, stated precisely** — the requirement says "global versus user/team/project
thresholds" and only the first is true:

| Level | Status |
|---|---|
| Per organization | Cap enforced, soft warn recorded, alerted, metered, 12-month history |
| Per user | **Absent.** `ai_quota_buckets` is keyed per user but no window is configured |
| Per team / project | **Absent** — those dimensions do not exist (REQ-006) |
| Global / instance-wide | **Absent** as a cap. `quota_windows` is unset in `services/gateway/policies.yaml` and governance `rate_limit` is `enabled: false` |
| Seat limits | Displayed, and enforced at provisioning; not a spend control |

> The unset `quota_windows` and disabled governance chain are **deliberate** in this
> installation (see `CLAUDE.md` and the header comments in both config files). Turning them
> on is a separate, explicit decision and was not folded into this change.

- **Specs:** `@REQ-009` · **Screenshots:** `evidence/req-009-spend-meters.png`

---


---

# REQ-010…043 — assessment and delivery notes (2026-08-25)

Assessed against HEAD at core 0.38.0. "New" marks work delivered in this pass.

## FinOps (REQ-010…017)

- **010 Spend attribution — Met.** Every `ai_requests` row carries user, org (via
  `organization_members`), department (`user_profile_ext`), model, provider,
  `cost_microdollars`; dashboards and reports slice on all of them
  (`repositories/analytics/site/`, `repositories/reports/`). Team/product/project are the
  REQ-006 Hub decision.
- **011/014 Forecasting — Met (new).** `gateway_org_budget::projected_month_end_spend`
  computes a seconds-based linear projection on the request path; the first
  projected-overrun crossing per org per month lands in `org_budget_warnings`
  (`kind='forecast_overrun'`, migration 031) and fires the Slack alert. Surfaced in the
  spend tab's warning table. No projection before three elapsed days — early-month noise
  must not train people to ignore the alert.
- **012/013 Budgets — Met.** Soft warn + hard cap per org plan
  (`services/access-control/plans.yaml`), enforced by `OrgBudgetGuard` (429 + retry-after at
  the month boundary). Organization scope; team/project awaits Hub.
- **015 Self-service reporting — Met (new).** CLI already had `--since/--until` + CSV;
  the admin UI now exports CSV: `/admin/reports/internal.csv?month=&dimension=organization|provider|model`
  (platform admin) and `/admin/reports/customer.csv?month=&org=&dimension=users|departments|models`
  (org-scoped), with download buttons on both report pages. Integer-microdollar precision
  survives the export.
- **016 Digests — Met (new).** `cost_digest` job (weekly, Monday 08:00 UTC,
  `extensions/web/jobs/src/cost_digest.rs`): per-org MTD spend vs cap, trailing-week spend
  and requests, linear pace — delivered through `slack_alerts`. The orphaned
  `daily_summary` module (declared a repository that did not exist, wired to nothing) is
  deleted.
- **017 Provider comparison — Met, cost side.** Internal report + CLI break cost down by
  provider and model. "Quality-normalized" needs an agreed quality baseline from Astound;
  the acceptance criterion is untestable without one.

## Optimization & access (REQ-018…025)

- **018 Model optimization — Partial.** Declarative `RouteMatch` predicates + the
  `RouteSelector` inventory seam exist in core; no optimizer scores cost/quality/latency.
  Blocked on the same quality baseline as 017.
- **019 Semantic cache — Gap.** Nothing in the gateway; provider prompt-caching
  passthrough is not a tenant cache. Scoped in the design note.
- **020 Provider abstraction — Met.** `/v1/messages` + `/v1/responses`, four wire
  protocols, `upstream_model` rewrite; production routes already redirect claude-* traffic
  to alternate providers with zero app changes.
- **021 Model access policy — Met.** `gateway_route` entities + plan/role/department ACL
  grants, enforced by `RouteEntitlementGuard` (403), auditable and revocable centrally.
- **022 Virtual keys — Partial.** PATs revoke and expire (`user_api_keys`), org budgets and
  quota buckets bound the damage — but a key is not a governance subject
  (`authenticate_api_key` collapses to `user_id`; `ai_gateway_policies` rows are global).
  Per-key budget/rate/scope needs a core change; scoped in the design note.
- **023 SSO provisioning — Met (new).** Salesforce OIDC + PKCE with gated JIT provisioning
  was already in; offboarding is now closed by `salesforce_deprovision`
  (`extensions/web/jobs/src/salesforce_deprovision.rs`, half-hourly): accounts whose
  Salesforce user is deactivated/deleted are disabled with sessions and PATs revoked, via
  the existing `SF_TARGET_*` JWT-bearer integration identity. SCIM was deliberately not
  built — Salesforce does not push standards SCIM to third-party apps and Odoo has none, so
  an endpoint would have no caller; revisit if Okta/Entra ever fronts the instance.
- **024 Shadow AI — Met, governed side.** `allow_unlisted_models: false`, route
  entitlements, full audit. Detecting AI traffic that never reaches the gateway is an
  egress/network control outside this platform — named for Astound IT rather than
  half-built here.
- **025 Time-bound access — Partial (new).** Share tokens now expire (30-day TTL inside the
  HMAC payload, `handlers/share.rs`; the 4-part token shape is pinned by the contract
  suite, expiry tamper included). Invites (7d), setup tokens (10m), exchange codes and JWTs
  were already bounded; PAT expiry remains optional. Grant-level `valid_until` needs a
  column and resolver change in core's `access_control_rules` — recorded as a core
  follow-up rather than forked.

## Observability & safety (REQ-026…031)

- **026 Audit trail — Met with caveat.** Full per-call audit rows with DB-locked actor
  attribution; append-only by convention, not mechanism (no REVOKE/WORM/hash chain), and no
  retention policy on `ai_requests`. Hardening options in the design note.
- **027 Traceability — Met.** `trace_id`/`session_id`/`mcp_execution_id` on every row;
  `ssr_chain` resolves any of them to the full chain.
- **028 Observability export — Partial.** OTLP ingest and the SSE audit stream exist;
  no push egress to Datadog/Splunk/OTEL collector. First exporter to build once Astound
  names the target stack.
- **029 Latency SLOs — Met (new).** The 5s constant is now a clamped `?slo_ms=` threshold
  (500–60000, picker on the spend tab), with breach share reported beside p50/p95 —
  same bound-parameter pattern REQ-005's window used. Per-use-case SLO taxonomy needs
  Astound's SLO definitions.
- **030 Safety guardrails — Partial, now enabled.** `services/gateway/policies.yaml`
  runs `[heuristic, secrets, pii_extended]`, blocking jailbreak/credit-card/SSN on ingress
  and secrets/SSN on buffered egress; streamed egress stays audit-only by design.
  pii_email/pii_phone are audit-only deliberately. No toxicity/brand-risk classifier —
  extend once Astound signs off the category set.
- **031 Anomaly detection — Met (new).** `usage_anomaly` job (hourly,
  `extensions/web/jobs/src/usage_anomaly.rs`): requests/cost/errors vs the trailing-week
  hourly baseline with multiplier + floor thresholds, persisted to `usage_anomalies`
  (migration 032) so restarts cannot forget an incident, Slack-alerted on first detection,
  surfaced on the spend tab. Thresholds will need tuning against real traffic.

## Gateway & lifecycle (REQ-032…043)

- **032 Provider failover — Gap (register correction).** The register said Partial; no
  failover exists in the gateway dispatch path — the "fallback" config it saw belongs to
  the agent AI service. Needs secondary-provider route fields + retry in core (est 2–3
  wks). Interim: a route flip is a one-line profile change.
- **033 Rate limiting — Met (new config).** Core quota machinery was unconfigured here;
  `policies.yaml` now carries per-user hourly (600 req / $20) and per-organization daily
  ($200) windows as a runaway backstop above the plan budgets, plus the existing role-tier
  multipliers. Sizes are first-pass — review against real traffic.
- **034 Private routing — Met.** Any compatible endpoint is a `ProviderEntry`; `Backend`
  surface keeps it un-advertised; boot validates route→provider references.
- **035 A/B testing — Gap.** First-match routing only; no split/assignment machinery. P3 —
  defer until a concrete experiment exists.
- **036 PII/PHI — Partial, now enabled (new).** 34 secret patterns + entropy backstop
  (ingress, governance) — note: the register's "35+" was optimistic and none are PHI.
  New `pii_extended` scanner adds conservative SSN + E.164 phone detection both directions
  (`gateway_safety.rs`), and the transcript redactor masks SSNs. Enforcement blocks rather
  than redacts in flight; PHI taxonomy and true redaction are scoped items.
- **037 Data residency — Met (new).** Provider/model YAML now carries
  `governance: {european, no_retain}` (`ModelGovernance`, core patch); routes declare
  `requires:` (`RouteRequirements`) and `GatewayConfig::validate` refuses to boot a route
  whose reachable provider/model cannot satisfy it. Dispatch re-checks (selector-refined
  routes and unlisted models included) and records `requires:...` in the route-match
  descriptor on the audit row. Classification decision per Ed: the scheme IS these model
  booleans — no separate taxonomy is owed.
  **Specs:** `req_037_residency.rs` in `tests/integration/gateway`; `@REQ-037` in
  `playwright/tests/requirements/req-037-038-residency.spec.ts`.
- **038 No-train enforcement — Met (new).** Same mechanism: `governance.no_retain` on the
  provider (model-level override wins), route `requires: {no_retain: true}`. The local
  profile's `claude-star-4203d1` route carries it as the live demonstration.
  **Specs:** `req_038_no_retain.rs` in `tests/integration/gateway`; `@REQ-038` in
  `playwright/tests/requirements/req-037-038-residency.spec.ts`.
- **039 Prompt versioning — Gap.** No registry; system-prompt overrides have no history.
  Design-first item.
- **040 Prompt distribution — Partial.** Ed25519-signed manifest distribution (skills)
  can carry prompt content with central revocation today; it is not a versioned template
  object with parameters and rollback (that is 039's design).
- **041 Tool governance — Met, now enabled.** All four stages of the chain
  (`services/governance/config.yaml`) run synchronously pre-dispatch, first-deny-wins,
  audited. They had been deliberately disabled during evaluation.
- **042 MCP registry — Met.** Declarative per-server YAML with oauth scopes/audience,
  plan entitlement, org-suspension revocation, JTI token revocation.
- **043 Schema validation — Met with caveat.** Registration fails fast on missing/invalid
  manifests and schema sync; tool `input_schema` is captured but not meta-validated
  against JSON Schema (a well-formed-but-wrong schema passes). Add if CI-level
  malformed-schema tests become a hard requirement.

## Two-organization split (b.gulyaev)

`ai-kit` and `ai-sdlc-delivery` are registered as sibling organizations on the `standard`
plan (`services/access-control/plans.yaml`) — separate seats, budgets, analytics, and ACLs;
org-scoped rosters mean neither group's admin can see the other's users. Deliberately no
`email_domains`: both groups arrive from domains `astound-digital` already claims, so
membership is an explicit admin invite into the org (or a move via
`PUT /admin/api/management/users/{id}/organization`). A "project" dimension *below* an
organization is the open Hub question (REQ-006).

## Open items for Astound

1. ~~Define "Hub"~~ **Decided 2026-08-25: a Hub is a geographic grouping** (region/
   location), not a team or project. The dimension (analytics filter, budgets, rate-limit
   scope) is now implementable per `design-org-hierarchy-and-scm.md` — read "Hub" there as
   geographic; scheduled as the next follow-on pass. Team/project remain undefined.
2. **Confirm the 30-day analytics ceiling** (REQ-003) satisfies "historical usage trends".
3. **Decide on Bridge device-link gating** (REQ-002) — self-service for account holders, or
   admin approval per device.
4. **Nominate the authoritative SCM and identity mapping** (REQ-008).
5. **Accept the onboarding trade** (REQ-002) — invite-only means an admin step per new user.
6. **Confirm which build is deployed** — several rows the register calls Partial have been
   built since the environment Astound assessed.
7. **Sign off the enabled governance posture** (REQ-030/033/036/041) — the chain, safety
   scanners, and quota windows are now ON with first-pass category/size choices; review
   the block list and quota sizes against real traffic.
8. **Name the observability target stack** (REQ-028) — Datadog, Splunk, or LGTM decides
   which exporter is built first.
9. ~~Quality baseline / data-classification scheme~~ Decided: quality comparison ships on
   sensible defaults (cost/latency, no normalization) and is revisited when a quality
   signal exists (017/018); classification is the `governance:` booleans on model YAML,
   landed and enforced (037/038).
10. **Fresh sheet export** — the Google Sheet has been extended past REQ-043; rows beyond
    it need a re-export to be assessed.
