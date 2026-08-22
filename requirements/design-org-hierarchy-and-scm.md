# Design note — organizational hierarchy (REQ-006) and SCM commit ingest (REQ-008)

Two requirements in the enterprise register cannot be built until Astound makes a decision
that only Astound can make. This note states what is decided, what is not, what each option
costs, and what the work looks like once the decision lands. It is deliberately not an
implementation.

See [`compliance-register.md`](compliance-register.md) for the status of every row.

---

## Part 1 — REQ-006: a "Hub" / project dimension

### Where the platform stands

Analytics scopes on exactly three dimensions, declared once in `SiteScope`
(`extensions/web/admin/src/repositories/analytics/site/mod.rs`) and applied identically in
every query:

```
organization  ?org=          organizations.slug
department    ?department=   user_profile_ext.department  (free text, not a foreign key)
user          ?user_id=      ai_requests.user_id
```

Authorization resolves on a precedence ladder, deny-overrides, narrowest scope wins
(`extensions/web/admin/src/authz/`):

```
organization (300) → role (200) → department (100) → user (0)
```

A user belongs to **exactly one** organization: `organization_members` has
`PRIMARY KEY (user_id)`. Department is not a table relationship at all — it is a normalised
free-text column, `COALESCE(NULLIF(upe.department, ''), 'Default')`.

There is no team, group, project or Hub table, column, parameter or widget.

### The question Astound has to answer

"Hub" is used in the register without a definition, and the four plausible readings lead to
materially different systems. Before anything is built:

1. **Does a Hub nest inside an organization, or cut across them?**
   Inside → it is a peer of department and the change is contained. Across → membership stops
   being a tree, `organization_members`' single-org primary key no longer holds, and every
   query that assumes one organization per user has to change.
2. **Does a user belong to one Hub or many?**
   One is a column. Many is a join table, and "which Hub was this request for?" becomes a
   question the request itself must answer — which means capture, not just reporting.
3. **Does a Hub grant entitlements, or is it only a reporting label?**
   A reporting label is analytics-only. An entitlement grant makes it a new band in the
   resolver, and every existing precedence interaction has to be re-reasoned: a Hub that can
   *widen* what an organization granted would break the "a customer admin can never widen"
   property the whole ladder rests on.
