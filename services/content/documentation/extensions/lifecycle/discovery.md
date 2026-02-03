---
title: "Extension Discovery"
description: "How the runtime discovers and validates extensions at startup."
author: "SystemPrompt Team"
slug: "extensions/lifecycle/discovery"
keywords: "discovery, registry, validation, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Extension Discovery

At startup, `ExtensionRegistry::discover()` collects all registered extensions, validates them, and sorts them for loading.

## Discovery Process

```rust
let registry = ExtensionRegistry::discover();
```

This performs:

1. **Collection** - Iterate `inventory::iter::<ExtensionRegistration>`
2. **Instantiation** - Call each factory to create extension instances
3. **Registration** - Add to internal maps
4. **Sorting** - Order by priority
5. **Validation** - Check dependencies and paths

## ExtensionRegistry

```rust
pub struct ExtensionRegistry {
    extensions: HashMap<String, Arc<dyn Extension>>,
    sorted_extensions: Vec<Arc<dyn Extension>>,
}

impl ExtensionRegistry {
    pub fn discover() -> Self;
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Extension>>;
    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn Extension>>;
    pub fn validate(&self) -> Result<(), LoaderError>;
}
```

## Sorting

Extensions are sorted by priority (lower values first):

```rust
fn sort_by_priority(&mut self) {
    self.sorted_extensions.sort_by_key(|ext| ext.priority());
}
```

Typical priorities:
- `1-10` - Core infrastructure (database, users)
- `10-50` - Domain extensions
- `50-100` - Feature extensions
- `100+` - Optional/plugin extensions

## Validation

### Dependency Validation

```rust
fn validate_dependencies(&self) -> Result<(), LoaderError> {
    for ext in self.iter() {
        for dep in ext.dependencies() {
            if !self.extensions.contains_key(dep) {
                return Err(LoaderError::MissingDependency {
                    extension: ext.id().to_string(),
                    dependency: dep.to_string(),
                });
            }
        }
    }
    Ok(())
}
```

### Cycle Detection

DFS-based circular dependency detection:

```rust
fn detect_cycles(&self) -> Result<(), LoaderError> {
    // DFS to find back edges
    // Returns Err(CircularDependency) if found
}
```

### Path Validation

API paths must not collide with reserved paths:

```rust
pub const RESERVED_PATHS: &[&str] = &[
    "/api/v1/oauth",
    "/api/v1/users",
    "/api/v1/agents",
    // ...
];

fn validate_api_paths(&self) -> Result<(), LoaderError> {
    for ext in self.iter() {
        if let Some(config) = ext.router_config() {
            if RESERVED_PATHS.contains(&config.base_path) {
                return Err(LoaderError::ReservedPathCollision {
                    extension: ext.id().to_string(),
                    path: config.base_path.to_string(),
                });
            }
        }
    }
    Ok(())
}
```

## Filtering

Get extensions with specific capabilities:

```rust
// Extensions with schemas
let schema_exts: Vec<_> = registry.iter()
    .filter(|ext| ext.has_schemas())
    .collect();

// Extensions with jobs
let job_exts: Vec<_> = registry.iter()
    .filter(|ext| ext.has_jobs())
    .collect();

// Extensions with routers
let api_exts: Vec<_> = registry.iter()
    .filter(|ext| ext.has_router(&ctx))
    .collect();
```

## Runtime Injection

Extensions can be injected programmatically:

```rust
use systemprompt::extension::runtime_config::{set_injected_extensions, InjectedExtensions};

let injected = InjectedExtensions {
    extensions: vec![Arc::new(TestExtension)],
};

set_injected_extensions(injected)?;
```

Injected extensions are merged during discovery.

## Debugging

### List Extensions

```bash
systemprompt extensions list
```

### Show Dependencies

```bash
systemprompt extensions deps
```

### Validate

```bash
systemprompt extensions validate
```

### Common Errors

**MissingDependency**:
```
Extension 'my-ext' requires dependency 'users' which is not registered
```

Fix: Ensure dependency is linked and registered.

**CircularDependency**:
```
Circular dependency detected: a -> b -> c -> a
```

Fix: Break the cycle by restructuring dependencies.

**ReservedPathCollision**:
```
Extension 'my-ext' uses reserved API path '/api/v1/users'
```

Fix: Use a different base path.