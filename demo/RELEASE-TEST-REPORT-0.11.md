# Demo Suite Validation Report — Release 0.11

Generated: 2026-05-21
Scope: All 42 category demos + preflight + seed (43 scripts total via sweep).
Method: sweep.sh run, per-log semantic validation, live cross-surface probes (HTTP + SQL).
Decision (per Ed): report only — no fixes attempted.

---

## 1. Executive summary

| Bucket | Count |
|---|---|
| Scripts run | 43 + preflight + seed = 45 |
| Sweep exit 0 | 43 / 43 |
| **Semantic regressions** | **3 P0, 2 P1, 4 P2** |
| Pre-existing issues (per `demo/issues.md`) unchanged | 4 |

Exit code 0 from every demo is misleading. Three substantive regressions are masked by the demo scripts continuing to print "✓" lines even when downstream behavior is broken:

- **P0-A** `/api/public/hooks/govern` returns correct decisions but **no longer persists per-rule rows to `governance_decisions`**. The last hook-endpoint write was 2026-05-18 — three days before this release. Every governance demo that displays "audit trail" is reading stale seed data.
- **P0-B** Preflight token extraction **silently falls back to the admin token**. The `data-copy="eyJ..."` attribute on `/admin/profile` is gone after the bootstrap UI rewrite. The "two-token" narrative (admin session vs plugin token) shown in preflight Step 4 is now fiction; both tokens are identical.
- **P0-C** `web validate` emits **14 warnings** — every template's content-type binding is unresolved, and the `services/web/assets` directory is missing. Demo continues with green "✓ Valid".

Plus a P0/P1 regression: `governance_decisions.policy` value was renamed `secret_injection` → `secret_scan`; demo scripts and `demo/issues.md` still reference the old name in narrative copy.

The sweep harness itself is fine — sweep.sh checks exit codes only, by design. It is the demo scripts' liberal use of `2>/dev/null`, `|| true`, and "decisions printed = decisions persisted" assumptions that hide these failures.

---

## 2. Per-script status matrix

Legend: ✅ pass · ⚠️ partial (exit 0 but semantic issue) · ❌ fail

