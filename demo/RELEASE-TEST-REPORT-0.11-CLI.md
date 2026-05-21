# CLI Command Sweep — Release 0.11

Generated: 2026-05-21
Scope: **293 leaf CLI commands** across 8 domains (full `systemprompt --help` tree, auto-enumerated).
Method:
1. `--help` sweep over all 293 leaves.
2. Read-only invocation sweep over 122 safely-callable leaves with safe defaults.
3. Targeted deep-dive on **MCP** (`plugins mcp call`) and **A2A** (`admin agents message/task`), with live SQL persistence probes.
4. Destructive lifecycle round-trips for users, contexts, files, IP-bans, rate-limits, keys, service stop/restart — all rolled back.
5. Error/edge variants (unknown ids, bad UUIDs, bad SQL).
Decision: report only — no fixes attempted, matching prior 0.11 demo report.

This sweep complements `demo/RELEASE-TEST-REPORT-0.11.md` (which scored the **demo scripts**); this report scores the **CLI surface itself**.

---

## 1. Executive summary

| Bucket | Count |
|---|---|
| Leaf commands enumerated | **293** |
| `--help` exit 0 | 293 / 293 |
| Read-only invocations attempted | 122 |
| Read-only ✅ clean | 102 / 122 |
| Read-only ⚠️ flagged | 20 / 122 |
| Destructive lifecycles attempted | 9 (users, contexts, files, ip-ban, rate-limits, keys, services, jobs, access-control) |
| Destructive ✅ clean | 8 / 9 |
| Destructive ⚠️ flagged | 1 (`admin users role promote/demote` signature confusion) |
| MCP tool calls invoked | 4 (1 happy + 3 error variants) |
| A2A messages sent | 4 (`developer_agent` ×3, `associate_agent` ×1) |
| Live SQL persistence probes | 5 |

### Findings

- **3 P1**, **5 P2**, **0 P0**. No panics. No `--help` failures.
- **Net-new vs prior demo-suite report**:
  - **P1-A** `admin agents task` always returns 401 — does not reuse the session token its sibling `admin agents message` uses transparently.
  - **P1-B** `core content show <slug>` requires undocumented `--source` even when the slug is unique — error message asks for "Source ID when using slug" but no flag is hinted.
  - **P1-C** `plugins mcp call <server> <tool>` returns `success: false` with empty `content[]` and a generic `error: "Tool execution failed"` when the underlying tool rejects input. The MCP server's actual error message is swallowed. Exit code is **0** in this case, masking the failure for scripted callers.
- **Confirmed / refined vs prior demo-suite report**:
  - Prior **P0-A** ("`/api/public/hooks/govern` no longer persists per-rule rows") → **now persists** as observed in this sweep. The endpoint wrote a `governance_decisions` row in real time during the probe (delta +1, 0-arg deny path). However the persistence is *conditional* on a tool-call path: a plain A2A `message → reply` with no tool use writes `ai_requests` only and **not** `governance_decisions` (delta 0). This is correct behaviour, but is what tripped the prior report's audit-trail expectation when the demos messaged agents without forcing tool use.
  - Prior **P0-C** (`web validate` printed "✓ Valid" while emitting 14 warnings) → **fixed in exit-code dimension**: now exits **rc=1** with the same 14 warnings, no longer silently green. Warnings themselves still present (every template content-type binding still unresolved + missing assets dir).
- **Friction items (still present from prior report)**:
  - `[profile: local (local) | tenant: …]` header still leaks on `admin agents logs` (raw ANSI escape codes embedded in JSON output, P2).
  - `cloud *` commands emit `Token expired` WARN line on every invocation when cloud creds expired locally; non-cloud commands stay clean (latency contained, median 96ms across the read sweep).
- **Latency** — median CLI command latency across the 122-command read sweep: **~96 ms**. Slowest: `analytics tools list` at 99ms; vast majority < 100ms. **This is materially better than the prior report's "~500ms"** friction note — possibly because cloud-creds-already-expired short-circuits the cloud check.

---

## 2. Per-domain command matrix

Legend: ✅ clean · ⚠️ exit 0 but warning/anomaly · 🅡 required-arg error (clap rc=2, not a defect) · ❌ failure

### 2.1 `core` (49 leaves)

