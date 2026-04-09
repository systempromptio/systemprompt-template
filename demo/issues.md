# Demo Issues & Friction

All 36 demos now pass (exit 0). This file documents issues found during testing, including friction that degrades the demo experience even when scripts don't fail.

---

## Fixed During Testing

These were caught and fixed:

| Script | Issue | Fix |
|--------|-------|-----|
| `infrastructure/02-database.sh` | `infra db migrations` requires subcommand | Changed to `infra db migrations status` |
| `governance/07-rate-limiting.sh` | `admin config rate-limits`, `security`, `server` all require subcommands | Added `show` subcommand to each |
| `skills/02-content-management.sh` | `core content popular` requires `<SOURCE>` argument | Added `documentation` argument |
| `analytics/02-agent-analytics.sh` | `analytics agents show developer_agent` fails if no agent activity exists | Added graceful fallback with `|| info` |
| `mcp/01-mcp-servers.sh` | `plugins mcp list` fails (AppPaths not initialized); `plugins mcp logs` fails (log files not found); `plugins mcp validate` requires `--service` flag; `plugins mcp tools` doesn't accept positional args | Rewrote to use `plugins mcp status` and `plugins mcp tools --server <name>` |
| `agents/05-agent-registry.sh` | `admin agents registry` fails with JSON parse error (`missing field 'url'`) | Added `|| info` fallback |
| `users/04-ip-ban.sh` | `admin users ban remove` requires `--yes` flag | Added `--yes` |
| `web/01-web-config.sh` | `web content-types`, `templates`, `assets` all require subcommands | Added `list` subcommand to each |
| `web/02-sitemap-validate.sh` | `web sitemap` requires subcommand | Changed to `web sitemap show` |
| `web/` and `cloud/` directories | Missing — background agents failed to create them | Recreated manually |

---

## CLI Bugs (Not Demo Bugs)

These are issues in the CLI itself that affect demo quality:

### 1. `plugins mcp list` — AppPaths not initialized
```
Error: Failed to list MCP servers
Caused by: Failed to load services configuration
           AppPaths not initialized - call AppPaths::init() first
```
**Impact:** Cannot demonstrate MCP server listing via `plugins mcp list`. Workaround: use `plugins mcp status` instead.

### 2. `plugins mcp logs <server>` — Log files not found
```
Error: Failed to get MCP server logs
Caused by: Log file not found for service 'systemprompt'. Available: []
```
**Impact:** Cannot show MCP server logs. DB log query also fails (app context init). No workaround — MCP logs are invisible.

### 3. `admin agents registry` — JSON parse error
```
Error: Failed to get agent registry
Caused by: Failed to parse registry response
           missing field `url` at line 1 column 13251
```
**Impact:** A2A agent discovery fails. The registry endpoint returns data but the CLI can't parse it. Likely a schema mismatch between the running agents and the CLI's expected struct.

### 4. `plugins mcp validate` — Requires `--service` in non-interactive mode
```
Error: --service is required in non-interactive mode
```
**Impact:** Can't validate all MCP servers at once from a script. Need to specify each server individually.

---

## Friction & UX Issues

### 5. Verbose DEBUG output pollutes demo output
Every CLI command outputs multiple DEBUG lines (cloud credentials, reqwest connections, pool management, secrets loading, extension discovery). This is extremely noisy for demos.

**Example:** A simple `infra services status` outputs 10+ lines of DEBUG before the actual status.

**Suggestion:** Default log level should be WARN or ERROR for demo scripts. Consider adding `RUST_LOG=error` to `_common.sh`, or add a `--quiet` mode that suppresses all log output.

### 6. Slow startup per command (~500ms)
Every CLI invocation validates cloud credentials against `api.systemprompt.io` (HTTPS roundtrip), loads secrets, and discovers 12 extensions. This adds ~500ms per command.

**Impact:** A demo with 5 steps takes ~2.5s of CLI startup overhead. The infrastructure/02-database.sh demo with 10 steps takes ~5s just in startup.

**Suggestion:** Cache credential validation. Skip cloud validation for local profiles. Or add a `--skip-cloud-check` flag.

### 7. `[profile: local ...]` prefix on every command
Every command prints `[profile: local (local) | tenant: local_19c2d6bdf6c]` which adds visual noise.

**Suggestion:** Suppress this in `--quiet` mode or when `--json`/`--yaml` output is requested.

### 8. No data on fresh install
Many analytics, logs, and trace demos show "No data found" on a fresh install. This is correct behavior, but the demo experience is poor.

**Suggestion:** Add a `demo/seed-data.sh` script that runs governance demos + an agent message first, so subsequent analytics/logs/trace demos have data to display.

### 9. `00-preflight.sh` requires manual user creation
On first run, preflight fails because no user exists in the database. Had to manually run `admin users create` + `admin users role promote`.

**Suggestion:** Preflight should auto-create the admin user if it doesn't exist, or clearly direct the user to do so.

### 10. Content status command path issue
```
systemprompt core content status --source documentation
```
Works but shows `--source` as a flag — some content commands use positional args (like `popular <SOURCE>`) while others use flags. Inconsistent API.

---

## Missing Demo Coverage

### 11. No demo for `agents/03-agent-messaging.sh` (skipped in test run)
This is the only demo that calls an AI model (~$0.01). It was skipped in the automated test run. Needs manual verification.

### 12. No demo for `performance/01-request-tracing.sh` or `performance/02-load-test.sh`
These were not in the automated test because they require specific token handling and download external tools. Need manual verification.

### 13. Conversations analytics not demoed
`analytics conversations stats/trends/list` exists but has no demo script. Consider adding.

### 14. Tool analytics not fully demoed
`analytics tools stats/list/trends/show` is only partially covered in `mcp/03-mcp-tool-execution.sh`.

### 15. `admin config show` (full config overview) not demoed
Could be useful to show the full configuration overview in one command.

### 16. `core contexts` not demoed as standalone
Context CRUD (list, show, create, edit, delete) is only shown as part of `agents/03-agent-messaging.sh`. Could be its own demo.

### 17. `core hooks` demo is limited
`core hooks list` and `core hooks validate` are in `skills/04-plugin-management.sh` but not prominent. Consider a standalone hooks demo, especially if more hook types are added.

---

## Structural Issues

### 18. Backward-compatible wrappers reference old demo names
The wrapper `demo/06-governance-secret-breach.sh` redirects to `demo/governance/06-secret-breach.sh` (different name). Similarly `demo/07-mcp-access-tracking.sh` redirects to `demo/mcp/02-mcp-access-tracking.sh`. Old documentation/scripts referencing these paths will work but the internal numbering has changed.

### 19. Architecture docs still reference flat numbering
`demo/architecture/*.md` files use the old numbered scheme (00-09). They should be updated to reference the new category structure.

### 20. Documentation site pages not updated
The 11 `services/content/documentation/demo*.md` files still reference the old flat demo paths. Need updating to link to the new category structure.

### 21. SVG recording scripts reference old demo paths
The migrated `demo/recording/svg/svg-*.sh` scripts reference the old governance API calls and paths. Need updating to work with the new category structure.
