---
title: "Getting Started with Extensions"
description: "Create your first SystemPrompt extension."
author: "SystemPrompt"
slug: "build-01-getting-started"
keywords: "getting-started, first-extension, tutorial"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Getting Started with Extensions

Create your first library extension. Reference: `extensions/web/` for working example.

> **Help**: `{ "command": "core playbooks show build_getting-started" }`

---

## Prerequisites

- Rust toolchain installed
- SystemPrompt template cloned
- Basic Rust knowledge

---

## Structure

```
extensions/hello/
├── Cargo.toml
└── src/
    └── lib.rs
```

---

## Step 1: Create Extension Crate

```bash
mkdir -p extensions/hello
cd extensions/hello
```

Create `Cargo.toml`:

```toml
[package]
name = "hello-extension"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib"]

[dependencies]
systemprompt = { workspace = true }
```

---

## Step 2: Implement Extension

Create `src/lib.rs`. See `extensions/web/src/extension.rs:15-45` for reference.

```rust
use systemprompt::extension::prelude::*;

#[derive(Debug, Default)]
pub struct HelloExtension;

impl Extension for HelloExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "hello",
            name: "Hello Extension",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

register_extension!(HelloExtension);

pub const PREFIX: &str = "hello";
```

---

## Step 3: Add to Workspace

In root `Cargo.toml`:

```toml
[workspace]
members = [
    "extensions/hello",
]

[dependencies]
hello-extension = { path = "extensions/hello" }
```

---

## Step 4: Link Extension

In template `src/lib.rs`. See `extensions/web/src/lib.rs:1-10` for reference.

```rust
pub use hello_extension as hello;

pub fn __force_extension_link() {
    let _ = core::hint::black_box(&hello::PREFIX);
}
```

---

## Step 5: Build and Verify

```bash
cargo build
cargo run -- extensions list
```

Output should include:

```
- hello (v0.1.0, priority: 100)
```

---

## Checklist

- [ ] `Cargo.toml` with `crate-type = ["rlib"]`
- [ ] `Extension` trait implemented
- [ ] `register_extension!` macro called
- [ ] `PREFIX` constant exported
- [ ] Added to workspace members
- [ ] Linked in `__force_extension_link()`

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Extension not showing | Verify `register_extension!` called |
| Missing in list | Check workspace members in root `Cargo.toml` |
| Linker strips extension | Add to `__force_extension_link()` |
| Compilation errors | Run `cargo clean && cargo build` |

---

## Quick Reference

| Task | Command |
|------|---------|
| Create crate | `mkdir -p extensions/hello` |
| Build | `cargo build --workspace` |
| Verify | `cargo run -- extensions list` |
| Clean build | `cargo clean && cargo build` |

---

## Related

-> See [Create Library Extension](../02-library-extensions/create-extension.md) for full extension
-> See [Rust Standards](../06-standards/rust-standards.md) for code style
-> See [Add Database Schema](../02-library-extensions/add-schema.md) for next step