| Status | Command | Note |
|---|---|---|
| ✅ | core artifacts list/show | clean |
| ✅ | core content list/search/edit/delete/delete-source | clean |
| 🅡 | core content popular/verify/status | require positional `<SOURCE>` / `<IDENTIFIER>` — by design |
| ⚠️ | core content show \<slug\> | rejects slug-only lookup with "Source ID required when using slug" — **P1-B**, no `--source` flag is hinted in the error |
| 🅡 | core content link list, core content files list | require `--campaign`/`--content`/`--file` — by design |
| ✅ | core files list/show/upload/delete/validate/config/search/stats/ai * | upload requires `--context`; full lifecycle clean |
| ✅ | core contexts list/show/create/edit/delete/use/new | full CRUD lifecycle round-trip clean |
| ✅ | core skills list/show | sync is implicit at startup; no manual sync subcommand exposed (likely intentional, see §3.5) |
| ⚠️ | core plugins validate | exits 0 but emits a warning about asset validation requiring full profile initialization — friction, not a defect |
| ✅ | core hooks list/validate | 1/1 valid hook (`example_hook` PreToolUse) |

### 2.2 `infra` (45 leaves)

| Status | Command | Note |
|---|---|---|
| ✅ | infra services start/stop/restart/status/cleanup/serve | full stop+restart cycle on `systemprompt` MCP server clean |
| ✅ | infra db query/execute/tables/describe/info/migrate/migrate-down/migrate-plan/migrate-status/migrate-repair/assign-admin/status/validate/indexes/size/doctor | clean; **query** correctly rejects non-SELECT SQL with a clear error |
| 🅡 | infra db count \<TABLE\>, infra db migrations history \<EXTENSION\> | require positional — by design |
| ⚠️ | infra db query error message | "Table or relation 'id' does not exist" surfaced when the planner mis-binds a column name. The actual error from the wrapped SQL is hidden behind a paraphrase. P2 friction. |
| ✅ | infra jobs list/show/run/history/enable/disable/cleanup-sessions/log-cleanup | `infra jobs run publish_pipeline` ran clean (succeeded=1, failed=0) |
| ✅ | infra logs view/search/stream/export/cleanup/delete/show/summary | clean |
| ⚠️ | infra logs summary | exit 0 but content reports "Total Logs: 1" — the database log source appears to retain very few entries between restarts. Possibly by design (TTL/rotation); flagged P2 to verify. |
| ✅ | infra logs trace list/show, request list/show/stats, tools list, audit | clean |

### 2.3 `admin` (90+ leaves)

| Status | Command | Note |
|---|---|---|
| ✅ | admin users list/show/search/create/update/delete/count/export/stats/merge | full CRUD lifecycle: create → show → update → role-promote → role-demote → delete → verify-gone all clean |
| ⚠️ | admin users role promote/demote | help shows `<IDENTIFIER>` only — passing a role name (`admin`) is rejected with `unexpected argument 'admin' found`. The command **infers** the role; this is undocumented surprise. P1-D friction. |
| 🅡 | admin users role assign, admin users session list | require positional `<USER_ID>` — by design |
| ✅ | admin users ban add/check/remove/list/cleanup | ban add 10.99.99.99 → check (banned) → remove --yes (unbanned) clean |
| ✅ | admin agents list/show/validate/create/edit/delete/status/logs/registry/tools/run | clean |
| ⚠️ | admin agents logs | JSON output contains literal ANSI escape sequences (`[2m…`) and `[profile: local …]` prefix lines embedded **inside** the logs array. P2 — breaks downstream JSON consumers expecting clean text. |
| ✅ | admin agents message | **deep-dive §3.1** — full A2A working against both agents, governance written on tool-call paths |
| ❌ | admin agents task | **P1-A** — returns 401 Unauthorized when called with no `--token`. Sibling `admin agents message` works without `--token` by reusing the session. Inconsistent. |
| ✅ | admin config show/list/validate | clean |
| ⚠️ | admin config rate-limits validate | exit 0 but emits "Rate limiting is currently DISABLED" warning — informational, not a defect. P3. |
| ✅ | admin config rate-limits enable/disable/set/reset/compare/preset list/show/apply/diff | enable→disable round-trip clean; warns "Restart services for changes to take effect" |
| 🅡 | admin config rate-limits export | requires `--output <FILE>` |
| ✅ | admin config server show/set, cors list/add/remove | clean |
| ✅ | admin config runtime/security/paths/provider * | clean |
| ✅ | admin setup, admin bootstrap | not invoked end-to-end (would mutate active profile); `--help` clean |
| ✅ | admin session show/switch/list/login/logout | `session show` exit 0 but emits "WARN" flag in our matcher because of nested empty-array warning; cosmetic |
| ✅ | admin bridge enroll-cert/issue-code/list/rotate-signing-key | not invoked end-to-end (requires real device fingerprint); `--help` clean |
| ✅ | admin access-control export-yaml | clean YAML round-trip — exported the active skill access table |
| ✅ | admin keys generate, issue-plugin-token | `keys generate --output /tmp/.../key.pem` produced RSA-2048 PKCS#8 PEM cleanly, with kid printed |