| # | Script | Exit | Status | Notes |
|---|---|---|---|---|
| – | `00-preflight.sh` | 0 | ⚠️ | Plugin-token scrape fails; admin token written to `.token`. See §3.1. |
| – | `01-seed-data.sh` | 0 | ✅ | 27842 governance_decisions, 244 plugin_usage_events, 71 markdown_content, 188 user_activity, 25 user_contexts. |
| 01 | `agents/01-list-agents` | 0 | ✅ | Lists `developer_agent` + `associate_agent` from admin + core views. |
| 02 | `agents/02-agent-config` | 0 | ✅ | Validation + MCP tool access + status all populated. |
| 03 | `agents/03-agent-messaging` | 0 | ✅ | 6 steps completed, 3 AI calls via gemini-2.5-flash, total cost 1668 µ$. Live AI call successful. |
| 04 | `agents/04-agent-tracing` | 0 | ✅ | Trace tree non-empty. |
| 05 | `agents/05-agent-registry` | 0 | ⚠️ | One MCP connection cycle hit `HTTP 403 Forbidden: Host header is not allowed` before recovering. See §3.4. Issue #3 from issues.md (registry parse) still surfaces same shape. |
| 06 | `analytics/01-overview` | 0 | ✅ | conversations=22, requests=6 (24h), tokens=4999. |
| 07 | `analytics/02-agent-analytics` | 0 | ✅ | Numbers present; `failed_tasks: 0` (false-positive grep). |
| 08 | `analytics/03-cost-analytics` | 0 | ✅ | Cost rollups present. |
| 09 | `analytics/04-request-analytics` | 0 | ✅ | Request stats populated. |
| 10 | `analytics/05-session-analytics` | 0 | ✅ | Sessions enumerated. |
| 11 | `analytics/06-content-traffic` | 0 | ✅ | Engagement events populated by seed. |
| 12 | `analytics/07-conversations` | 0 | ✅ | Conversation rollups present. |
| 13 | `analytics/08-tool-analytics` | 0 | ⚠️ | Shows `failed: 2` of 31 tool executions historically (likely from §3.4 connection failures). |
| 14 | `cloud/01-cloud-overview` | 0 | ⚠️ | Cloud creds **expired** (token expired warning on every CLI call). Demo continues. Cosmetic until cloud commands needed. |
| 15 | `governance/01-happy-path` | 0 | ⚠️ | HTTP response correct; **post-call DB rows for this session = 0** under 0.11. Demo reads earlier rows from seed. See §3.2. |
| 16 | `governance/02-refused-path` | 0 | ⚠️ | Same as #15. |
| 17 | `governance/03-audit-trail` | 0 | ⚠️ | Counts queried are from seed (27k rows). The demo's claim "every decision audited" is no longer true post-release. |
| 18 | `governance/04-governance-happy` | 0 | ⚠️ | HTTP correct; audit row not persisted. |
| 19 | `governance/05-governance-denied` | 0 | ⚠️ | HTTP correct; audit row not persisted. |
| 20 | `governance/06-secret-breach` | 0 | ⚠️ | HTTP correctly denies on all 3 secrets; **no DB row written** for the re-run session. Audit log shows 36 historical rows including 12 `auth_failure` rows from earlier broken-token runs and a renamed-policy split (3 `secret_injection` + 15 `secret_scan`). See §3.3. |
| 21 | `governance/07-rate-limiting` | 0 | ✅ | Limiter config printed. |
| 22 | `governance/08-hooks` | 0 | ✅ | Lists `example_hook` PreToolUse. |
| 23 | `infrastructure/01-services` | 0 | ✅ | 3/3 services running. |
| 24 | `infrastructure/02-database` | 0 | ✅ | Tables/indexes enumerated. |
| 25 | `infrastructure/03-jobs` | 0 | ✅ | Job history present; `publish_pipeline` succeeded (succeeded=1, failed=0). |
| 26 | `infrastructure/04-logs` | 0 | ✅ | Logs view returns rows; `Errors: 0` reported. |
| 27 | `infrastructure/05-config` | 0 | ⚠️ | Heavy "Token expired" noise (cloud creds), but all `includes:` resolve. |
| 28 | `mcp/01-mcp-servers` | 0 | ✅ | 1 MCP server, 1 tool (intentional consolidation in 0.11 — see §4). |
| 29 | `mcp/02-mcp-access-tracking` | 0 | ✅ | 5 OAuth + 5 access rows. |
| 30 | `mcp/03-mcp-tool-execution` | 0 | ⚠️ | Same `failed: 2` historical anomaly as #13. |
| 31 | `performance/01-request-tracing` | 0 | ⚠️ | **Two queries return `row_count: 0`** — `governance_decisions` for the live session is empty even after the demo fires a govern hook. See §3.2. |
| 32 | `performance/02-load-test` | 0 | ⚠️ | 2000-request benchmark; demo proudly states "Every one of those 2,000 requests: … audit written". The final SQL probe returns `row_count: 0`. See §3.2. |
| 33 | `skills/01-skill-lifecycle` | 0 | ✅ | 2 skills synced from YAML. |
| 34 | `skills/02-content-management` | 0 | ✅ | 71 markdown rows. |
| 35 | `skills/03-file-management` | 0 | ✅ | Files enumerated. |
| 36 | `skills/04-plugin-management` | 0 | ✅ | `enterprise-demo` plugin shown with 2 skills + 2 agents + 1 MCP server. Hook validation: 1/1 valid. |
| 37 | `skills/05-contexts` | 0 | ✅ | Full CRUD round-trip on `core contexts` succeeded. |
| 38 | `users/01-user-crud` | 0 | ⚠️ | Misleading name — script is **read-only** (list/count/stats/search), no create/update/delete despite "CRUD" title. |
| 39 | `users/02-role-management` | 0 | ✅ | Read-only role inspection. |
| 40 | `users/03-session-management` | 0 | ✅ | Current session + profiles printed. |
| 41 | `users/04-ip-ban` | 0 | ✅ | Add 192.168.99.99, verify, remove, verify — clean. |
| 42 | `web/01-web-config` | 0 | ✅ | Content types + templates listed. |
| 43 | `web/02-sitemap-validate` | 0 | ⚠️ | "✓ Valid" but **14 warnings**: missing assets dir + 13 unresolved template→content-type bindings. See §3.5. |

