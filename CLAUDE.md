# SystemPrompt

**Everything is done through playbooks.** Read `guide_start` first.

## MANDATORY: Read Playbook Before Any Task

```bash
# ALWAYS START HERE - Read the getting started guide
systemprompt core playbooks show guide_start
```

Then read the specific playbook for your task and follow it exactly.

---

## Critical Rules

1. **`core/` is READ-ONLY** — Never modify. It's a git submodule.
2. **Rust code → `extensions/`** — All `.rs` files live here.
3. **Config only → `services/`** — YAML/Markdown only. No Rust code.
4. **Use playbooks** — Read the playbook before starting ANY task.

---

## Playbook Reference

| Task | Playbook ID |
|------|-------------|
| **Start here** | `guide_start` |
| Architecture | `build_architecture` |
| Rust standards | `build_rust-standards` |
| Build extension | `build_extension-checklist` |
| Review extension | `build_extension-review` |
| Build MCP server | `build_mcp-checklist` |
| Review MCP server | `build_mcp-review` |
| File structure | `build_file-structure` |
| Extension boundaries | `build_boundaries` |
| Services ↔ Extensions | `build_services-extensions` |
| Session/auth | `cli_session` |
| Agent operations | `cli_agents` |
| Service management | `cli_services` |
| Database queries | `cli_database` |
| View logs | `cli_logs` |
| Deploy | `cli_deploy` |
| Content creation | `content_blog`, `content_linkedin`, `content_twitter` |

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