4. **Is a project the same thing as a Hub, or a level below it?**
   The register lists them separately ("team/group, individual user, Hub, and potentially
   project"), which implies at least two new levels, not one.

**Recommendation: Hub as an org-nested, single-membership, reporting-only dimension**, sized
below. It is the reading that answers the register's actual acceptance criteria ("filter and
aggregate analytics by organizational dimension and drill down to user-level detail") without
touching authorization at all. Entitlements can be added later; they cannot easily be removed.

### What the recommended option costs

| Area | Change |
|---|---|
| Schema | `hubs (id, org_id, name, …)` with a unique `(org_id, name)`; `hub_id` on the membership row |
| Bootstrap | A `hubs:` block in `services/access-control/plans.yaml`, projected like organizations |
| Scope | A fourth field on `SiteScope`, plus its null-skip predicate in each site query |
| UI | One more `<select>` in `filters.rs`, one more chip in `urls.rs` |
| Repositories | The `site/*` queries already `LEFT JOIN organization_members`; the join is in place |
| Tests | A `@REQ-006` spec asserting the new dimension narrows the leaderboard |

Estimate: **2–3 days** including tests and the evidence screenshot. The cost is low precisely
because the join it needs already exists in every query.

### What the other readings cost

- **Cross-organization Hubs:** ~2 weeks. `organization_members`' primary key changes, every
  "the caller's organization" call site (`find_organization_for_user`, `listing_scope`,
  `may_administer`, `resolve_scope`) becomes "organizations", and the cross-tenant guards
  that were just tightened have to be re-derived against a many-to-many model.
- **Hub as an entitlement band:** add ~1 week on top of whichever membership model is chosen,
  most of it spent on precedence tests rather than on the band itself.
- **Project as a third level:** the same shape again, but with a harder capture problem —
  see below.

### The capture problem, which is the real constraint

Organization and department are properties of the **user**, so a request inherits them by
join. That is why the existing three dimensions cost nothing per request.

A project is a property of the **work**, not the person: the same developer works on several
in a day. Attributing a request to a project therefore requires the request to carry it, and
`ai_requests` has no such column — nor should it grow one speculatively. The plausible
sources, in order of fidelity:

- `user_commits.cwd` — already captured, already per-session, and maps cleanly to a checkout.
  A repo-to-project mapping table would make this work with **no new capture at all**.
- `plugin_usage_events` session metadata — same idea, wider coverage, more parsing.
- An explicit project id threaded from the bridge — highest fidelity, requires a client change.

If "project" turns out to mean "repository", the first option is nearly free and should be
the answer. That is worth checking before scoping anything larger.

### A related fact to record

`tenant_id` is **not** available as an analytics dimension and will not become one. Core
migration `003_drop_runtime_tenancy.sql` dropped it from `ai_requests`, `ai_gateway_policies`
and `ai_quota_buckets` because it was never populated with a real value; gateway policies are
global and quota is keyed per user. Any reporting model should key on **organization**. The
one surviving `tenant_id` column, on `tenant_activity`, is an unrelated web-activity table
that no analytics query reads.

### Knock-on: per-team and per-project budgets

REQ-009 asks for thresholds at "global versus user/team/project" level. Per-organization is
built and enforced; per-team and per-project are blocked on this same decision, because the
subject a budget is keyed to has to exist before it can be capped. The quota machinery itself
is already general — `ai_quota_buckets` carries `subject_kind`/`subject_id`, and core's
`quota::resolve_subject` matches a window's `subject` against any registered
`SubjectAttributeProvider` dimension, of which this repo registers three. A new dimension
therefore becomes budgetable as soon as it is registered; no quota work is implied.

---

## Part 2 — REQ-008: commits made outside Claude Code

### Where the platform stands

`user_commits` (`extensions/web/schema/17_usage_metrics.sql`) holds one row per commit
**observed through a Claude Code Bash tool call**: hash, message, branch, cwd, files changed,
insertions, deletions. It is written by `handlers::hooks_track::commits` from `PostToolUse`
stdout, deduplicated on a `(user_id, cwd, commit_hash)` unique index, and rolled into
`admin_usage_daily_rollups` hourly by the `usage_daily_rollup` job. The Code tab plots it
against AI line deltas over the same window and scope.

So commit activity **is** already correlated with AI usage for the same user and period —
which is what the register's acceptance criteria asks for. The gap is coverage: a commit made
in a terminal Claude Code never saw is invisible, which biases the series toward AI-assisted
work precisely when it is being used to judge AI-assisted work.

### The easy half

An ingest endpoint upserting into `user_commits` on the **existing** unique index. Because
`(user_id, cwd, commit_hash)` already deduplicates, hook-observed and SCM-observed records of
the same commit converge on one row instead of double-counting — the schema was already
shaped for this. Roughly a day for a GitHub webhook, plus a `source` column so the two
capture paths stay distinguishable in reporting.

### The hard half, which is the actual decision

**Identity.** The webhook knows a GitHub login and a commit-author email. `user_commits` needs
`users.id`. Nothing today maps between them, and each option has a failure mode worth naming:

| Option | Cost | Fails when |
|---|---|---|
| Match commit-author email to `users.email` | Trivial | Developers commit under personal or `noreply` addresses — common, and silent |
| Store an SCM login per user (profile field + admin UI) | ~2 days | Someone changes their handle; nobody notices until the series dips |
| OAuth-link the SCM account to the user | ~1 week | Nothing, but it needs an app registration and a user-facing flow |
| `cwd`-derived, as today | Free | Only covers what Claude Code saw — the current state |

**Recommendation: store an SCM login per user**, with unmatched commits recorded against a
null user rather than dropped. Unattributable volume then shows up as a visible number instead
of a quiet undercount, which matters because this metric will be read as an engineering
productivity signal.

### Prerequisites from Astound

1. Which SCM is authoritative — GitHub, Bitbucket, or both. The register names both.
2. Whether commits on all branches count, or only default-branch merges. This materially
   changes the numbers and should be decided before anyone reads a trend from them.
3. Whether bot and automation commits are excluded (they should be, and the existing
   `users.is_bot` flag is the natural predicate).
4. Consent posture: ingesting SCM activity for named individuals is employee monitoring, and
   the same commit-count caveat the register itself records — *"contextual, not a standalone
   productivity KPI"* — should be stated wherever the number is surfaced.

### Explicitly out of reach: REQ-007

No amount of SCM ingest produces a **tab-acceptance rate**. That needs an editor-level
accept/reject signal, and Claude Code emits none — the product surface says so verbatim in
`view_code.rs`. The nearest real measurements (permission-grant rate, AI lines vs committed
lines) are already shipped, labelled as proxies, and deliberately never subtracted from each
other. Closing REQ-007 means an IDE integration against a different data source, not more work
on this one.

---

## Summary of decisions needed

| # | Decision | Blocks | Recommended |
|---|---|---|---|
| 1 | What a Hub is (nesting, membership, entitlement) | REQ-006, per-team budgets | Org-nested, single, reporting-only |
| 2 | Whether "project" means "repository" | REQ-006 capture design | Check first — it may be nearly free |
| 3 | Authoritative SCM | REQ-008 | — |
| 4 | SCM identity mapping | REQ-008 | Stored SCM login per user |
| 5 | Branch policy for commit counting | REQ-008 credibility | Default branch only |
| 6 | Whether the 30-day analytics ceiling stands | REQ-003 sign-off | — |
