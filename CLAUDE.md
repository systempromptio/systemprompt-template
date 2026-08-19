# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Astound Digital

**Use the CLI to discover commands.** `systemprompt --help` is your starting point.

---

## Quick Start

```bash
# First-time setup: writes .systemprompt/profiles/local/, starts Docker Postgres,
# runs publish_pipeline. With no key arg, the CLI prompts for which provider to
# use; the chosen provider becomes ai.default_provider (others disabled) and the
# gateway default. Passing keys is non-interactive — the first becomes default.
just setup-local                                                          # interactive provider pick
just setup-local <anthropic_key> [openai_key] [gemini_key] [http_port=8080] [pg_port=5432]

# Build (auto-uses live DB if reachable, else SQLX_OFFLINE=true)
just build            # debug
just build --release  # release

# Lint (workspace, -D warnings, same offline fallback as build)
just clippy

# Regenerate .sqlx/ offline query cache (needs live DB)
just prepare

# Start services
just start

# Discover CLI commands
systemprompt --help

# List skills
systemprompt core skills list
```

---

## Shared Build State (read this before you compile)

Several agents work this clone at once. Builds, clippy, and tests are expensive
and take a shared cargo lock, so a build started mid-iteration stalls everyone.

**Do all the work first, validate once at the end.** Never run `just build` or
`just clippy` between edits to see how you're doing; finish the change set, then
run the gate a single time.

**Check the shared state before spending anything:**

```bash
just build-status     # in-flight run + last result per recipe + is it still fresh?
just server-status    # running server, its binary, and whether that binary is stale
```

`just build`, `just clippy`, `just test-*`, and `just lint-gates` are
single-flight (`scripts/build-coordinator.sh`). They key on a content
fingerprint of the source tree, so:

| situation | what happens |
|-----------|--------------|
| this tree already passed this recipe | returns immediately, no compile |
| identical run already in flight | attaches to its log, exits with its status |
| someone else's run in flight | queues, then runs |

Results land in `.build/` (gitignored): `runs.jsonl`, `latest/<recipe>.json`,
`logs/`, `binaries.jsonl`. Read them instead of re-running.

`just start` reports the running server first, then starts. It does not restart
a server another agent is already running (say so and stop), and it warns when
the binary predates the current source, but it only refuses outright when there
is no binary at all. Staleness is reported from the ledger when the binary came
from a coordinated build, and from file mtimes otherwise.

Always go through the justfile. A bare `cargo build` bypasses coordination and
re-creates the contention. Escape hatches when you truly need them:
`BUILD_FORCE=1`, `START_FORCE=1`, `BUILD_NO_COORD=1`.

---

## Preflight (this repo has no CI — these gates ARE the CI)

```bash
just preflight          # the mandatory pre-merge gate: static → lint → tests → coverage
just preflight-static   # seconds: fmt, sqlx cache, 23 source gates
just preflight-lint     # clippy (both workspaces), doc-check, msrv-check
just preflight-full     # weekly: preflight + deny + audit + machete + hack
just init-hooks         # once per clone: tracked .githooks/ (pre-commit + pre-push)
```

`verify` = preflight minus the coverage tier; use it mid-iteration, `preflight`
before merging. The pre-push hook runs the static tier automatically
after `just init-hooks`.

**Coverage floor + ratchet.** `just coverage` runs an instrumented llvm-cov
pass over all three workspaces (root, `tests/`, `bridge/`) into
`coverage-report/` (gitignored); `just coverage-check` enforces the tracked
`coverage/baseline.json` — a global floor, a 0.5pt total ratchet, and per-crate
ratchets. If you raised coverage, re-record with `just coverage-baseline` and
commit the file; lowering it is a review-visible act. Never use cargo-llvm-cov
here (it re-injects the mold linker flags and silently produces zero profraws
— see `scripts/coverage.sh`).

Tests live in the `tests/` workspace (`unit/`, `integration/`, `contract/`) or
in-crate `tests/` dirs — inline `#[cfg(test)]` modules are banned. DB-backed
suites need Docker Postgres up (`just db-up`).

---

## CLI Structure

