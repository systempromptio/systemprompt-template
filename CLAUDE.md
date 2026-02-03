# systemprompt.io

**Everything is done through playbooks.** Guides are your starting point.

---

## Guides (Start Here)

**Guides are the principal entry points for all tasks.** Each guide links to detailed playbooks.

| Guide | Command | Use For |
|-------|---------|---------|
| **Getting Started** | `systemprompt core playbooks show guide_start` | First read, master index |
| **Coding Standards** | `systemprompt core playbooks show guide_coding-standards` | All code standards, Rust patterns |
| **Playbook Authoring** | `systemprompt core playbooks show guide_playbook` | Writing/editing playbooks |
| **Documentation** | `systemprompt core playbooks show guide_documentation` | Writing documentation |
| **Recipes** | `systemprompt core playbooks show guide_recipes` | Quick recipes for common tasks |

### Guide Pathways

```
guide_start ─────────────────────────────────────────────────────────────
    │
    ├── CLI Operations ────────► cli_session, cli_agents, cli_services...
    │
    ├── Building ──────────────► guide_coding-standards
    │                                   │
    │                                   ├── build_rust-standards
    │                                   ├── build_extension-checklist
    │                                   └── build_mcp-checklist
    │
    ├── Content ───────────────► content_blog, content_linkedin...
    │
    └── Writing Playbooks ─────► guide_playbook
```

---

## MANDATORY: Playbook-First Workflow

**NO TASK may be started without a playbook.** This is non-negotiable.

```bash
# ALWAYS START HERE - Read the getting started guide
systemprompt core playbooks show guide_start

# For coding tasks - Read coding standards first
systemprompt core playbooks show guide_coding-standards
```

### The Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│  1. FIND PLAYBOOK                                               │
│     systemprompt core playbooks list                            │
│     systemprompt core playbooks show <playbook_id>              │
│                           │                                      │
│                           ▼                                      │
│  2. PLAYBOOK EXISTS? ─────┬─────────────────────────────────    │
│                           │                                      │
│     YES                   │                    NO                │
│      │                    │                     │                │
│      ▼                    │                     ▼                │
│  3a. FOLLOW IT            │              3b. CREATE IT           │
│      exactly              │                  (see below)         │
│                           │                                      │
│                           ▼                                      │
│  4. PLAYBOOK ACCURATE? ───┬─────────────────────────────────    │
│                           │                                      │
│     YES                   │                    NO                │
│      │                    │                     │                │
│      ▼                    │                     ▼                │
│  5a. COMPLETE TASK        │              5b. FIX IT FIRST        │
│                           │                  then continue       │
└─────────────────────────────────────────────────────────────────┘
```

### If Playbook is Inaccurate

**STOP.** Do not continue with broken instructions.

1. Identify the issue (bug in playbook OR outdated instructions)
2. Verify correct behavior: `systemprompt <domain> <subcommand> --help`
3. Edit the playbook file in `services/playbook/`
4. Sync: `systemprompt core playbooks sync --direction to-db -y`
5. Verify fix: `systemprompt core playbooks show <playbook_id>`
6. Resume task with corrected playbook

### If No Playbook Exists

**STOP.** Do not improvise. Create the playbook FIRST.

1. Research the task thoroughly (CLI help, existing code, related playbooks)
2. Audit existing patterns in `services/playbook/`
3. Create playbook following `guide_playbook` standards
4. Sync: `systemprompt core playbooks sync --direction to-db -y`
5. Verify: `systemprompt core playbooks show <new_playbook_id>`
6. NOW execute the task following your new playbook

**Playbook locations:**
- `services/playbook/guide/` — guide_* playbooks
- `services/playbook/cli/` — cli_* playbooks
- `services/playbook/build/` — build_* playbooks
- `services/playbook/content/` — content_* playbooks

---

## Debugging & Troubleshooting

**When something fails, use the logging CLI.** See `cli_logs` playbook for full reference.

```bash
# Quick error check
systemprompt infra logs view --level error --since 1h

# Debug AI request failures
systemprompt infra logs request list --limit 10
systemprompt infra logs audit <request-id> --full

# Debug MCP tool failures (vague "tool failed" errors)
systemprompt plugins mcp logs <server-name>
grep -i "error\|failed" logs/mcp-<server-name>.log | tail -30

