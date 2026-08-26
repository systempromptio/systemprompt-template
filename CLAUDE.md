# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Enterprise Demo

**Use the CLI to discover commands.** `systemprompt --help` is your starting point.

---

## Branching & Release Flow

**All work lands on `next`. Never push to `main`.**

`next` is the repository's default branch, so a fresh clone starts there. `main`
is protected by a ruleset that requires a pull request and grants **no bypass to
anyone** — a direct `git push origin main` is refused for agents, sessions and
repository admins alike. Protection is pinned to `main` by name, so moving the
default branch does not move it.

```
next   ← default branch. Every agent, every session. Push freely.
         Bar to land: it builds and works. No gates, no test suites.
  ↓ `just gate` when you are ready, then `just promote` to open the release PR
         This is where every pre-release check runs.
main   ← protected, release-only. Tagged. Never pushed to directly.
```

**What you owe before pushing to `next`:** the code compiles and is functional.
That is the whole bar. Run `just build` once at the end of your change set, make
sure it does not error, and push. Do **not** run `just gate`, the test suites, or
any other pre-release check as part of landing on `next`.

**The pre-release cycle belongs to the `next` -> `main` transition, not to
`next`.** There is no scheduled job and nothing gating a push to `next`. The
gates run when a person decides to promote, and only then:

1. `just gate [REF]` — dispatches every gate workflow against the ref
   (default: the tip of `next`) and waits.
2. `just promote [SHA]` — freezes that commit on the `promote` ref and **opens**
   the release pull request onto `main`. It does not merge; you do.
3. Tag `main` once merged. Tags are not covered by the ruleset.

The commit is frozen on `promote` rather than the PR being headed at `next`
because a PR headed at `next` merges whatever `next` points at *when you merge
it* — anything pushed meanwhile would ride along ungated. That happened once
for real.

## Building Against Local Core (`next` tracks `next`)

While `next` is open, this template builds against the **sibling
`../systemprompt-core` checkout on its own `next` branch**, not against the
published crates.io release. Core `next` carries unreleased API changes, so a
build resolved from crates.io would not be validating the code that actually
ships together.

**The patch is a local working-tree edit and is never committed.** What is
committed always pins a published core with both `[patch.crates-io]` blocks
commented out, so nobody else ends up building against a path on your machine.
A `.git/hooks/pre-commit` guard rejects any commit that stages an active patch
block or a local `../systemprompt-core` path.

Two blocks route the `systemprompt-*` crates at the local checkout, and **they
must be uncommented and commented in lockstep**:

| Manifest | Patches |
|----------|---------|
| `Cargo.toml` | the root workspace: the binary and every `extensions/` crate |
| `tests/Cargo.toml` | the `tests/` workspace, which is a *separate* workspace |

`[patch]` only applies from the manifest of the workspace being built. Patch the
root alone and the test crates silently resolve core from crates.io while the
extensions under test resolve it locally, so the suites compile against a
different core than the binary does. That mismatch surfaces as a confusing
"variant not found" error naming a path under `~/.cargo/registry/`. If you see a
core path in an error that is not `../systemprompt-core/`, the patch blocks are
out of lockstep.

To enable the patch locally, uncomment both blocks and hide all four files from
git so ordinary commits never carry them:

```bash
git update-index --skip-worktree Cargo.toml Cargo.lock tests/Cargo.toml tests/Cargo.lock
```

The lockfiles are hidden alongside the manifests because resolving against the
local checkout rewrites them too. To land a genuine manifest change (a new
dependency, a version bump), clear the flag with `--no-skip-worktree`, comment
the patch blocks back out, commit, then re-enable both.

`just core-bump` refuses to run while the root block is active: publish core,
bump the pinned version, and re-comment both blocks before promoting to `main`.

**Do not run `just prepare` while the patch is active.** It bakes core's own
queries into the template's `.sqlx/` cache. Local core edits need core's
per-crate cache regenerated in the core checkout instead.

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
- Governance runs as a four-stage synchronous pipeline on every tool call: **scope check → secret scan (35+ patterns) → blocklist → rate limit**. Every decision is audited to Postgres with a trace_id linking identity → agent → tool → result → cost.
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

All runtime configuration lives as flat YAML files under `services/`. The root `services/config/config.yaml` is a thin aggregator with an explicit `includes:` list — every resource file must be listed.

