---
title: "Build Playbook"
description: "Build core, web, and MCP extensions."
keywords:
  - build
  - compile
  - cargo
  - rust
---

# Build Playbook

Build core, web, and MCP extensions.

> **Help**: `{ "command": "build" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Build Core

```bash
// MCP: systemprompt
systemprompt build core
systemprompt build core --release
just build
```

---

## Build Web

```bash
// MCP: systemprompt
systemprompt build web
systemprompt build web --release
```

---

## Build MCP Extensions

### Build All MCP Servers

```bash
// MCP: systemprompt
systemprompt build mcp
systemprompt build mcp --release
```

### Build Specific MCP Server

```bash
cargo build -p <crate-name>
cargo build -p <crate-name> --release
```

---

## Troubleshooting

**Missing dependencies**: Run `rustup update` and `cargo check` to verify toolchain and dependencies.

**Compilation error**: Run `cargo clippy -p <crate-name> -- -D warnings` for detailed errors, and `cargo fmt -p <crate-name> -- --check` for formatting issues.

**Web build failed**: Check `node --version` and run `npm install` to reinstall dependencies.

-> See [Services Playbook](services.md) for restarting services after rebuilding.

---

## Quick Reference

| Task | Command |
|------|---------|
| Build all | `just build` |
| Build core | `systemprompt build core` |
| Build web | `systemprompt build web` |
| Build MCP | `systemprompt build mcp` |
| Release build | `just build --release` |
| Build single crate | `cargo build -p <crate-name>` |
| Release single crate | `cargo build -p <crate-name> --release` |