Summary: 30 ✅ · 13 ⚠️ · 0 ❌.

---

## 3. Per-failure deep-dive

### 3.1 — P0-A · Preflight plugin-token scrape silently broken

**Symptom**: preflight log line 55:
```
WARNING: Could not extract plugin token from dashboard.
Falling back to admin token (works for all demo endpoints).
```

**Live evidence**:
```
$ curl -s -b "access_token=$ADMIN_TOKEN" http://localhost:8080/admin/profile | grep -c 'data-copy'
0
```

**Root cause**: `storage/files/admin/partials/layout.hbs` + `js_services.rs` + the new bootstrap.js were rewritten to render the install panel as a plain hint:
```html
<div class="install-tab-content" data-install-panel="claude-code" ...>
  <p class="install-tab-hint">Use the CLI to install marketplaces and plugins.
   Run <code>systemprompt --help</code> to discover commands.</p>
</div>
```
The `data-copy="eyJ..."` attribute the preflight regex targets no longer exists. Preflight falls back to the admin session JWT, then prints the same "two tokens, same user" comparison table — but both tokens are now identical, so Step 4's pedagogical content is fiction.

**Knock-on effect**: every downstream demo runs with `scope=admin`, not `scope=service`. For most demos this is silently equivalent. For analytics dashboards that filter on `session_id=plugin_cowork-bundle`, results are now empty.

**Confidence**: high. Reproduced live.

**Touched by release**: `extensions/web/site/src/assets/js_services.rs`, `storage/files/admin/partials/layout.hbs`, `storage/files/js/services/bootstrap.js`.

---

### 3.2 — P0-B · `/api/public/hooks/govern` no longer persists per-rule decisions