# Debug agent issues
systemprompt infra logs trace list --agent <agent-name> --status failed
```

**Key debugging workflow:**
1. `infra logs view --level error` — Find the error
2. `infra logs request list` — Find failed AI requests
3. `infra logs audit <id> --full` — Get full conversation context
4. `plugins mcp logs <server>` or `logs/mcp-*.log` — Get MCP tool errors

-> Full playbook: `systemprompt core playbooks show cli_logs`

---

## Critical Rules

1. **`core/` is READ-ONLY** — Never modify. It's a git submodule.
2. **Rust code → `extensions/`** — All `.rs` files live here.
3. **Config only → `services/`** — YAML/Markdown only. No Rust code.
4. **CSS files → `storage/files/css/`** — NEVER put CSS in `extensions/*/assets/css/`.
5. **Playbooks are mandatory** — Find, fix, or create a playbook BEFORE any task.
6. **Brand name is `systemprompt.io`** — Always lowercase. NEVER "SystemPrompt" (caps).
7. **It's a library, not a framework** — Embedded code you own and extend. NEVER call it a "framework".

---

## CSS Files (IMPORTANT)

**All CSS files go in `storage/files/css/`** and must be registered in `extensions/web/src/extension.rs`.

```
storage/files/css/          <- CSS SOURCE (put files here)
├── core/variables.css
├── components/header.css
├── feature-base.css
├── feature-rust.css
└── ...

extensions/web/src/extension.rs  <- REGISTER here in required_assets()

web/dist/css/               <- OUTPUT (generated, never edit)
```

**To add CSS:**
1. Create file in `storage/files/css/`
2. Register in `extension.rs` `required_assets()`
3. `just build && systemprompt infra jobs run copy_extension_assets`

See playbook: `systemprompt core playbooks show build_web-assets`

---

## Playbook Categories

| Category | Purpose | Example IDs |
|----------|---------|-------------|
| `guide_*` | **Start here** — Principal entry points | `guide_start`, `guide_coding-standards`, `guide_playbook` |
| `cli_*` | All CLI operations | `cli_session`, `cli_agents`, `cli_cloud`, `cli_analytics` |
| `build_*` | Development standards | `build_architecture`, `build_rust-standards` |
| `content_*` | Content creation | `content_blog`, `content_linkedin` |

## Key Playbooks

### Guides (Always Start Here)

| Task | Playbook ID |
|------|-------------|
| **First read** | `guide_start` |
| **Coding standards** | `guide_coding-standards` |
| **Writing playbooks** | `guide_playbook` |
| **Documentation** | `guide_documentation` |

### CLI Operations

| Task | Playbook ID |
|------|-------------|
| Session/auth | `cli_session` |
| Secrets management | `cli_secrets` |
| Cloud setup | `cli_cloud` |
| Analytics & tracking | `cli_analytics` |
| View logs & SQL | `cli_logs` |
| Agent operations | `cli_agents` |
| Agent mesh | `cli_mesh` |
| Service management | `cli_services` |
| MCP & plugins | `cli_plugins` |
| Database queries | `cli_database` |
| Jobs & scheduler | `cli_jobs` |
| Deploy | `cli_deploy` |

### Building

| Task | Playbook ID |
|------|-------------|
| Architecture | `build_architecture` |
| Rust standards | `build_rust-standards` |
| Build extension | `build_extension-checklist` |
| Build MCP server | `build_mcp-checklist` |

### Content

| Task | Playbook ID |
|------|-------------|
| Blog posts | `content_blog` |
| LinkedIn | `content_linkedin` |
| Twitter | `content_twitter` |

---

## Quick Start

```bash
# Build
just build

# Start services
just start

# Read the getting started guide
systemprompt core playbooks show guide_start

# List all playbooks
systemprompt core playbooks list

# Read a specific playbook
systemprompt core playbooks show <playbook_id>

# Raw markdown output (for piping)
systemprompt core playbooks show <playbook_id> --raw

# Sync playbooks if missing
systemprompt core playbooks sync --direction to-db -y
```

---

## CLI Structure

```
systemprompt <domain> <subcommand> [args]
```

| Domain | Purpose |
|--------|---------|
| `core` | Content, playbooks, skills, files, contexts |
| `infra` | Services, database, jobs, logs |
| `admin` | Users, agents, config, session |
| `cloud` | Cloud deployment, sync |
| `analytics` | Metrics & insights |
| `web` | Frontend configuration |
| `plugins` | MCP servers, extensions |
| `build` | Build MCP extensions |

**For details on any domain, read the corresponding playbook.**
