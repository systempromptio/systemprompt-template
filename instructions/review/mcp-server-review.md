# MCP Server Review

> Review this MCP server as though you were Steve Klabnik implementing world-class idiomatic Rust.

---

## Input

- **Folder:** `{server_path}`
- **Checklist:** `/instructions/build/mcp-server-checklist.md`
- **Standards:** `/instructions/build/rust-standards.md`

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

6. Execute each checklist item from `/instructions/build/mcp-server-checklist.md`

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

Generate `{server_path}/status.md` using `/instructions/review/status-template.md`.

**Verdict:** COMPLIANT if zero violations. NON-COMPLIANT otherwise.
