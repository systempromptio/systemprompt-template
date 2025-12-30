# MCP Extension Review Checklist

Deterministic quality gates for code review.

---

## Pre-Merge Verification

### 0. Dependency Versions

| Dependency | Check | Command |
|------------|-------|---------|
| systemprompt-core | Latest commit on main | `cd ../systemprompt-core && git log -1 --oneline` |
| rmcp | Latest crates.io version | `cargo search rmcp --limit 1` |
| All Cargo.lock | Up to date | `cargo update --dry-run` |

```bash
cargo update --dry-run 2>&1 | grep -E "(Updating|would update)" || echo "✅ All dependencies up to date"
```

### 1. Compile-Time Checks

```bash
cargo clippy -p systemprompt-mcp-infrastructure -- -D warnings
cargo fmt -p systemprompt-mcp-infrastructure -- --check
cargo test -p systemprompt-mcp-infrastructure
```

### 2. Source File Limits

| Metric | Limit | Check Command |
|--------|-------|---------------|
| File length | ≤300 lines | `wc -l src/**/*.rs` |
| Function length | ≤75 lines | Manual review |
| Parameters | ≤5 | Manual review |
| Cognitive complexity | ≤15 | Manual review |

### 3. Forbidden Constructs

| Pattern | Allowed | Check Command |
|---------|---------|---------------|
| `unsafe` | Never | `grep -r "unsafe" src/` |
| `unwrap()` | Never | `grep -r "\.unwrap()" src/` |
| `panic!()` | Never | `grep -r "panic!" src/` |
| `todo!()` | Never | `grep -r "todo!" src/` |
| `//` comments | Never | `grep -r "^\s*//" src/` |
| `///` doc comments | Never | `grep -r "^\s*///" src/` |
| `#[cfg(test)]` in src | Never | `grep -r "#\[cfg(test)\]" src/` |
| `env::var()` direct | Never | `grep -r "env::var" src/` |
| `dotenvy::dotenv()` | Never | `grep -r "dotenvy" src/` |

**Permitted patterns** (see [tools.md](../implementation/tools.md)):

| Pattern | Context | Example |
|---------|---------|---------|
| `unwrap_or_default()` | Optional tool arguments | `request.arguments.unwrap_or_default()` |
| `unwrap_or_else()` | Computed defaults | `tag.unwrap_or_else(\|\| generate_tag())` |
| `unwrap_or(CONSTANT)` | Named constants only | `value.unwrap_or(DEFAULT_ENV)` |

**Still forbidden** (fuzzy defaults):

| Pattern | Why |
|---------|-----|
| `unwrap_or("hardcoded")` | Magic strings hide problems |
| `unwrap_or(0)` / `unwrap_or(false)` | Silent failure masking |
| `unwrap_or_default()` for non-collection types | Use explicit error handling |

### 3.1 Configuration Rules

| Rule | Description |
|------|-------------|
| Profiles REQUIRED | `ProfileBootstrap::init()` then `Config::init()` |
| No env vars | Never use `std::env::var()` directly |
| No .env files | Never use `dotenvy::dotenv()` |
| Fail explicitly | Missing config must error, never use defaults |
| Named constants | Use `const` for default values, never inline literals |

See [profiles.md](../config/profiles.md) for correct bootstrap pattern.

### 4. Required Patterns

| Pattern | Present |
|---------|---------|
| Typed identifiers (`McpServerId`, `McpExecutionId`) | ☐ |
| `#[must_use]` on value-returning functions | ☐ |
| `Arc<T>` for shared state across handlers | ☐ |
| `#[derive(Clone)]` on server structs | ☐ |
| `thiserror` for domain errors | ☐ |
| `tracing` with structured fields | ☐ |

### 5. Tool Implementation Requirements

| Requirement | Present |
|-------------|---------|
| Input validation via `parse_tool_input<T>()` | ☐ |
| Progress reporting via `report_progress()` | ☐ |
| Skill loading (if AI-powered) | ☐ |
| Error conversion to `McpError` | ☐ |
| Execution record start | ☐ |
| Execution record complete | ☐ |
| Timeout wrapper for async operations | ☐ |

---

## Status Output Template

Generate `instructions/status.md` after review:

```markdown
# systemprompt-mcp-infrastructure

| Field | Value |
|-------|-------|
| Layer | Entry |
| Reviewed | YYYY-MM-DD |
| Verdict | COMPLIANT / NON-COMPLIANT |

## Checklist

| Category | Status |
|----------|--------|
| Dependency Versions | ✅ / ❌ |
| Compile-Time Checks | ✅ / ❌ |
| Test Suite | ✅ / ❌ |
| Source File Limits | ✅ / ❌ |
| Forbidden Constructs | ✅ / ❌ |
| Required Patterns | ✅ / ❌ |
| Tool Requirements | ✅ / ❌ |

## Violations

| File | Line | Violation | Category |
|------|------|-----------|----------|
| src/foo.rs | 42 | `unwrap()` usage | Forbidden Construct |

Or: None

## Commands

| Command | Result |
|---------|--------|
| `cargo clippy -- -D warnings` | PASS / FAIL |
| `cargo fmt -- --check` | PASS / FAIL |
| `cargo test` | PASS (N tests) / FAIL |

## Actions Required

1. {Action item}
2. {Action item}

Or: None
```

---

## Quick Check Script

```bash
#!/bin/bash
set -e

echo "=== Dependency Versions ==="
cargo update --dry-run 2>&1 | grep -E "(Updating|would update)" && echo "❌ Dependencies need update" || echo "✅ All dependencies up to date"

echo "=== Forbidden Constructs ==="
! grep -rn "\.unwrap()" src/ && echo "✅ No unwrap()"
! grep -rn "unsafe" src/ && echo "✅ No unsafe"
! grep -rn "panic!" src/ && echo "✅ No panic!"
! grep -rn "todo!" src/ && echo "✅ No todo!"
! grep -rn "^\s*//" src/ && echo "✅ No inline comments"
! grep -rn "^\s*///" src/ && echo "✅ No doc comments"
! grep -rn "#\[cfg(test)\]" src/ && echo "✅ No inline tests"
! grep -rn "env::var" src/ && echo "✅ No direct env vars"
! grep -rn "dotenvy" src/ && echo "✅ No .env loading"

echo "=== File Lengths ==="
find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 300 { print "❌ " $2 ": " $1 " lines" }'

echo "=== Compile Checks ==="
cargo clippy -p systemprompt-mcp-infrastructure -- -D warnings
cargo fmt -p systemprompt-mcp-infrastructure -- --check

echo "=== Test Suite ==="
cargo test -p systemprompt-mcp-infrastructure

echo "=== All checks passed ==="
```

---

## See Also

- [prompt.md](./prompt.md) - Review instructions for AI
- [Core Rust Standards](../../../../systemprompt-core/instructions/rust/rust.md)
- [Tool Implementation](../implementation/tools.md)
