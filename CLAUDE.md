# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Enterprise Demo

**Use the CLI to discover commands.** `systemprompt --help` is your starting point.

---

## Quick Start

```bash
# First-time setup: writes .systemprompt/profiles/local/, starts Docker Postgres,
# runs publish_pipeline. At least one AI key is required.
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
systemprompt core skills sync --help
```

---

## Architecture (big picture)

- `src/main.rs` is a thin entry point that delegates to the read-only `core/` submodule. All customization is **compile-time** via the [`inventory`](https://docs.rs/inventory) crate — there is no dynamic plugin loader.
- Rust code lives in `extensions/`: `extensions/mcp/*` for MCP server extensions, `extensions/web` for page data and template rendering. Each MCP extension has its own crate with `Cargo.toml` + `.sqlx/` offline cache.
- Configuration is YAML under `services/`, loaded through `services/config/config.yaml`'s explicit `includes:` list. Unknown keys error loudly (`#[serde(deny_unknown_fields)]`).
- Governance runs as a four-stage synchronous pipeline on every tool call: **scope check → secret scan (35+ patterns) → blocklist → rate limit**. Every decision is audited to Postgres with a trace_id linking identity → agent → tool → result → cost.
- Per-clone Docker Postgres: `just db-up / db-down / db-reset / db-logs [tenant=local]`. Project name is derived from a hash of the repo path, so multiple clones on one host get isolated containers and volumes.
- Deploy flow: `just build-all` (release binary + MCP servers + web assets) then `just deploy`. The `publish_pipeline` job also runs automatically at server startup.

---

## Debugging & Troubleshooting

```bash
# Quick error check
systemprompt infra logs view --level error --since 1h

# Debug AI request failures
systemprompt infra logs request list --limit 10
systemprompt infra logs audit <request-id> --full

# Debug MCP tool failures
systemprompt plugins mcp logs <server-name>

# Debug agent issues
systemprompt infra logs trace list --agent <agent-name> --status failed
```

**Key debugging workflow:**
1. `infra logs view --level error` — Find the error
2. `infra logs request list` — Find failed AI requests
3. `infra logs audit <id> --full` — Get full conversation context
4. `plugins mcp logs <server>` or `logs/mcp-*.log` — Get MCP tool errors

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
  web/config.yaml           Web frontend config (full WebConfig)
  content/config.yaml       Content source config
```

Unknown YAML keys cause loud errors at load time (`#[serde(deny_unknown_fields)]`). Nested `includes:` resolve recursively. Plugin YAMLs are binding descriptors that reference top-level agents, skills, mcp servers, and content sources by id — never inline copies.

---

## Critical Rules

1. **`core/` is READ-ONLY** — Never modify. It's a git submodule.
2. **Rust code -> `extensions/`** — All `.rs` files live here.
3. **Config only -> `services/`** — YAML/Markdown only. No Rust code.
4. **CSS files -> `storage/files/css/`** — NEVER put CSS in `extensions/*/assets/css/`.
5. **Brand name is `Enterprise Demo`** — Use "Enterprise Demo" for display, "demo.systemprompt.io" for URLs.
6. **It's a library, not a framework** — Embedded code you own and extend. NEVER call it a "framework".
7. **Demo scripts must work on macOS and Linux** — BSD vs GNU differ on `grep -oP`, `head -n -1`, `sha256sum`, `sed -i`, and binary downloads (pick `hey_darwin_amd64` vs `hey_linux_amd64`). `demo/_common.sh` provides `install_hey()` for the last case; prefer `grep -oE` + `sed -n 's/.../\1/p'` over `grep -oP … \K …`.

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

This runs (in order): `bundle_admin_css` -> `bundle_admin_js` -> `copy_extension_assets` -> `content_prerender`. Order matters — bundles must be built before `copy_extension_assets` copies them to `web/dist/`. Admin pages are SSR'd at runtime from `.hbs` templates in `storage/files/admin/templates/`, not precompiled.

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