```
services/
  config/config.yaml        Root aggregator (includes all resource files)
  agents/<id>.yaml          Flat agent definitions
  mcp/<name>.yaml           Flat MCP server definitions
  skills/<id>.yaml          Flat skill definitions
  skills/<id>.md            Skill instruction bodies (referenced via !include)
  plugins/<name>.yaml       Flat plugin binding descriptors
  ai/config.yaml            AI provider config
  scheduler/config.yaml     Job scheduler
  slack/<name>.yaml         Inbound Slack apps (`slack_apps:` map — ships disabled)
  web/config.yaml           Web frontend config (full WebConfig)
  content/config.yaml       Content source config
```

Unknown YAML keys cause loud errors at load time (`#[serde(deny_unknown_fields)]`). Nested `includes:` resolve recursively. Plugin YAMLs are binding descriptors that reference top-level agents, skills, mcp servers, and content sources by id — never inline copies.

---

## Slack (inbound, off by default)

`services/slack/example.yaml` ships **disabled**. Fill in the workspace id, install a
Slack app, put `slack_signing_secret` / `slack_bot_token` in the profile secret store,
then flip `enabled: true`. Core mounts the transport already
(`POST /api/v1/slack/{events,commands,interactivity}`); the binary opts in with the
`slack` feature in `Cargo.toml`.

Route the slash command, not a channel: core's event handler dispatches on both
`message` and `app_mention`, so a routed channel sends every line of chatter to the
agent. The example routes `/systemprompt` to `developer_agent`, which already carries
the `systemprompt` MCP server and `oauth.scopes: [admin]`.

Four gates stand between a Slack message and a tool call, each denying by default:

1. **Workspace** — `authz.allowed_roles` is projected at startup into an
   `access_control_rules` row for `slack_workspace:<workspace_id>` with
   `default_included=false` (`repositories/config/acl_yaml_loader.rs`).
2. **Identity** — the sender must map to an account holding the granted role.
   `link_by_workspace_email: true` attaches them to the account owning their *confirmed*
   Slack email; otherwise link by hand with
   `POST /api/public/admin/users/{user_id}/slack-identity` (`{"slack_user_id": "U…"}`),
   `DELETE` to detach. An unlinked sender becomes a role-less first-touch user and fails
   gate 1.
3. **Token** — core mints the A2A token with the sender's own permissions (`admin` role
   ⇒ `Admin`, everyone else ⇒ `User`) and audience `[a2a, mcp]`; an agent declaring
   `oauth.scopes: [admin]` rejects the weaker token.
4. **MCP server** — `services/mcp/systemprompt.yaml` requires audience `mcp` and scope
   `admin`, and `roles.yaml` grants `mcp_server:systemprompt` to `admin` only.

Bot scopes: `commands`, `chat:write`, `users:read`, plus `users:read.email` only if
`link_by_workspace_email` is on.

---

## Critical Rules

1. **Core is a crate dependency** — pinned to crates.io for published builds, and every commit keeps it that way. Locally, both `[patch.crates-io]` blocks (root + `tests/`) are uncommented as a working-tree edit held back by `skip-worktree`, routing core at the sibling `../systemprompt-core` checkout on its `next` branch. That checkout IS editable for cross-repo work. Publish + bump before promoting to `main`. See [Building Against Local Core](#building-against-local-core-next-tracks-next).
2. **Rust code -> `extensions/`** — All `.rs` files live here.
3. **Config only -> `services/`** — YAML/Markdown only. No Rust code.
4. **CSS files -> `storage/files/css/`** — NEVER put CSS in `extensions/*/assets/css/`.
5. **Brand name is `Enterprise Demo`** — Use "Enterprise Demo" for display, "demo.systemprompt.io" for URLs.
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

Plugins are flat YAML files under `services/plugins/<name>.yaml` that aggregate agents, skills, mcp servers, and content sources by reference:

```yaml
plugins:
  enterprise-demo:
    id: enterprise-demo
    name: "Enterprise Demo"
    version: "2.0.0"
    enabled: true
    agents:
      include: []
    skills:
      include:
        - example_web_search
        - use_dangerous_secret
    mcp_servers: []
    content_sources: []
```

Every id listed must resolve to a real top-level resource in `services/`. `ServicesConfig::validate()` enforces this at load time.
