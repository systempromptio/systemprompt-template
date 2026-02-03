---
title: "Build Playbook"
description: "Build core and MCP extensions."
author: "SystemPrompt"
slug: "cli-build"
keywords: "build, compile, cargo, rust"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Build Playbook

Build core and MCP extensions.

---

## Build Core

```json
{ "command": "build core" }
{ "command": "build core --release" }
```

Terminal-only (requires shell):

```bash
just build
```

---

## Build MCP Extensions

### Build All MCP Servers

```json
{ "command": "build mcp" }
{ "command": "build mcp --release" }
```

### Build Specific MCP Server (Terminal Only)

```bash
cargo build -p <crate-name>
cargo build -p <crate-name> --release
```

---

## Troubleshooting

**Missing dependencies**: Run `rustup update` and `cargo check` to verify toolchain and dependencies.

**Compilation error**: Run `cargo clippy -p <crate-name> -- -D warnings` for detailed errors, and `cargo fmt -p <crate-name> -- --check` for formatting issues.

---

## Quick Reference

| Task | Command |
|------|---------|
| Build all | `just build` |
| Build core | `systemprompt build core` |
| Build MCP | `systemprompt build mcp` |
| Release build | `just build --release` |
| Build single crate | `cargo build -p <crate-name>` |
| Release single crate | `cargo build -p <crate-name> --release` |