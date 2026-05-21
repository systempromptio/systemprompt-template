# Demo Suite + CLI Stress-Test Report — post-0.11 (patched core HEAD)

Generated: 2026-05-21
Core: `../systemprompt-core` HEAD `10d322e5` (post-`v0.11.0`, includes audit
hardening, RS256 verify fix, oauth state-binding + JTI revocation, plugin-env
no-synth, `admin keys issue-plugin-token`, `web validate` strict exit, MCP
allowed_hosts default, trace_id propagation, etc.)
Template: working tree on `main` with in-flight changes (this report).
Build: `target/release/systemprompt` (release, LTO thin).

---

## 1. Build & lint gate

| Step | Result |
|---|---|
| `just prepare` (sqlx, live DB, 957 queries cached) | OK |
| `just build --release` | OK (4m 16s) |
| `just clippy` (workspace, `-D warnings`, no new `#[allow]`) | OK |
| `just publish` (14 pipeline steps) | OK |
| `just lint-no-synthesis` guard (via `just clippy`) | OK |

---

## 2. Service bring-up

| Step | Result |
|---|---|
| `target/release/systemprompt infra services start --profile local --kill-port-process` | OK |
| Health (`GET /health`) | `{"status":"healthy"}` |
| Processes | gateway + `systemprompt-mcp-agent` + `developer_agent` (9101) + `associate_agent` (9102) |
| `./demo/00-preflight.sh` | OK (after Step 3 rewrite — see §6) |
| `./demo/01-seed-data.sh` | OK (governance_decisions: 27937, plugin_usage_events: 270, markdown_content: 71) |

---

## 3. Demo sweep (`./demo/sweep.sh`)

44 scripts, **44 pass / 0 fail** after one fix (see §6).

Categories: agents (5), analytics (8), cloud (1), governance (9), infrastructure
(5), mcp (3), performance (2), skills (5), users (4), web (2). Includes the two
paid agent-messaging demos.

---

## 4. CLI stress probes

### 4a. Authentication / session / token

| Probe | Evidence |
|---|---|
| `admin session show --profile local` | active session, 23h 5m remaining |
| `admin session list` | 4 profiles enumerated (local + 3 others) |
| `admin keys issue-plugin-token --email admin@localhost.dev --profile local --token-only` | RS256 JWT, `aud=[api,plugin]`, `plugin_id=cowork-bundle`, 365-day expiry, signs to real admin user (`sub=3eb86e75-…`, `email=admin@localhost.dev`) — no synthesized identity |
| `cloud auth status` | `token_status: Expired` reported (no panic; local-only mode honored — verifies `8fbe9b13` gating) |
| Tampered JWT → `GET /admin/profile` | HTTP 307 (redirect to login). No impersonation, no panic. |
| Valid plugin token → `GET /admin/profile` | HTTP 200 |
| Unauth → `GET /admin/profile` | HTTP 307 |

### 4b. MCP

| Probe | Evidence |
|---|---|
| `plugins mcp list` | `systemprompt` server, port 5010, status `ready` |
| `plugins mcp status` | running, PID 1803931, release + debug binaries present |
| `plugins mcp logs systemprompt` | DB log source returns `MCP service started` |
| `plugins mcp validate` | `total:1 valid:1 invalid:0` (`unhealthy:1` is the documented OAuth-required indicator) |
| `plugins mcp tools` | 1 tool: `systemprompt` (CLI passthrough) |
| `plugins mcp list-packages` | `["systemprompt"]` |
| `plugins mcp call systemprompt systemprompt --args '{"command":"core skills list"}'` | `success:true`, 87 ms, returns 2 skills (`example_web_search`, `use_dangerous_secret`) as table artifact |
| `infra logs trace show <trace_id>` post-call | MCP event recorded with user + session, 50 ms latency |

### 4c. Agent messaging (paid, ~$0.0007 actual)

