---
title: "Extension Registration"
description: "How extensions register with the runtime using the inventory crate and register_extension! macro."
author: "SystemPrompt Team"
slug: "extensions/lifecycle/registration"
keywords: "registration, inventory, macro, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Extension Registration

Extensions register at compile time using the `register_extension!` macro, which leverages the `inventory` crate for zero-cost plugin discovery.

## The register_extension! Macro

After implementing the Extension trait, register your extension:

```rust
use systemprompt::extension::prelude::*;

#[derive(Debug, Default)]
pub struct MyExtension;

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "my-extension",
            name: "My Extension",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    // ... other methods
}

register_extension!(MyExtension);
```

## Macro Variants

### Type Registration

For types that implement `Default`:

```rust
register_extension!(MyExtension);
```

Expands to:

```rust
inventory::submit! {
    ExtensionRegistration {
        factory: || Arc::new(MyExtension::default()) as Arc<dyn Extension>,
    }
}
```

### Expression Registration

For types requiring configuration:

```rust
register_extension!(MyExtension::new(config));
```

Expands to:

```rust
inventory::submit! {
    ExtensionRegistration {
        factory: || Arc::new(MyExtension::new(config)) as Arc<dyn Extension>,
    }
}
```

## ExtensionRegistration

The registration struct:

```rust
#[derive(Debug, Clone, Copy)]
pub struct ExtensionRegistration {
    pub factory: fn() -> Arc<dyn Extension>,
}

inventory::collect!(ExtensionRegistration);
```

## Preventing Linker Stripping

Rust's linker removes unused code. Extensions in separate crates may be stripped if not directly referenced. The template's `src/lib.rs` prevents this:

```rust
pub use my_extension_crate as my_extension;
pub use other_extension_crate as other_extension;

pub fn __force_extension_link() {
    let _ = core::hint::black_box(&my_extension::MyExtension::PREFIX);
    let _ = core::hint::black_box(&other_extension::OtherExtension::PREFIX);
}
```

The `black_box` function prevents the compiler from optimizing away the reference.

## Adding a New Extension

1. Create extension crate:

```toml
# extensions/my-extension/Cargo.toml
[package]
name = "my-extension"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib"]

[dependencies]
systemprompt = { workspace = true }
```

2. Implement and register:

```rust
// extensions/my-extension/src/lib.rs
use systemprompt::extension::prelude::*;

#[derive(Debug, Default)]
pub struct MyExtension;

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "my-extension",
            name: "My Extension",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

register_extension!(MyExtension);

pub const PREFIX: &str = "my-extension";
```

3. Add to workspace:

```toml
# Cargo.toml (root)
[workspace]
members = [
    "extensions/my-extension",
    # ...
]
```

4. Link in template:

```rust
// src/lib.rs
pub use my_extension;

pub fn __force_extension_link() {
    // Add reference
    let _ = core::hint::black_box(&my_extension::PREFIX);
}
```

5. Add dependency:

```toml
# Cargo.toml (root)
[dependencies]
my-extension = { path = "extensions/my-extension" }
```

## The Inventory Crate

The `inventory` crate provides compile-time plugin registration:

- **Zero runtime cost** - Registration happens at compile time
- **No global mutable state** - Type-safe collection
- **Automatic discovery** - No manual registration list
- **Linker-based** - Uses linker sections for collection

## Verification

Check registered extensions:

```bash
systemprompt extensions list
```

Output:

```
Registered Extensions:
  - users (v0.1.0, priority: 10)
  - oauth (v0.1.0, priority: 20)
  - my-extension (v0.1.0, priority: 100)
```