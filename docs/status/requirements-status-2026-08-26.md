# Requirements register — validated status report

Date: 2026-08-26 · Branch: `next` · Register: `[Enterprise AI] systemprompt.io requirements register (1).xlsx` (53 REQs)

Every status below was revalidated by running the evidence suites on this date against a freshly started local stack (`just start` + deterministic `just e2e-seed`):

- **Playwright requirements suite** (`just e2e`): **126 passed, 4 skipped, 0 failed** (36.7s). The 4 skips are the deliberate `test.fixme` placeholders for the roadmap rows REQ-019/032/035/039.
- **Gateway integration tier** (`just test-integration`, `req_0NN_*` Rust modules among the DB-backed suites): **685 passed, 0 failed** (10m22s).
- **Screenshot evidence pack** (`just e2e-screens`): regenerated into `storage/files/images/evidence/` with an `index.md` manifest recording the URL and principal behind every PNG.

Replication per requirement: `just e2e-req REQ-0NN` (or the register's `just e2e -- --grep @REQ-0NN`). Documentation: https://astound.systemprompt.io/documentation (enterprise-* pages, each ending in a "Verified evidence" table plus screenshots).

## Summary

| Group | Count | REQs |
|---|---|---|
| **Done** — delivered, tests pass, documented, screenshots attached | 27 | 001–005, 009–016, 020, 021, 023, 024, 027, 029, 031, 033, 034, 037, 038, 041, 042, 043 |
| **Partial** — built and passing, one piece pending | 4 | 025, 030, 036, 040 |
| **Blocked on an Astound decision** — substrate built and tested, completion needs a DEC answer | 8 | 006, 007, 008, 017, 018, 022, 026, 028 |
| **Blocked externally** — RAG/knowledge platform (MCP) and ADFS SSO | 10 | 044–052 (RAG/knowledge), 053 (ADFS) |
| **Not done** — scoped on the roadmap, not started | 4 | 019, 032, 035, 039 |

## Done (27)

All 27 register rows marked Delivered on 2026-08-25/26 revalidated green today. Each row: at least one passing tagged e2e test, a documentation page, and screenshot evidence.

| REQ | Requirement | e2e proof (all passing) | Docs page | Screenshot(s) |
|---|---|---|---|---|
| 001 | Admin user management | 6 tests: roster, search, session revoke, sign-out-everywhere, role-escalation refusals | enterprise-user-access | req-001-users-roster, req-001-user-detail |
| 002 | Controlled registration | 6 tests: self-register refused (in & out of domain), invite-only login, no anonymous bridge codes | enterprise-user-access | req-002-login-invite-only |
| 003 | Usage analytics dashboard | 3 tests: window selector, non-zero volumes for seeded org, empty-state not 500 | enterprise-analytics | req-003-overview, req-003-usage-trends |
| 004 | AI cost & model analytics | 2 tests: model mix matches seed, cost-per-request reported | enterprise-cost-management | req-004-spend, req-004-model-mix |
| 005 | User adoption analytics | 3 tests: leaderboard, configurable inactivity window, clamped ranges | enterprise-analytics | req-005-seats, req-005-inactive-seats |
| 009 | Spend limits & budget monitoring | 3 tests: spend vs plan cap, soft+hard thresholds named, non-admin refused | enterprise-cost-management | req-009-spend-meters |
| 010 | Real-time spend attribution | 3 tests: dept filter, model/provider slices, every row carries actor+model+cost | enterprise-cost-management | req-004-spend |
| 011 | Spend overrun early warning | 1 test: budget warning table names both threshold kinds | enterprise-cost-management | req-009-spend-meters |
| 012 | Soft budget alerts | 2 tests: soft threshold as distinct tick, crossings recorded non-blocking | enterprise-cost-management | req-009-spend-meters |
| 013 | Hard budget cutoffs | 1 test: seeded hard cap renders as meter ceiling | enterprise-cost-management | req-009-spend-meters |
| 014 | Burndown forecasting | 1 test: month burn-up rendered org-scoped | enterprise-cost-management | req-009-spend-meters |
| 015 | Self-service cost reporting | 3 tests: P&L CSV export, org-scoped customer report, export button | enterprise-cost-management | req-004-spend |
| 016 | Scheduled cost digests | 2 tests: `cost_digest` job registered, scheduler admin-only | enterprise-cost-management | — (scheduler surface) |
| 020 | Model provider abstraction | 1 test: pattern→provider indirection | enterprise-model-routing | req-037-gateway-routes |
| 021 | Central model access policy | 3 tests: rules centrally readable, non-admin read+write refused | enterprise-model-routing | req-037-gateway-routes |
| 023 | Enterprise SSO provisioning | 2 tests: passkey-only login door (Salesforce linked from profile), reconciliation job registered | enterprise-user-access | req-002-login-invite-only |
| 024 | Shadow AI detection & blocking | 2 tests: unlisted model refused, ungoverned /v1 call refused | enterprise-model-routing | req-037-gateway-routes |
| 027 | End-to-end traceability | 2 tests: one trace id resolves session/identity/spans; list reaches detail | enterprise-audit-observability | req-027-trace-chain |
| 029 | Latency SLOs | 2 tests: SLO threshold as query param reflected in split; picker options | enterprise-audit-observability | req-029-latency-slo |
| 031 | Usage anomaly detection | 1 test: anomalies section renders incl. empty state | enterprise-audit-observability | req-009-spend-meters (anomalies/warnings) |
| 033 | Per-consumer rate limiting | 1 test: every governed request lands a status row the quota guard reads | enterprise-safety-guardrails | req-033-requests-log |
| 034 | Private/self-hosted routing | 3 tests: no route exposes the backend provider, catalog scoping | enterprise-model-routing | req-037-gateway-routes |
| 037 | Data residency routing | 1 test: route carries its `requires:` block; boot+dispatch enforcement | enterprise-model-routing | req-037-gateway-routes |
| 038 | No-train/no-retain enforcement | 2 tests: `requires` survives admin round-trip; non-admin cannot rewrite | enterprise-model-routing | req-037-gateway-routes |
| 041 | Pre-execution tool governance | 1 test: denied tool call surfaces as DENY with policy named | enterprise-tool-governance | req-026-requests-audit |
| 042 | Governed MCP registry | 1 test: systemprompt server registered with admin scope visible | enterprise-tool-governance | req-042-mcp-catalog |
| 043 | Tool schema validation | 1 test: governed tool named alongside the decision | enterprise-tool-governance | req-026-requests-audit |

## Partial (4)

- **REQ-025 — Time-bound external access** (register: In progress). PAT expiry-at-issue and revocation both pass (`just e2e-req REQ-025`). What's missing for Done: an automated end-to-end expiry demonstration (token observed working, then observed refused after its expiry instant) rather than issuance-parameter proof. Docs: enterprise-user-access.
- **REQ-030 — Content safety guardrails** (register: Testing). Scanners are live with our default policy and the adversarial suite passes; a policy denial lands in the audit chain (`just e2e-req REQ-030`). Waiting on **DEC-003**: Astound must sign off the category list and block-vs-audit posture — until then enforcement reflects our judgement, not their policy. Docs: enterprise-safety-guardrails.
- **REQ-036 — PII/PHI detection & redaction** (register: Testing). Same position as REQ-030 — passing with defaults, awaiting DEC-003 sign-off (and a PHI in-scope/out-of-scope answer). Docs: enterprise-safety-guardrails.
- **REQ-040 — Central prompt template distribution** (register: In progress). Plugin and skills catalogs list centrally-managed bundles and both tests pass (`just e2e-req REQ-040`). Missing for Done: the update/revoke half of the publish/update/revoke lifecycle called out in the register's validation method. Docs: enterprise-tool-governance.

## Blocked on an Astound decision (8)

Each of these has its delivered substrate built, tested, and passing today; the remaining scope cannot be finished until the named decision (Decisions tab of the register, all "Awaiting Astound") is answered.

| REQ | Requirement | Blocked by | What already passes today |
|---|---|---|---|
| 006 | Org analytics drill-down | **DEC-001** (team/project data model) | Org/dept/user filters narrow views; cross-org URL access refused; drill-down is not an existence oracle (4 tests) |
| 007 | AI productivity metrics | **DEC-002** (identity/SCM source of truth) | Labelled proxy metrics render from seeded rollups with user filter (3 tests) |
| 008 | Commit activity analytics | **DEC-002** | Commit rollups plotted beside AI usage on a shared time scope (3 tests) |
| 017 | Provider cost comparison | **DEC-005** (quality baseline) | Cost-by-provider report with shares/totals + CSV agreeing with the page (2 tests) |
| 018 | Cost/quality/latency routing | **DEC-005** | Per-model latency+spend for one period; declarative, admin-editable routing seam (2 tests) |
| 022 | Governed virtual keys | **DEC-008** (per-key budget use cases) | PAT issue/revoke as a real state change; expiry at issue; every key inherits its owner's quotas (part of REQ-020–025 suite) |
| 026 | Immutable audit trail | **DEC-007** (compliance regime → hardening choice) | Full audit spine: actor, model, policy chain, cost, trace ids; unknown id dead-ends (3 tests) |
| 028 | Observability export | **DEC-006** (target stack: Datadog/Splunk/LGTM) | OTLP ingest + SSE audit stream; the audited spine exporters would read is complete (1 test) |

## Blocked externally (10)

### RAG / knowledge platform — REQ-044 to REQ-052

Blocked on the knowledge-platform MCP backend. What exists, per `docs/tickets/req-047-knowledge-bank-mcp.md`: the **knowledge-bank MCP server is scaffolded** (`extensions/mcp/knowledge-bank/`, port 5030, OAuth + admin scope, disabled by default) with the three-tool contract (`search` / `list_sources` / `index_stats`) pinned by unit tests, and it honestly refuses to fabricate results until a retrieval backend is configured. The production backend is the agreed migration of the Node "project-context" system (Bedrock + Titan v2 + Cohere rerank + LanceDB) — not yet ported.

- **REQ-044 (User awareness)** — largely supported already: session-context access control (project/role/permissions → MCP servers, models, sources) is enforced by the existing `access_control_rules` resolver; evidence written up in `docs/tickets/req-044-user-awareness-evidence.md`. Recommend re-statusing to "Supported with caveat (project dimension pending DEC-001)".
- **REQ-045 / REQ-051 (Conversation history)** — the audit spine already stores per-user prompts/responses/tools/timestamps (proven by REQ-026/027 tests); the user-facing/manager search interface is the missing piece and is scoped with the knowledge platform.
- **REQ-046–050, 052** — retrieval accuracy SLO, RAG pipeline, source integrations (Jira/Confluence/GitHub/Salesforce), knowledge graph, use-case library, retrieval ACLs: all depend on the unbuilt retrieval backend. No e2e tags exist for these yet by design.

### ADFS SSO — REQ-053

Blocked on Astound answers, per `docs/tickets/req-053-adfs-sso.md` (design + questions prepared): protocol choice matters because **no SAML/WS-Fed code exists anywhere** — the built path is OIDC (Salesforce SSO is live end-to-end on core primitives, and core has a generic `TrustedIssuer` seam ready). Also open: the AD-group → role/permission mapping rules. Implementation starts when the ticket's questions are answered.

## Not done — roadmap (4)

Register status Open, docs on enterprise-roadmap, `test.fixme` placeholders keep the tags greppable:

- **REQ-019** — Tenant-isolated semantic caching
- **REQ-032** — Automatic provider failover
- **REQ-035** — A/B model/provider testing
- **REQ-039** — Prompt versioning/rollback (note: REQ-040's catalog substrate is the natural base)

## Evidence changes made in this pass

- `just e2e-screens` extended: 5 new screenshots (req-018-models-latency, req-029-latency-slo, req-033-requests-log, req-040-plugin-catalog, req-040-skills-catalog) and the audit/trace/gateway/mcp captures restored; the script now writes straight to the tracked `storage/files/images/evidence/` (the old deleted `requirements/evidence/` path is gone).
- The four doc pages that said "No screenshots exist for them yet" (model-routing, audit-observability, safety-guardrails, tool-governance) now carry screenshot sections; the disclaimer is removed.
- Documentation screenshots now render as a gallery: thumbnails open a lightbox (native `<dialog>`) with prev/next cycling, keyboard navigation, and mobile-friendly sizing (`sp-evidence-lightbox` component).
- Stale references to the deleted `requirements/` pack fixed in `playwright/scripts/capture.ts` and the justfile.
- REQ-023's UI evidence updated for the passkey-only login change (commit 4a7763df retired the Salesforce sign-in door): the spec now asserts the passkey form is the only entry point and no SSO button remains, and the enterprise-user-access doc reflects that Salesforce is linked from the profile page.
