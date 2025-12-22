# Extension Review

> Review this extension as though you were Steve Klabnik implementing world-class idiomatic Rust.

---

## Input

- **Folder:** `{extension_path}`
- **Checklist:** `/instructions/build/extension-checklist.md`
- **Standards:** `/instructions/build/rust-standards.md`

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

5. Execute each checklist item from `/instructions/build/extension-checklist.md`

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

Generate `{extension_path}/status.md` using `/instructions/review/status-template.md`.

**Verdict:** COMPLIANT if zero violations. NON-COMPLIANT otherwise.
