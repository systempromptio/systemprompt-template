---
title: "Playbook Authoring Guide"
description: "Write machine-executable playbooks. Concise, deterministic, self-repairing."
author: "SystemPrompt"
slug: "guide-playbook"
keywords: "playbook, authoring, meta, standards"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Playbook Authoring

Write machine-executable instruction sets for agents and users.

---

## Core Principles

1. **Deterministic** — Exact commands, not suggestions
2. **Testable** — Every command verified before commit
3. **Bounded** — One domain per playbook, link to others via CLI
4. **Self-repairing** — Agents fix broken playbooks immediately
5. **No bloat** — Every line serves a purpose
6. **No inline comments** — Code blocks contain only executable content

---

## File Location

```
services/playbook/
├── guide/        guide_* — Onboarding, meta
├── cli/          cli_* — All CLI operations (commands, setup, tracking)
├── build/        build_* — Development standards (Rust, extensions)
│   └── cloud/    build_cloud_* — Docker, containers, deployment
└── content/      content_* — Content creation workflows
```

Filename becomes playbook ID: `cli/agents.md` → `cli_agents`

---

## Playbook Organization

### When to Split

Split a playbook when:
- **Multiple domains**: Covers 3+ distinct CLI domains (e.g., `core`, `admin`, `infra`)
- **Excessive length**: Exceeds 200 lines
- **Mixed concerns**: Combines operations that have different audiences

Keep combined when:
- Commands are tightly coupled
- Users need to read all content sequentially
- Under 150 lines

### Multi-Topic Playbooks

When a domain covers multiple sub-topics, use clear section headers within a single file:

```markdown
# Plugins & MCP Server Playbook

## Extensions
(Extension management commands)

## MCP Servers
(MCP server lifecycle commands)

## MCP Tools
(Tool listing and calling commands)

## Capabilities
(Extension capabilities commands)
```

This keeps related content together while maintaining clear boundaries.

### Naming Conventions

| Pattern | Playbook ID | File Path |
|---------|-------------|-----------|
| CLI playbook | `cli_agents` | `cli/agents.md` |
| Build playbook | `build_architecture` | `build/architecture.md` |
| Cloud playbook | `build_cloud_docker` | `build/cloud/docker.md` |
| Guide playbook | `guide_start` | `guide/start.md` |
| Content playbook | `content_blog` | `content/blog.md` |

### Cross-References

Use relative paths between playbooks:

```markdown
-> See [Session Playbook](session.md) for authentication.
-> See [Build Playbook](../build/architecture.md) for architecture.
```

---

## Required Structure

```markdown
---
title: "Title"
description: "Single sentence. What it does."
keywords:
  - keyword1
  - keyword2
---

# Title

Single-line description matching frontmatter.

> **Help**: `{ "command": "..." }` via `systemprompt_help`
> **Requires**: Prerequisites (if any) -> See [Playbook](path.md)

---

## Section

Commands in JSON:

{ "command": "domain subcommand args" }

---

## Quick Reference

| Task | Command |
|------|---------|
| Do X | `domain subcommand` |
```

---

## Checklist

- [ ] YAML frontmatter with title, description, keywords
- [ ] H1 title matches frontmatter title
- [ ] Single-line description after H1
- [ ] `> **Help**:` block with MCP command reference
- [ ] `> **Requires**:` block if prerequisites exist
- [ ] Horizontal rules (`---`) between sections
- [ ] Commands in JSON code blocks (no comments)
- [ ] Tables for option lists and quick reference
- [ ] `## Quick Reference` table at end
- [ ] Links to related playbooks use relative paths
- [ ] No prose paragraphs — use lists, tables, code
- [ ] All CLI commands validated with `--help`
- [ ] All core code links verified to exist
- [ ] Single domain per playbook (split if 3+ domains)
- [ ] Under 200 lines (split if exceeding)
- [ ] Folder structure with index.md if multiple subtopics

---

## Command Format

### Standard CLI Command

```json
{ "command": "admin agents list --enabled" }
```

### Command with Placeholder

```json
{ "command": "admin agents show <name>" }
```

### Terminal-Only Command

