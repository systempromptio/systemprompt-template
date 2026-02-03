---
title: "Extension Review Playbook"
description: "Code review process for extensions on systemprompt-core."
keywords:
  - extension
  - review
  - compliance
  - validation
category: build
---

# Extension Review

> **Help**: `{ "command": "core playbooks show build_extension-review" }`

> Review this extension as though you were Steve Klabnik implementing world-class idiomatic Rust.

> **Reference**: Use `extensions/blog/` as the canonical example for comparison.

---

## Input

- **Folder:** `{extension_path}`
- **Checklist:** `systemprompt core playbooks show build_extension-checklist`
- **Standards:** `systemprompt core playbooks show build_rust-standards`

---

## Steps

1. Verify required files exist:
   - `Cargo.toml`
   - `src/lib.rs`
   - `src/extension.rs`
   - `src/config.rs` (if extension needs configuration)
   - `src/error.rs`

2. Verify directory structure based on features:
   - `src/models/` (if domain types)
   - `src/repository/` (if database)
   - `src/services/` (if business logic)
   - `src/api/` (if HTTP endpoints)
   - `src/jobs/` (if background tasks)
   - `schema/` (if database tables)

3. Read all `.rs` files in `{extension_path}/src/`

4. Read `Cargo.toml`

5. Execute each checklist item from the extension-checklist playbook

6. For each violation, record: `file:line` + violation type

7. Generate `status.md` using output template

---

## Validation Commands

```bash
# Structure checks
test -f {extension_path}/Cargo.toml
test -f {extension_path}/src/lib.rs
test -f {extension_path}/src/extension.rs
test -f {extension_path}/src/error.rs

# Config checks (if config.rs exists)
test -f {extension_path}/src/config.rs && {
    # Must have Raw and Validated types
    grep -q "struct.*Raw" {extension_path}/src/config.rs
    grep -q "struct.*Validated" {extension_path}/src/config.rs
    # Must implement ExtensionConfig
    grep -q "impl ExtensionConfig" {extension_path}/src/config.rs
    # Must have register_config_extension!
    grep -q "register_config_extension!" {extension_path}/src/
}

# Boundary checks (should be empty or only show allowed deps)
grep -E "systemprompt-core-(api|scheduler)" {extension_path}/Cargo.toml

# Repository pattern (no runtime SQL)
grep -rn "sqlx::query[^!]" {extension_path}/src/

# SQL in services (forbidden)
grep -rn "sqlx::" {extension_path}/src/services/

# Config anti-patterns (should be empty)
grep -rn "std::env::var.*CONFIG" {extension_path}/src/  # No env var config loading
grep -rn "unwrap_or_else.*default" {extension_path}/src/config.rs  # No silent fallbacks

# Code quality
cargo clippy -p {extension_name} -- -D warnings
cargo fmt -p {extension_name} -- --check
```

---

## Output

Generate `{extension_path}/status.md` using the status-template playbook.

**Verdict:** COMPLIANT if zero violations. NON-COMPLIANT otherwise.

---

## Status Template

```markdown
# {crate_name} Compliance

**Layer:** {Shared | Infrastructure | Domain | Application | Entry}
**Reviewed:** {YYYY-MM-DD}
**Verdict:** {COMPLIANT | NON-COMPLIANT}

---

## Checklist

| Category | Status |
|----------|--------|
| Boundary Rules | pass/fail |
| Required Structure | pass/fail |
| Code Quality | pass/fail |

---

## Violations

| File:Line | Violation | Category |
|-----------|-----------|----------|
| `src/foo.rs:42` | `unwrap()` usage | Code Quality |
| `src/bar.rs:15` | Direct SQL in service | Repository Pattern |

{Or: "None"}

---

## Commands Run

cargo clippy -p {crate_name} -- -D warnings  # {PASS/FAIL}
cargo fmt -p {crate_name} -- --check          # {PASS/FAIL}

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
| Generate status | Create `status.md` in extension root |

-> See [Extension Checklist](build_extension-checklist) for full checklist.
-> See [Rust Standards](build_rust-standards) for code quality rules.
-> See [Architecture](build_architecture) for dependency rules.