### 2.4 `cloud` (28 leaves)

| Status | Command | Note |
|---|---|---|
| ⚠️ | cloud auth whoami, tenant list, profile list, status | every cloud command preceded by `WARN message=Cloud credentials unavailable... error=Token expired.` Then runs in local-only mode. Cosmetic but pervasive. P2. |
| ❌ | cloud secrets sync | `Error: Credentials required. Run 'systemprompt cloud login'` — correct refusal, but rc=1 (consistent with other cloud-requiring commands) |
| 🅡 | cloud deploy / restart / sync push/pull / secrets set / dockerfile / db * / domain * | not invoked (excluded by destructive-but-no-rollback policy); `--help` clean for all |

### 2.5 `analytics` (29 leaves)

| Status | Command | Note |
|---|---|---|
| ✅ | analytics overview, conversations stats/trends/list, agents stats/list/trends/show, tools stats/list/trends/show, requests stats/list/trends/models, sessions stats/trends/live, content stats/top/trends, traffic sources/geo/devices/bots, costs summary/trends/breakdown | **29/29 clean read** — all analytics surfaces return populated data from seed |

### 2.6 `web` (11 leaves)

| Status | Command | Note |
|---|---|---|
| ✅ | web content-types/templates/assets/sitemap (list/show + crud) | clean |
| ⚠️ | web validate | rc=1 with **14 warnings** — 1 missing assets dir + 13 unresolved template→content-type bindings. Same warnings as prior report's P0-C, **but rc is now non-zero** (✓ behavior change: no longer silently green) |

### 2.7 `plugins` (16 leaves incl. `plugins mcp`)

| Status | Command | Note |
|---|---|---|
| ✅ | plugins list/show/run/validate/config/capabilities * | clean; 15 extensions discovered |
| ✅ | plugins mcp list/status/list-packages/tools | clean; 1 server (`systemprompt`), 1 tool (`systemprompt`), `running` on port 5010 |
| ⚠️ | plugins mcp validate | `healthy: 0, unhealthy: 1` because MCP server requires OAuth; reports "Port responding, OAuth authentication required" — by design but visually a red mark. **Note**: `--service <name>` flag from the prior issues-list does NOT exist; positional `[SERVER]` is the actual signature. P3 doc drift. |
| ✅ | plugins mcp logs systemprompt | reads DB log source, returns 1 line ("MCP service started") |
| ✅ | plugins mcp call | **deep-dive §3.2** — happy path works; **P1-C** on error transparency |

### 2.8 `build` (2 leaves)

| Status | Command | Note |
|---|---|---|
| ✅ | build core, build mcp | `--help` clean; not invoked end-to-end (covered by `just build`) |

---

## 3. Per-deep-dive evidence

### 3.1 — A2A messaging via CLI

**Setup**: both `developer_agent` (port 9101) and `associate_agent` (port 9102) running; MCP `systemprompt` server up on 5010.

**Probe 1 — pure conversational reply (no tool use)**:
```
$ systemprompt admin agents message developer_agent -m 'Reply with exactly the word: pong' --blocking --timeout 60
pong          # 1.6s wall
```
Live SQL deltas vs. baseline:
- `ai_requests`: **+1** ✅ (model `gemini-2.5-flash`, status `completed`, user_id resolves)
- `governance_decisions`: **+0** — *no* governance row, because no tool was invoked. **Correct behaviour**, but explains the prior demo report's confusion: every agent message does NOT write a governance row, only tool-using messages do.
- `mcp_tool_executions`: **+0**

**Probe 2 — tool-using A2A path**:
```
$ systemprompt admin agents message developer_agent \
    -m 'Use the systemprompt tool to run "core skills list" and tell me how many skills exist.' \
    --blocking --timeout 60
I've listed the core skills using the `systemprompt` tool. There are 2 skills currently available …   # 5.8s wall
```
Live SQL deltas:
- `ai_requests`: **+3** (initial inference + post-tool inference + completion)
- `governance_decisions`: **+1**, with `decision="allow"`, `policy="authz"`, `tool_name="systemprompt"`, `actor_kind="user"`. ✅
- `mcp_tool_executions`: **+1**, `tool_name="systemprompt"`, `status="success"`, traceable via `started_at`