```bash
just login
just build
```

Mark clearly when command requires terminal (not MCP).

---

## Linking to Core Code

For build/extension playbooks, link to actual implementation in core:

| Link Type | Format |
|-----------|--------|
| Trait definition | `core/crates/shared/traits/src/extension.rs` |
| Config example | `core/crates/shared/config/src/profile.rs` |
| Database schema | `core/crates/infra/database/schema/` |

### GitHub URL Pattern

```
https://github.com/systempromptio/systemprompt-core/blob/main/<path>
```

### Example Reference

```markdown
-> Implements `Extension` trait from [core/crates/shared/traits/src/extension.rs](https://github.com/systempromptio/systemprompt-core/blob/main/crates/shared/traits/src/extension.rs)
```

---

## Linking Rules

### Cross-Reference Other Playbooks

```markdown
-> See [Session Playbook](../cli/session.md) for authentication.
```

### Prerequisites Block

```markdown
> **Requires**: Active session -> See [Session Playbook](../cli/session.md)
```

### Within Troubleshooting

```markdown
**Context issues** -- See [Contexts Playbook](../cli/contexts.md) for solutions.
```

---

## Writing Rules

| Rule | Good | Bad |
|------|------|-----|
| Commands | `{ "command": "admin agents list" }` | "Run the agents list command" |
| Code blocks | No comments | `// MCP: systemprompt` |
| Structure | Bullet lists, tables | Prose paragraphs |
| Scope | One domain | Multiple concerns |
| Links | `-> See [X](path.md)` | Inline explanations |
| Placeholders | `<name>`, `<id>` | `{name}`, `$NAME` |
| Errors | Table with issue/solution | Narrative explanations |

---

## Troubleshooting Section Pattern

```markdown
## Troubleshooting

**Issue name** -- Solution in one sentence. Command if applicable.

| Issue | Solution |
|-------|----------|
| Agent not responding | Check `admin agents status <name>` |
| Command not found | Verify CLI version with `--version` |
```

---

## Validation Protocol

Before committing a playbook:

**1. Validate all CLI commands**

```bash
systemprompt admin agents list --help
systemprompt core playbooks sync --help
```

**2. Verify playbook links resolve**

```bash
ls services/playbook/cli/session.md
ls services/playbook/cli/contexts.md
```

**3. Verify core code links (if any)**

```bash
curl -s -o /dev/null -w "%{http_code}" \
  "https://github.com/systempromptio/systemprompt-core/blob/main/crates/shared/traits/src/extension.rs"
```

**4. Sync to database**

```bash
systemprompt core playbooks sync --direction to-db -y
```

**5. Verify accessible**

```bash
systemprompt core playbooks show <playbook_id>
```

---

## Self-Repair Protocol

When a playbook command fails:

1. **Stop current task**
2. **Find correct syntax**: `systemprompt <domain> <subcommand> --help`
3. **Edit playbook file** in `services/playbook/`
4. **Sync**: `systemprompt core playbooks sync --direction to-db -y`
5. **Verify**: `systemprompt core playbooks show <playbook_id>`
6. **Resume task**

---

## Anti-Patterns

| Anti-Pattern | Problem | Fix |
|--------------|---------|-----|
| Inline comments | Not executable | Remove all comments from code blocks |
| Prose explanations | Not scannable | Use tables/lists |
| Untested commands | Breaks agent execution | Validate with `--help` before commit |
| Broken core links | Dead references | Verify URLs return 200 |
| Multiple domains | Scope creep | Split into linked playbooks |
| Oversized file | Hard to maintain | Split if >200 lines, use folder pattern |
| Vague instructions | Non-deterministic | Use exact commands |
| Missing Quick Reference | No summary | Add table at end |

---

## Quick Reference

| Task | Command |
|------|---------|
| List playbooks | `core playbooks list` |
| Show playbook | `core playbooks show <id>` |
| Show raw | `core playbooks show <id> --raw` |
| Filter by category | `core playbooks list --category build` |
| Sync to database | `core playbooks sync --direction to-db -y` |
| Validate CLI command | `systemprompt <domain> <subcommand> --help` |