```
systemprompt <domain> <subcommand> [args]
```

| Domain | Purpose |
|--------|---------|
| `core` | Skills, content, files, contexts, plugins, hooks, artifacts |
| `infra` | Services, database, jobs, logs |
| `admin` | Users, agents, config, setup, session |
| `cloud` | Auth, deploy, sync, secrets, tenant, domain |
| `analytics` | Overview, conversations, agents, tools, requests, sessions, content, traffic, costs |
| `web` | Content-types, templates, assets, sitemap, validate |
| `plugins` | Extensions, MCP servers, capabilities |
| `build` | Build core workspace and MCP extensions |

**Use `systemprompt <domain> --help` to explore any domain.**

---

## CLI Discovery Workflow

When you need to perform a task, use the CLI help to find the right command:

```bash
# Top-level help
systemprompt --help

# Domain help
systemprompt core --help
systemprompt infra --help

# Subcommand help
systemprompt core skills --help
systemprompt core skills show --help
```

---

## Architecture (big picture)

- `src/main.rs` is a thin entry point that delegates to the published `systemprompt` core crates (sibling checkout at `../systemprompt-core`, patched in via `[patch.crates-io]` for cross-repo work). All customization is **compile-time** via the [`inventory`](https://docs.rs/inventory) crate — there is no dynamic plugin loader.
- Rust code lives in `extensions/`: `extensions/mcp/*` for MCP server extensions, `extensions/web` for page data and template rendering. Each MCP extension has its own crate with `Cargo.toml` + `.sqlx/` offline cache.
- Configuration is YAML under `services/`, loaded through `services/config/config.yaml`'s explicit `includes:` list. Unknown keys error loudly (`#[serde(deny_unknown_fields)]`).
- Governance is a four-stage synchronous pipeline on every tool call: **scope check → secret scan (35+ patterns) → blocklist → rate limit**. Every decision is audited to Postgres with a trace_id linking identity → agent → tool → result → cost. **All four stages are disabled in this installation** (`services/governance/config.yaml`), as are the gateway safety scanners (`services/gateway/policies.yaml`) — both files carry the reason and the instructions to switch them back on. The chain still runs and still audits: calls are recorded as `decision=allow, policy=governance_disabled`. Authentication is separate and is *not* disabled — an invalid or expired token is still denied, with `policy=authentication`. Do not disable governance by deleting the config file: a missing file falls back to defaults, which is all four stages **enabled**.
- Per-clone Docker Postgres: `just db-up / db-down / db-logs [tenant=local]`. Project name is derived from a hash of the repo path, so multiple clones on one host get isolated containers and volumes. There is no destructive reset recipe — recover migration checksum drift in place with `just repair-migrations`.
- Deploy flow: `just build-all` (release binary + MCP servers + web assets) then `just deploy`. The `publish_pipeline` job also runs automatically at server startup.

---

## Debugging & Troubleshooting

```bash
# Quick error check
systemprompt infra logs view --level error --since 1h

# Debug AI request failures
systemprompt infra logs request list --limit 10
systemprompt infra logs audit <request-id>

# Debug MCP tool failures
systemprompt plugins mcp logs <server-name>

# Debug agent issues
systemprompt infra logs trace list --agent <agent-name> --status failed
```

**Key debugging workflow:**
1. `infra logs view --level error` — Find the error
2. `infra logs request list` — Find failed AI requests
3. `infra logs audit <id>` — Get full conversation context
4. `plugins mcp logs <server>` or `logs/mcp-*.log` — Get MCP tool errors

---

## Viewing Governance

Every inference call (`/v1/messages`) and every MCP tool call lands a row in the governance spine. Same CLI surface for both — no separate "gateway logs" vs "tool logs":

```bash
# Every AI request — user, model, token counts, cost, latency, status
systemprompt infra logs request list --limit 20
systemprompt infra logs request list --since 1h --provider anthropic   # request list filters: --since / --model / --provider (no --status)
systemprompt infra logs trace list --status failed          # only failed runs — --status lives on trace list, not request list

# Full audit for one request — identity, policy evals, prompt, response, cost
systemprompt infra logs audit <request-id>

# Tool-call traces (PreToolUse → decision → spawn → result)
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace list --agent <name> --status failed
systemprompt infra logs trace show <trace-id>

# Cost + usage rollups (hits the same audit table)
systemprompt analytics costs summary
systemprompt analytics requests stats
systemprompt analytics agents
systemprompt analytics tools
```

`logs request list` shows one row per `/v1/messages` hit — the gateway path Pi / any Anthropic-SDK client uses. `logs trace list` shows MCP tool calls. Both are backed by the same 18-column `ai_requests` / trace tables with `user_id`, `tenant_id`, `session_id`, `trace_id` — so `audit <id>` reconstructs the chain from identity to cost.

**`infra logs` vs `analytics` — operational vs dashboard.** The `infra logs request {list,stats}` commands are quick operational views (recent rows, by-provider / by-model aggregate). Their `analytics requests {list,stats}` counterparts are dashboard metrics over a time range with model filtering, cache-hit rate, and CSV export. Same `ai_requests` table underneath — reach for `infra logs` when triaging a live issue, `analytics` when reporting. The `--help` on each cross-references the other.

For live tailing while reproducing an issue: `infra logs view --follow --since 30s`.

---

## Services Configuration

All runtime configuration lives as YAML under `services/`. The root `services/config/config.yaml` is a thin aggregator with an explicit `includes:` list — every **flat** resource file must be listed there. Skills and plugins are the exception: they are auto-discovered from their nested directories and must *not* be added to `includes:`.

```
services/
  config/config.yaml        Root aggregator (includes the flat resource files)
  agents/<id>.yaml          Flat agent definitions (none ship here — see below)
  mcp/<name>.yaml           Flat MCP server definitions
  skills/<id>/config.yaml   Skill definitions (nested dir, auto-discovered)
  plugins/<id>/config.yaml  Plugin binding descriptors (nested dir, auto-discovered)
  governance/config.yaml    Policy chain (disabled here — see Architecture)
  gateway/policies.yaml     Gateway quotas + safety scanners (scanners off here)
  ai/config.yaml            AI provider config
  scheduler/config.yaml     Job scheduler
  web/config.yaml           Web frontend config (full WebConfig)
  content/config.yaml       Content source config
```

Unknown YAML keys cause loud errors at load time (`#[serde(deny_unknown_fields)]`). Nested `includes:` resolve recursively. Plugin YAMLs are binding descriptors that reference top-level agents, skills, mcp servers, and content sources by id — never inline copies.

**This instance ships no A2A agents.** `services/config/config.yaml` says so explicitly: nothing under `services/agents/`, nothing spawned on the agent port range, and no `agents/<id>.md` in any plugin bundle. Skills, MCP servers and artifacts carry the capability instead — so `admin agents list` returning nothing is the correct answer here, not a fault.

---

## Critical Rules

1. **Core is a crate dependency** — consumed from crates.io; the sibling `../systemprompt-core` checkout IS editable for cross-repo work via the `[patch.crates-io]` toggle (publish + bump + re-comment before landing).
   **Adopting a new core version:** bump the pins in **both** `Cargo.toml` and `tests/Cargo.toml` — a stale pin silently drops the patch and resolves the old crate from crates.io, so the build passes having proved nothing. Verify with `scripts/sync-release-version.sh <version> --check` and confirm the build log names the sibling path. Then run migrations with the **new** binary, `just prepare` to refresh the offline cache, and read core's changelog for tightened identifier validators and new `NOT NULL` columns — both are runtime failures that `cargo build` cannot catch (a validated `ContextId` panics in `new()`; a `NOT NULL` column breaks seed migrations that never named it). Full procedure: `docs/RELEASING.md` Step A0.
2. **Rust code -> `extensions/`** — All `.rs` files live here.
3. **Config only -> `services/`** — YAML/Markdown only. No Rust code.
4. **CSS files -> `storage/files/css/`** — NEVER put CSS in `extensions/*/assets/css/`.
5. **Brand name is `Astound Digital`** — Use "Astound Digital" for display, "astounddigital.com" for URLs.
6. **It's a library, not a framework** — Embedded code you own and extend. NEVER call it a "framework".
7. **Demo scripts must work on macOS and Linux** — BSD vs GNU differ on `grep -oP`, `head -n -1`, `sha256sum`, `sed -i`, and binary downloads (pick `hey_darwin_amd64` vs `hey_linux_amd64`). `demo/_common.sh` provides `install_hey()` for the last case; prefer `grep -oE` + `sed -n 's/.../\1/p'` over `grep -oP … \K …`.
8. **No Co-Authored-By in commits** — `coauthorAttribution: false` is set in `.claude/settings.json`. Never add `Co-Authored-By:` trailers to commit messages.

---

## Repository Naming Convention

Every function under `extensions/web/admin/src/repositories/` is named for what
it returns, so a call site reads the same as its signature:

| Returns | Prefix | Example |
|---------|--------|---------|
| `Vec<T>` — zero or more rows | `list_` | `list_top_users` |
| `Option<T>` — a row that may be absent | `find_` | `find_session_header` |
| `T` — exactly one value, or an error | `get_` | `get_request_stats` |
| a page plus its total, `(Vec<T>, i64)` | `list_` | `list_requests_paged` |

Mutations keep the verb that describes them: `insert_`, `update_`, `delete_`,
`set_`, `count_`.

`scripts/check-repository-naming.sh` enforces this: it rejects `fetch_`
outright, and checks every other prefix against the function's actual return
type, so the table above cannot quietly stop being true.

`fetch_` is banned because it is not a synonym for the three above —
it was doing all three jobs at once, which is how the convention drifted: a
reader could not tell from `fetch_summary` whether an absent row was `None` or
an error, and had to open the file to find out.

---

## CSS Files (IMPORTANT)

**All CSS files go in `storage/files/css/`** and must be registered in `extensions/web/src/extension.rs`.

```
storage/files/css/          <- CSS SOURCE (put files here)
extensions/web/src/extension.rs  <- REGISTER here in required_assets()
web/dist/css/               <- OUTPUT (generated, never edit)
```

**To add CSS:**
1. Create file in `storage/files/css/`
2. Register in `extension.rs` `required_assets()`
3. `just publish` to compile templates, bundle CSS/JS, and copy all assets to `web/dist/`

---

## Publishing Assets

After changing templates, CSS, JS, or static files, run:

```bash
just publish
```

This runs (in order): `bundle_admin_css` -> `copy_extension_assets` -> `content_prerender`. Order matters — bundles must be built before `copy_extension_assets` copies them to `web/dist/`. Admin pages are SSR'd at runtime from `.hbs` templates in `storage/files/admin/templates/`, not precompiled.

**Exception: the public-site partials are compiled into the binary.** `services/web/templates/partials/{head-assets,header,footer,scripts}.html` are `include_str!`-embedded by `extensions/web/site/src/partials.rs`. Editing them requires a rebuild (`just build`) and a server restart before `just publish` — running publish alone keeps serving the markup baked into the old binary.

---

## Plugins

Each plugin is a directory holding one `config.yaml` — `services/plugins/<id>/config.yaml` — auto-discovered, so it is **not** listed in the root aggregator's `includes:`. The file's root key is `plugin:` (singular; one plugin per file). It aggregates agents, skills, mcp servers, and content sources by reference:

```yaml
plugin:
  id: astound-commons
  name: "Astound Commons — Workspace Setup"
  version: "2.0.0"
  enabled: true
  skills:
    source: explicit
    include:
      - cowork_setup
      - apply_brand_voice
  agents:
    source: explicit
    include: []
  mcp_servers: {}
```

`source:` selects where members come from — keep it `explicit` and list ids under `include:`. Leaving it to default to `Instance` makes the plugin claim every skill and agent on the instance, which is how the marketplace once showed every plugin with all 230 skills.

Every id listed must resolve to a real top-level resource in `services/`. `ServicesConfig::validate()` enforces this at load time.

Skills follow the same nested shape: `services/skills/<id>/config.yaml`, with the instruction body beside it.