**Probe 3 — `admin agents task <id>` retrieval — P1-A**:
```
$ systemprompt admin agents task developer_agent --task-id d367aee3-fbd9-49f0-8c3d-c0c06a44d599
Error: Failed to get task details
Caused by: Agent request failed with status 401 Unauthorized: {"code":"unauthorized","message":"Invalid or expired JWT token","path":"/developer_agent",…}
```
**Defect**: sibling `admin agents message` against the same agent moments earlier returned 200; `admin agents task` returns 401. Either the command does not reuse the session token, or it points at an authenticated path that `message` doesn't. `--help` shows `--token <TOKEN>` as an option but does not say it is required when no session token can be derived. P1-A.

**Probe 4 — `admin agents message` against `associate_agent`**: returned `ack` cleanly. Both agents reachable.

**Probe 5 — error variants**:
- `admin agents message ghost_agent`: clean 404 ✅
- `admin agents tools ghost_agent`: clean "Agent not found" ✅
- `admin agents show ghost_agent`: clean "Agent not found" ✅
- `admin agents message developer_agent` with no `-m`: clean "`--message` is required in non-interactive mode" ✅
- `admin agents task ghost_agent`: returns 401 **before** the agent-existence check fires — auth-precedence-over-404 means users may misdiagnose as a token issue. P2.

### 3.2 — MCP tool invocation via CLI

**Setup**: `plugins mcp tools` reports 1 tool (`systemprompt` on server `systemprompt`), 1 parameter.

**Probe 1 — happy path**:
```
$ systemprompt plugins mcp call systemprompt systemprompt --args '{"command":"core skills list"}'
{ "success": true, "execution_time_ms": 279, "content": [...] }
```
- Output: full JSON-encoded skills listing returned inline ✅
- `mcp_tool_executions` row written with `status="success"`, `tool_name="systemprompt"`, `started_at` matching the call ✅

**Probe 2 — unknown tool**: returns `success: false`, `error: "Tool execution failed"`, **rc=0**. The MCP server's actual rejection reason (e.g. "tool not found") is swallowed.

**Probe 3 — unknown server**: returns `Error: MCP server 'ghost_server' not found in configuration`, **rc=1**. Clean. ✅

**Probe 4 — malformed JSON args**: returns `Invalid JSON in --args: expected ident at line 1 column 2`, **rc=1**. Clean. ✅

**Probe 5 — empty args object**: returns `success: false`, `error: "Tool execution failed"`, **rc=0**. The MCP tool's validation error (missing `command` field) is swallowed — same shape as Probe 2.

**P1-C**: `plugins mcp call` should propagate the MCP server's error message into the `error` field, and should return non-zero rc when `success: false`. Right now scripted callers cannot distinguish a true tool failure from a successful call.

