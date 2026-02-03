---
title: "Extension Dependencies"
description: "Declare and manage dependencies between extensions."
author: "SystemPrompt Team"
slug: "extensions/lifecycle/dependencies"
keywords: "dependencies, ordering, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Extension Dependencies

Extensions can declare dependencies on other extensions, ensuring correct loading order and availability.

## Declaring Dependencies

Use the `dependencies()` method:

```rust
impl Extension for MyExtension {
    fn dependencies(&self) -> Vec<&'static str> {
        vec!["users", "oauth"]
    }

    // ...
}
```

The runtime validates that all declared dependencies exist before proceeding.

## Loading Order

Extensions load in priority order, but dependencies are validated regardless of priority:

1. Extensions sorted by `priority()` (lower first)
2. For each extension, verify all dependencies are registered
3. If dependency missing, fail with `LoaderError::MissingDependency`

## Priority vs Dependencies

**Priority** controls execution order within the same tier:

```rust
fn priority(&self) -> u32 {
    50  // Lower = loads first
}
```

**Dependencies** ensure availability:

```rust
fn dependencies(&self) -> Vec<&'static str> {
    vec!["users"]  // Must exist, regardless of priority
}
```

Best practice: Set priority lower than dependencies:

```rust
// users extension: priority 10
// oauth extension: priority 20, depends on users
// my extension: priority 50, depends on users and oauth
```

## Cycle Detection

The runtime detects circular dependencies:

```rust
// This will fail:
// a depends on b
// b depends on c
// c depends on a
```

Error:
```
LoaderError::CircularDependency {
    chain: "a -> b -> c -> a"
}
```

## Accessing Dependencies

Access other extensions via `ExtensionContext`:

```rust
fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
    // Check if dependency exists
    if !ctx.has_extension("users") {
        return None;
    }

    // Get extension
    let users_ext = ctx.get_extension("users")?;

    // Use extension capabilities...
}
```

## Typed Dependencies

For compile-time dependency checking, use the typed extension system:

```rust
use systemprompt::extension::prelude::*;

// Define extension type
pub struct MyExtension;

impl ExtensionType for MyExtension {
    const ID: &'static str = "my-extension";
    const NAME: &'static str = "My Extension";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
}

// Declare typed dependencies
impl Dependencies for MyExtension {
    type Deps = (UsersExtension, OAuthExtension);
}
```

The `ExtensionBuilder` validates these at compile time:

```rust
let registry = ExtensionBuilder::new()
    .extension(UsersExtension::default())
    .extension(OAuthExtension::default())
    .extension(MyExtension::default())  // Compiles only if deps registered first
    .build()?;
```

## DependencyList Trait

```rust
pub trait DependencyList: 'static {
    fn validate(registry: &TypedExtensionRegistry) -> Result<(), MissingDependency>;
    fn dependency_ids() -> Vec<&'static str>;
}
```

Implemented for tuples:

```rust
impl<A: ExtensionType, B: ExtensionType> DependencyList for (A, B) {
    fn dependency_ids() -> Vec<&'static str> {
        vec![A::ID, B::ID]
    }
}
```

## No Dependencies

For extensions without dependencies:

```rust
impl Dependencies for MyExtension {
    type Deps = ();
}

// Or use the marker trait
impl NoDependencies for MyExtension {}
```

## Common Patterns

### Core Dependencies

Most extensions depend on infrastructure:

```rust
fn dependencies(&self) -> Vec<&'static str> {
    vec!["database", "users"]
}
```

### Feature Dependencies

Feature extensions depend on domain:

```rust
fn dependencies(&self) -> Vec<&'static str> {
    vec!["users", "content", "analytics"]
}
```

### Optional Dependencies

Check at runtime for optional features:

```rust
fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
    let has_analytics = ctx.has_extension("analytics");

    // Conditionally add analytics routes
    if has_analytics {
        router = router.nest("/analytics", analytics_routes());
    }
}
```