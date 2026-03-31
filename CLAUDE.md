# Enterprise Demo

**Use the CLI to discover commands.** `systemprompt --help` is your starting point.

---

## Quick Start

```bash
# Build
just build

# Start services
just start

# Discover CLI commands
systemprompt --help

# List skills
systemprompt core skills list

# Show a skill
systemprompt core skills show <skill_id>
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

## Dev Standards

Before writing code, read the relevant standards skill:

- **Rust**: [services/plugins/systemprompt-dev/skills/dev-rust-standards/SKILL.md](services/plugins/systemprompt-dev/skills/dev-rust-standards/SKILL.md)
- **Frontend (JS/CSS)**: [services/plugins/systemprompt-dev/skills/dev-frontend-standards/SKILL.md](services/plugins/systemprompt-dev/skills/dev-frontend-standards/SKILL.md)
- **Architecture**: [services/plugins/systemprompt-dev/skills/dev-architecture-standards/SKILL.md](services/plugins/systemprompt-dev/skills/dev-architecture-standards/SKILL.md)

For extension development, read the relevant extension skill:

- **Data Providers**: [services/plugins/systemprompt-dev/skills/dev-ext-data-providers/SKILL.md](services/plugins/systemprompt-dev/skills/dev-ext-data-providers/SKILL.md)
- **Rendering**: [services/plugins/systemprompt-dev/skills/dev-ext-rendering/SKILL.md](services/plugins/systemprompt-dev/skills/dev-ext-rendering/SKILL.md)
- **Infrastructure (Jobs/Router/Schemas)**: [services/plugins/systemprompt-dev/skills/dev-ext-infrastructure/SKILL.md](services/plugins/systemprompt-dev/skills/dev-ext-infrastructure/SKILL.md)
- **Content Feeds (RSS/Sitemap)**: [services/plugins/systemprompt-dev/skills/dev-ext-feeds/SKILL.md](services/plugins/systemprompt-dev/skills/dev-ext-feeds/SKILL.md)
- **AI & Tool Providers**: [services/plugins/systemprompt-dev/skills/dev-ext-providers/SKILL.md](services/plugins/systemprompt-dev/skills/dev-ext-providers/SKILL.md)
- **Hooks & Events**: [services/plugins/systemprompt-dev/skills/dev-ext-hooks/SKILL.md](services/plugins/systemprompt-dev/skills/dev-ext-hooks/SKILL.md)

Generate plugin output: `systemprompt core plugins generate --id systemprompt-dev`

---

## Critical Rules

1. **`core/` is READ-ONLY** — Never modify. It's a git submodule.
2. **Rust code -> `extensions/`** — All `.rs` files live here.
3. **Config only -> `services/`** — YAML/Markdown only. No Rust code.
4. **CSS files -> `storage/files/css/`** — NEVER put CSS in `extensions/*/assets/css/`.
5. **Brand name is `Enterprise Demo`** — Use "Enterprise Demo" for display, "demo.systemprompt.io" for URLs.
6. **It's a library, not a framework** — Embedded code you own and extend. NEVER call it a "framework".

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

This runs (in order): `compile_admin_templates` -> `bundle_admin_css` -> `bundle_admin_js` -> `copy_extension_assets` -> `content_prerender`. Order matters — bundles must be built before `copy_extension_assets` copies them to `web/dist/`.

---

## Claude Code Plugins

Plugins bundle agents, skills, and MCP servers for use with Claude Code. Each plugin lives in `services/plugins/<plugin-id>/`.

### Running Claude Code with a Plugin

```bash
# Run with a specific plugin
claude --plugin services/plugins/systemprompt-admin

# Run with multiple plugins
claude --plugin services/plugins/systemprompt-admin --plugin services/plugins/systemprompt-dev
```

### Available Plugins

| Plugin | Path | Purpose |
|--------|------|---------|
| `systemprompt-admin` | `services/plugins/systemprompt-admin/` | Platform administration (users, analytics, agents, services, logs, jobs, database) |
| `systemprompt-dev` | `services/plugins/systemprompt-dev/` | Development toolkit (10 skills, 6 agents: standards, architecture, extensions) |
| `common-skills` | `services/plugins/common-skills/` | Shared skills (Odoo, brand, excalidraw, content) |
| `sales-skills` | `services/plugins/sales-skills/` | Sales CRM (reports, debug, emails, health) |

### Plugin Structure

```
services/plugins/<plugin-id>/
  config.yaml          <- Plugin manifest (skills, agents, MCP servers)
  .claude-plugin/
    plugin.json        <- Claude Code plugin metadata
  .mcp.json            <- MCP server connections (if any)
  agents/              <- Agent system prompt docs (*.md)
  hooks/               <- Event hooks
  skills/              <- Plugin-local skill overrides
```

### Admin Skills Reference

The `systemprompt-admin` plugin provides these skills (see `services/skills/` for full docs):

| Skill | File | CLI Domain |
|-------|------|------------|
| User Management | [`systemprompt_admin_user_management/index.md`](services/skills/systemprompt_admin_user_management/index.md) | `admin users` |
| Analytics | [`systemprompt_admin_analytics/index.md`](services/skills/systemprompt_admin_analytics/index.md) | `analytics` |
| Agent Management | [`systemprompt_admin_agent_management/index.md`](services/skills/systemprompt_admin_agent_management/index.md) | `admin agents` |
| Log Management | [`systemprompt_admin_logs/index.md`](services/skills/systemprompt_admin_logs/index.md) | `infra logs` |
| Service Management | [`systemprompt_admin_services/index.md`](services/skills/systemprompt_admin_services/index.md) | `infra services` |
| Job Scheduling | [`systemprompt_admin_jobs/index.md`](services/skills/systemprompt_admin_jobs/index.md) | `infra jobs` |
| Database Management | [`systemprompt_admin_database/index.md`](services/skills/systemprompt_admin_database/index.md) | `infra db` |
| Skill Creation | [`systemprompt_admin_skill_creation/index.md`](services/skills/systemprompt_admin_skill_creation/index.md) | `core skills`, `core plugins` |