**Symptom**: all governance-flavour demos (#15-20, #31, #32) display `row_count: 0` when querying `governance_decisions WHERE session_id = '<this run's session>'`. HTTP responses remain correct.

**Live evidence**:
```
$ SID="probe-$(date +%s)"
$ curl -s -X POST "$BASE/api/public/hooks/govern?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
    -d "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Read\",\
         \"agent_id\":\"developer_agent\",\"session_id\":\"$SID\",\
         \"cwd\":\"/tmp\",\"tool_input\":{\"file_path\":\"/src/main.rs\"}}"
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow"}}

$ systemprompt infra db query \
  "SELECT * FROM governance_decisions WHERE session_id = '$SID'"
{ "rows": [], "row_count": 0, ... }
```

Same with secret-breach payloads: response correctly denies with detector reason; **no row written**.

Last successful hook-endpoint write was 2026-05-18 (3 days before this release):
```
last session_id in governance_decisions written via hooks:
  loadtest-1779093920-govern  2026-05-18T08:45:21
```

After that date, the table only receives:
- `policy=authz` rows from MCP tool gate-keeping (`tool_name=systemprompt`, UUID session_ids, fired by the MCP server's internal scope check), AND
- direct INSERTs from `01-seed-data.sh`.

**Implication**: the demo narrative "every decision audited to Postgres" (per CLAUDE.md and load-test #32) is false for hook-endpoint traffic. The audit pipeline for `/hooks/govern` is broken.

**Touched by release**:
- `extensions/web/schema/11_audit_event_notify.sql` — added 3 new audit-notify triggers (`audit_event_notify_governance`, `audit_event_notify_ai_requests`, `audit_event_notify_plugin_usage`). Triggers exist and fire on INSERT, but if the INSERT itself stops happening upstream, the triggers are dormant.
- New columns on `governance_decisions`: `actor_kind`, `actor_id`, `act_chain`. If the audit writer in 0.11 core requires these to be non-null and the hook path doesn't populate them, the INSERT silently fails (the writer is `tokio::spawn`-backed; errors get logged but never surface to the HTTP response).

**Confidence**: high — reproduced live, multiple sessions, multiple payloads.

**Recommended triage**: grep core 0.11 for `INSERT INTO governance_decisions` and confirm whether the `actor_kind`/`actor_id`/`act_chain` columns are required by the new repo method but absent from the hook handler's context-build. The fact that MCP-internal calls succeed but hook calls fail points squarely at differential context construction.

---

### 3.3 — P1 · `governance_decisions.policy` renamed `secret_injection` → `secret_scan`

**Symptom**: the audit table query in `governance/06-secret-breach.sh` returns mixed policy values:

```
auth_failure       12   (historical, from broken-token runs)
secret_injection   3    (legacy name, oldest 3 detections)
secret_scan       15    (current name)
default_allow      6
```

**Implication**:
- `demo/issues.md` and all demo prose still reference `secret_injection`.
- Any documentation, dashboard query, or alert rule pinned to `secret_injection` is now silently dead.
- For the same physical event ("AWS key detected") two rows can exist under two different policy names, breaking historical aggregations.

**Confidence**: high.

**Touched by release**: core 0.11 rule engine.

---

### 3.4 — P1 · MCP `Host header is not allowed` 403 on initial connection

**Symptom**: `agents-05-agent-registry.log` contains:
```
ERROR rmcp::transport::worker: worker quit with fatal: Transport channel closed,
  when UnexpectedServerResponse("HTTP 403 Forbidden: Forbidden: Host header is not allowed")
WARN  systemprompt_mcp: MCP server connection validation failed
  server="systemprompt" error="MCP client initialize: …"
```

Followed ~17ms later by a successful `Service initialized as client` log line — the client retries and succeeds. So the demo passes, but the first connection attempt is consistently rejected.

**Implication**: this is the most plausible source of the `failed: 2` historical tool-execution counts surfaced by `analytics/08` and `mcp/03`.

**Confidence**: medium. The retry masks the failure end-to-end; the underlying host-validation policy is likely a new 0.11 default.

**Touched by release**: `extensions/mcp/systemprompt/src/server.rs` plus core's HTTP host filter.

---

### 3.5 — P1 · `web validate` emits 14 unresolved-binding warnings

```
Web Configuration Validation
✓ Valid — 4 items checked, no errors (14 warning(s))
  - [config]    Assets directory not found: services/web/assets
  - [templates] Template 'feature'      → unknown content type 'feature'
  - [templates] Template 'legal-post'   → unknown content type 'legal'
  - [templates] Template 'legal-post'   → unknown content type 'page'
  - [templates] Template 'blog-post'    → unknown content type 'blog'
  - [templates] Template 'docs-page'    → unknown content type 'docs' (× 7 variants)
  - [templates] Template 'homepage'     → unknown content type 'homepage'
  - [templates] Template 'blog-list'    → unknown content type 'blog-list'
  - [templates] Template 'feature-page' → unknown content type 'feature-page'
```

The validator now distinguishes "errors" from "warnings" and prints ✓ if no errors — but template→content-type wiring is fundamentally broken in this release for the marketing site templates. Site renders may fall back to defaults or 404.

**Confidence**: high.

**Touched by release**: `extensions/web/` (template registration) and/or new content-type registration in core 0.11.

---

### 3.6 — P2 · `01-user-crud.sh` is not CRUD

```
1. Lists all users
2. Shows user count
3. Displays user statistics
4. Searches users by keyword
```

No create/update/delete. Documentation rot, not a release regression. Either rename to `01-user-listing.sh` or add the C/U/D operations.

---

### 3.7 — P2 · `demo/.token` overwritten by a non-preflight writer

During this validation pass, between `00-preflight.sh` (which wrote a `iss=systemprompt-local` token for `ed@`) and `sweep.sh`, the file was rewritten to a `iss=systemprompt-airgap` token for `airgap-admin@demo.systemprompt.io`. The only other writer in the demo tree is `scenarios/airgap/03-governance.sh`, which sweep does NOT iterate (sweep is `-maxdepth 2`).

Either (a) a previous airgap run left state on the host that the build/start cycle restores, or (b) something in the harness re-runs the airgap flow as a side effect. Either way: the entire sweep ran against an `airgap-admin` JWT for a user that does not exist in the local DB, producing 1739 `auth_failure` rows historically and biasing the seed-displayed counts.

**Confidence**: medium — the artifact is reproducible (cat the file), but the writer is unidentified.

---

### 3.8 — P2 · Cloud credentials expired

Every CLI invocation in every log prints:
```
WARN systemprompt_cli::bootstrap: Cloud credentials unavailable;
  continuing in local-only mode. Cloud commands will require
  'systemprompt cloud login'. error=Token expired.
```

29 of 43 demo logs contain this warning. It does not fail demos because local mode is sufficient. But it doubles the size of every log and overwhelms real warnings.

`.systemprompt/credentials.json` `api_token` `exp=1778498455` = 2026-05-11. Expired ~10 days ago.

---

### 3.9 — P2 · Workspace version skew

`systemprompt --version` → `0.11.0` (the core submodule)
Workspace `Cargo.toml` → `version = "0.9.2"`
MCP server reports `server_info: { version: "0.9.2", … }` over the wire.

Likely intentional (template vs core versioning) but worth confirming.

---

## 4. Cross-surface inconsistencies

| Surface | What it says | Reality |
|---|---|---|
| `/api/public/hooks/govern` HTTP response | `permissionDecision: allow / deny` (correct) | – |
| `governance_decisions` table | Empty for the live session_id | Mismatch — §3.2 |
| `audit_event_notify_governance` PG trigger | Exists and would fire | Dormant — no INSERTs to fire on |
| `/admin/profile` HTML | No `data-copy` attribute | Preflight regex expects one — §3.1 |
| `demo/issues.md` baseline | "All 42 category demos pass" | Sweep still green, **semantics regressed** — §3.2 §3.5 |

---

## 5. Masked-failure inventory

Scripts that exit 0 despite the underlying contract being broken:

- `governance/01..06`, `performance/01..02` — see §3.2.
- `web/02-sitemap-validate` — see §3.5.
- `00-preflight.sh` — see §3.1; the WARNING line is there but the script returns 0 and writes a degraded token.
- `users/01-user-crud` — see §3.6.

---

## 6. Friction inventory

- ~500ms cloud-cred validation per CLI call (already documented in `demo/issues.md` #5).
- "Token expired" warnings on **every** CLI call (§3.8) — should be silenced once per profile, or only emitted when a cloud command is invoked.
- `[profile: local …]` prefix appears on most outputs (already documented in `demo/issues.md` #6) — sweep took 29.9s, cloud probing dominates.
- `RUST_LOG=warn` still leaks DEBUG-level lines on some commands.

---

## 7. Delta vs `demo/issues.md` baseline

| Issue | Status |
|---|---|
| #1 `plugins mcp list` AppPaths uninit | **Still present** — same shape |
| #2 `plugins mcp logs <server>` empty | **Still present** — same shape |
| #3 `admin agents registry` JSON parse | **Still present** — graceful fallback in demo unchanged |
| #4 `plugins mcp validate` requires `--service` | **Still present** |
| #5 ~500ms startup latency | Still present, possibly worse now |
| #6 `[profile: local …]` prefix noise | Still present |

New issues introduced (or first surfaced) in 0.11:

| New | Severity | Where |
|---|---|---|
| Hook endpoint stops auditing governance decisions | **P0** | §3.2 |
| `/admin/profile` no longer renders plugin-token install widget | **P0** | §3.1 |
| `web validate` 14 unresolved template bindings + missing assets dir | **P0** | §3.5 |
| Policy name renamed `secret_injection` → `secret_scan` | P1 | §3.3 |
| MCP server first-connect `Host header is not allowed` 403 | P1 | §3.4 |
| `01-user-crud.sh` is read-only despite name | P2 | §3.6 |
| `demo/.token` mysteriously becomes the airgap token | P2 | §3.7 |
| Cloud creds expired → log spam on every CLI call | P2 | §3.8 |

---

## 8. Prioritised follow-up list (no fixes attempted)

### P0 — fix before next release

1. **Restore hook-endpoint audit writes**. Compare 0.10's `record_decision()` call sites against 0.11. The new `actor_kind`/`actor_id`/`act_chain` columns are the prime suspects — confirm the hook handler is populating them. Backstop: add a sentry alert that fails CI if `governance_decisions` row count doesn't increase after a `/hooks/govern` call in integration tests.
2. **Decide what `/admin/profile` should now offer for `SYSTEMPROMPT_TOKEN`**. Options: (a) restore the install-widget JWT, (b) replace preflight's scrape with a CLI command (`systemprompt admin session token --plugin`), (c) drop the two-token pedagogical narrative and ship one admin token. Recommendation: (b) — preflight should not be parsing HTML.
3. **Fix `web validate` template bindings** or change the validator's classification so an unbound template is an error, not a warning. Right now it's the worst combination: 14 broken bindings displayed as a green "✓".

### P1 — fix before customers run demos

4. Rename `secret_injection` → `secret_scan` everywhere in demo prose and `demo/issues.md`. Add a one-line migration note in `demo/RELEASE-NOTES.md`.
5. Investigate MCP `Host header is not allowed` 403 on initial connection. Likely a new 0.11 hostname allow-list missing `localhost` or the resolved tenant host. If it's intentional (defense in depth), document the retry behavior.

### P2 — cleanup

6. Rename `users/01-user-crud.sh` → `users/01-user-listing.sh` (or implement actual CRUD).
7. Identify what's overwriting `demo/.token`. Worst case it's the host's shell history re-running an airgap recipe; either way, harden preflight to always overwrite-on-run (it does today) and re-read after.
8. Either renew `.systemprompt/credentials.json` or make "expired token" a one-shot bootstrap warning instead of per-call.
9. Audit MCP server version reporting — `0.9.2` from a 0.11 CLI is confusing in logs.

---

## 9. Artifacts on disk

| Path | Contents |
|---|---|
| `/tmp/demo-sweep/*.log` | 43 sweep logs, one per script |
| `/tmp/demo-deep/00-preflight.log` | Standalone preflight run |
| `/tmp/demo-deep/00-preflight.err` | Empty (no stderr) |
| `/tmp/demo-deep/01-seed-data.log` | Standalone seed run |
| `/tmp/demo-deep/preflight-rerun.log` | Preflight re-run with clean `.token` |
| `/tmp/demo-deep/gov-06-rerun.log` | Governance-06 re-run, confirms HTTP correct but DB writes still 0 |
| `/tmp/demo-deep/perf-01-rerun.log` | Perf-01 re-run, confirms DB writes still 0 |
| `/tmp/demo-deep/admin-profile.html` | The post-0.11 `/admin/profile` body — no `data-copy` |
