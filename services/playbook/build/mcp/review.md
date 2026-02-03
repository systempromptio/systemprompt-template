---
title: "MCP Server Review Playbook"
description: "Code review process for MCP servers on systemprompt-core."
keywords:
  - mcp
  - review
  - compliance
  - validation
category: build
---

# MCP Server Review

> **Help**: `{ "command": "core playbooks show build_mcp-review" }`

> Review this MCP server as though you were Steve Klabnik implementing world-class idiomatic Rust.

> **Reference**: Use `extensions/mcp/systemprompt/` as the canonical example for comparison.

---

## Input

- **Folder:** `{server_path}`
- **Checklist:** `systemprompt core playbooks show build_mcp-checklist`
- **Standards:** `systemprompt core playbooks show build_rust-standards`

---

## Steps

1. Verify required files exist:
   - `Cargo.toml`
   - `src/main.rs`
   - `module.yml`

2. Verify directory structure:
   - `src/tools/` (if providing tools)
   - `src/prompts/` (if providing prompts)
   - `src/resources/` (if providing resources)

3. Read all `.rs` files in `{server_path}/src/`

4. Read `Cargo.toml`

5. Read `module.yml`

6. Execute each checklist item from the mcp-checklist playbook

7. For each violation, record: `file:line` + violation type

8. Generate `status.md` using output template

---

## Validation Commands

```bash
# Structure checks
test -f {server_path}/Cargo.toml
test -f {server_path}/src/main.rs
test -f {server_path}/module.yml

# Binary target
grep -q "\[\[bin\]\]" {server_path}/Cargo.toml

# Code quality
cargo clippy -p {server_name} -- -D warnings
cargo fmt -p {server_name} -- --check

# Build
cargo build -p {server_name}
```

---

## Output

Generate `{server_path}/status.md` using the status-template playbook.

**Verdict:** COMPLIANT if zero violations. NON-COMPLIANT otherwise.

---

## Status Template

```markdown
# {crate_name} Compliance

**Layer:** MCP Server
**Reviewed:** {YYYY-MM-DD}
**Verdict:** {COMPLIANT | NON-COMPLIANT}

---

## Checklist

| Category | Status |
|----------|--------|
| Required Structure | pass/fail |
| Tool Quality | pass/fail |
| Code Quality | pass/fail |

---

## Violations

| File:Line | Violation | Category |
|-----------|-----------|----------|
| `src/tools/foo.rs:42` | `unwrap()` usage | Code Quality |
| `src/main.rs:15` | Missing logging init | Entry Point |

{Or: "None"}

---

## Commands Run

cargo clippy -p {crate_name} -- -D warnings  # {PASS/FAIL}
cargo fmt -p {crate_name} -- --check          # {PASS/FAIL}
cargo build -p {crate_name}                   # {PASS/FAIL}

---

## Actions Required

1. {Action to fix violation}
2. {Action to fix violation}

{Or: "None - fully compliant"}
```

---

## Quick Reference

| Task | Command |
|------|---------|
| Run review | Follow steps above |
| Check lint | `cargo clippy -p {crate} -- -D warnings` |
| Check format | `cargo fmt -p {crate} -- --check` |
| Build | `cargo build -p {crate}` |
| Generate status | Create `status.md` in server root |

-> See [MCP Checklist](build_mcp-checklist) for full checklist.
-> See [Rust Standards](build_rust-standards) for code quality rules.
-> See [Architecture](build_architecture) for dependency rules.