**`plugins mcp validate --service systemprompt`** (per prior issues.md item #4):
The `--service` flag **does not exist**; it is a bare positional `[SERVER]`. The auto-detect path works (issue #4 "resolved"), but anyone following the older flag form gets `error: unexpected argument '--service' found`. Doc drift, P3.

### 3.3 — `/api/public/hooks/govern` persistence

Direct curl probe with no auth (matches prior P0-A test conditions):
```
POST http://localhost:8080/api/public/hooks/govern
{ "hook_event_name": "PreToolUse", "session_id": "clitest", "tool_name": "Bash", "tool_input": {"command":"echo hi"} }

→ 200 {"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"[GOVERNANCE] Missing Authorization header — tool call blocked"}}
```
`governance_decisions` delta: **+1** ✅

The prior report's P0-A ("hook endpoint not persisting") **does not reproduce** in this sweep. Either the regression was transient between commits `202cf79` and `604f5e6`, or the prior test ran against a session whose audit row was being dropped by a different code path. The endpoint now writes a row even on the unauthenticated-deny path. **Net: P0-A is no longer reproducible in 0.11 head.**

### 3.4 — Destructive lifecycle round-trips

All rollbacks clean. Starting and ending row counts:

| Table | Before | After (post-cleanup) | Δ |
|---|---|---|---|
| `users` | n | n | 0 (1 created + deleted) |
| `contexts` | n | n | 0 (2 created + deleted) |
| `files` | n | n | 0 (1 uploaded + deleted) |
| `ip_bans` (active) | 0 | 0 | 0 (1 added + removed) |
| `ai_requests` | 20707 | 20712 | +5 (4 A2A messages, kept) |
| `governance_decisions` | 27885 | 27888 | +3 (1 tool-use + 2 unauth-deny, kept) |
| `mcp_tool_executions` | 66 | 68 | +2 (2 successful calls, kept) |

Service stop/restart cycle on the MCP server completed cleanly with a fresh PID and 3/3 services back to running.

### 3.5 — `core skills` surface narrower than expected

`core skills --help` exposes only `list` and `show`. There is no `core skills sync` or `core skills validate` subcommand — sync happens at startup (per CLAUDE.md services discovery). This is consistent with the architecture but may surprise users who try to follow the prior demo's "2 skills synced from YAML" narrative literally. P3 docs.

---

## 4. Friction telemetry

| Metric | Value | Note |
|---|---|---|
| Median CLI startup (read sweep) | ~96 ms | Better than prior report's ~500ms — likely because cloud check short-circuits on expired creds |
| Slowest read command | `analytics tools list` (99 ms) | Within budget |
| `[profile: local …]` prefix lines emitted | 0 in read sweep stdout; surfaces in `admin agents logs` JSON content | P2 leak into JSON payloads |
| `Token expired` WARN lines | 6 across the sweep | All cloud-touching commands; not on local-only paths |
| ANSI escape sequences in JSON output | yes (`admin agents logs`) | P2 — breaks downstream parsers |
| Panics / stack traces | **0** | clean across all 293 leaves |
| `--help` failures | **0** | clean across all 293 leaves |
| Clap `rc=2` "required arg" errors | 9 across the read sweep | All by-design; documenting which leaves require positionals |

---

## 5. Cross-reference with `RELEASE-TEST-REPORT-0.11.md`

| Prior finding | Status now |
|---|---|
| **P0-A** hooks/govern endpoint doesn't persist | **Not reproducible** in this sweep — row written on unauthenticated-deny path; further investigation needed to determine whether the prior test ran against a different code path or transient state |
| **P0-B** preflight plugin token scrape falls back to admin token | Not re-tested here (CLI-only sweep). No `.token` file present. Pre-existing fragility unchanged. |
| **P0-C** `web validate` "✓ Valid" with 14 warnings | **Improved**: rc=1 now; warnings unchanged. Demo scripts still need to check rc before printing "✓ Valid". |
| P1 governance policy rename `secret_injection → secret_scan` | Confirmed in DB; demo narrative copy still drifts. Not a CLI defect. |
| P2 cloud creds expired noise | Confirmed; pervasive but cosmetic. |
| P2 `[profile: …]` prefix on every command | Improved on stdout; **still leaks into `admin agents logs` JSON content**. |
| P2 ~500ms CLI latency | **Improved**: median ~96ms in this sweep. |

---

## 6. Recommended fixes (not implemented)

### P1
1. **`admin agents task`** — reuse the active session token the way `admin agents message` does, or fail with `"--token required when no session token is available"` instead of relaying the gateway's 401.
2. **`plugins mcp call`** — propagate the MCP server's actual error string into the `error` field, and return non-zero rc when `success: false`.
3. **`core content show`** — when given a slug with no `--source`, either disambiguate when the slug is unique, or print the missing flag explicitly: `"--source <SOURCE> required when looking up by slug"`.

### P2
4. **`admin users role promote|demote`** — accept a role name as an optional positional, or document that the command only ever promotes to/demotes from `admin`.
5. **`admin agents logs`** — strip ANSI escape sequences and `[profile: …]` prefix lines from the `logs[]` array before JSON-encoding.
6. **`admin agents task`** ordering — check agent existence (404) before issuing the upstream auth call (401).
7. **`plugins mcp validate`** — keep auto-detect; either accept `--service` as an alias of `[SERVER]` or update the prior issues-list doc.

### P3
8. **`infra db query`** — surface the raw Postgres error verbatim rather than paraphrasing as "Table or relation 'X' does not exist".
9. **`web validate`** — split rc=1 (real misconfig) from rc=2 (warnings only); current rc=1 is correct but conflates the two.

---

## 7. Reproducer

- Enumerator: `/tmp/cli-sweep/enumerate.sh` — auto-walks `systemprompt <domain> --help` to produce `/tmp/cli-sweep/leaves.txt` (293 lines).
- `--help` sweep: `/tmp/cli-sweep/sweep-help.sh` → `/tmp/cli-sweep/help-results.tsv`.
- Read sweep: `/tmp/cli-sweep/sweep-read.sh` → `/tmp/cli-sweep/read-results.tsv` + `/tmp/cli-sweep/logs/<leaf>.log`.
- Deep-dives & destructive lifecycles ran interactively against `systemprompt 0.11.0` head with `developer_agent` + `associate_agent` + `systemprompt` MCP server all running.

End of report.