| Probe | Evidence |
|---|---|
| `admin agents list` | both agents present, `developer_agent` primary/default |
| `admin agents registry` | gateway A2A registry returns 2 agents with full `url`/`version`/`status`/`streaming`/`skills_count` (verifies `23d82d5`-era registry parse fix) |
| `admin agents show developer_agent` | `provider:gemini`, `model:gemini-2.5-flash`, 2 skills, 1 mcp server |
| `admin agents tools developer_agent` | 1 MCP tool exposed |
| `admin agents status` | both running (PIDs 1804127, 1804131) |
| `admin agents message developer_agent -m 'Reply OK only'` | `OK` (724 ms) |
| `admin agents message developer_agent --json --blocking -m 'Reply OK only'` | full task JSON: status `TASK_STATE_COMPLETED`, history with user+agent roles, contextId/taskId propagated |
| `infra logs trace show <trace_id>` | full chain: skills evaluated → understanding → planning → AI call (gemini, 1109 in / 1 out tok, $0.000335, 520 ms) → completion, attributed to user `3eb86e75-…` session `sess_bf33fa…` |
| `infra logs audit <request_id> --full` | full conversation history reconstructed; user identity, model, token counts, cost all present |

---

## 5. Security commits exercised end-to-end

| Core commit | Exercised by | Verdict |
|---|---|---|
| `9bf8dfcf` audit-insert hardening | governance/00-audit-write-smoke, governance/03-audit-trail, every `logs audit --full` probe | OK |
| `07bfa07d` RS256 verify + agent MCP bootstrap | all MCP calls + every CLI auth path | OK |
| `971bc91b` + `a793b181` oauth state-binding / JTI revocation | session login/show round-trip | OK |
| `04463ac9` `admin keys issue-plugin-token` | preflight Step 3 (rewritten), §4a | OK |
| `8fbe9b13` gated cloud cred bootstrap | `cloud auth status` with expired token returns clean expired state | OK |
| `90335633` trace_id propagation | §4b trace_show, §4c trace_show | OK |
| `db917b58` nbf + act_chain depth cap | plugin-token mint embeds `nbf` claim verified in decode | OK (claim present, expired-nbf path not negatively probed) |
| `189c635a` `web validate` non-zero on warnings | demo/web/02 (script updated to tolerate) | OK (exit semantics correct; warnings real — see §7) |
| `603377f1` MCP allowed_hosts default fix | MCP server starts cleanly, no allowed_hosts errors | OK |

The "plugin-env handler removed synthesized admin" path is implicitly covered:
the tampered-token probe returns 307 not 200, and the minted plugin token's
`sub`/`email` resolve to the real seeded admin row (`admin@localhost.dev`).

---

## 6. Fixes applied during this run

Two changes, kept minimal:

1. **`demo/00-preflight.sh` Step 3** — replaced the HTML-scrape of
   `/admin/profile` with a call to the new
   `admin keys issue-plugin-token --token-only` command. The scrape path was
   correctly failing loudly per the post-0.11 CHANGELOG; the mint command has
   now landed in core (`04463ac9`), so preflight should use it. Token contract
   reflected in the diff: `aud=[api,plugin]`, `plugin_id=cowork-bundle` (no
   `scope=service` — that was speculative in the old comments).
2. **`demo/web/02-sitemap-validate.sh`** — tolerate non-zero exit from
   `systemprompt web validate` (per `189c635a` it now exits 1 on warnings).
   The script's purpose is to *display* validation output, not gate the demo.

Neither change touches `core/`, no `#[allow]` was added, no YAML was edited.

---

## 7. Known follow-ups (file into `demo/issues.md`)

- **Template/content-type drift in `services/web/`.** `web validate` reports 14
  warnings: templates `docs-page`, `homepage`, `feature`, `feature-page`,
  `blog-list`, `blog-post`, `legal-post` reference content types (`docs`,
  `docs-index`, `docs-list`, `guide`, `reference`, `tutorial`, `homepage`,
  `feature`, `feature-page`, `blog-list`, `blog`, `legal`, `page`) that are not
  declared in any `services/content/*` source's `allowed_content_types`. Plus
  one config warning: `services/web/assets` directory missing. Pre-existing —
  surfaced by `189c635a` flipping the exit code. Real config gap to clean up
  but out of scope for this validation pass.
- **`cloud auth status` token expired.** Working as designed in local-only
  mode. If air-gap/cloud demos are needed, run `systemprompt cloud login`.

---

## 8. Verdict

Template (working tree) + patched core HEAD `10d322e5` is **build-clean,
lint-clean, and functionally green end-to-end**. 44/44 demo sweep scripts pass.
All three high-risk surfaces (auth, MCP, agent messaging) exercised against
the release binary with trace + audit reconstruction confirmed. No `core/`
edits, no synthesized identities, no `#[allow]` suppressions, no YAML
write-back.
