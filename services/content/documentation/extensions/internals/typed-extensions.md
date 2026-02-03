---
title: "Typed Extensions"
description: "Compile-time type-safe extension traits for schema, API, job, provider, and config extensions."
author: "SystemPrompt Team"
slug: "extensions/internals/typed-extensions"
keywords: "typed, compile-time, safety, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Typed Extensions

Typed extension traits provide compile-time type safety for extension capabilities. They enforce patterns at the type level rather than runtime.

## ExtensionType Trait

The foundation for typed extensions:

```rust
pub trait ExtensionType: Default + Send + Sync + 'static {
    const ID: &'static str;
    const NAME: &'static str;
    const VERSION: &'static str;
    const PRIORITY: u32 = 100;

    fn type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}
```

Implementation:

```rust
#[derive(Debug, Default)]
pub struct MyExtension;

impl ExtensionType for MyExtension {
    const ID: &'static str = "my-extension";
    const NAME: &'static str = "My Extension";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const PRIORITY: u32 = 50;
}
```

## ExtensionMeta Trait

Runtime access to extension metadata:

```rust
pub trait ExtensionMeta: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn priority(&self) -> u32;
}

impl<T: ExtensionType> ExtensionMeta for T {
    fn id(&self) -> &'static str { T::ID }
    fn name(&self) -> &'static str { T::NAME }
    fn version(&self) -> &'static str { T::VERSION }
    fn priority(&self) -> u32 { T::PRIORITY }
}
```

## SchemaExtensionTyped

Type-safe schema definitions:

```rust
pub trait SchemaExtensionTyped: ExtensionMeta {
    fn schemas(&self) -> Vec<SchemaDefinitionTyped>;

    fn migration_weight(&self) -> u32 {
        100
    }
}
```

Implementation:

```rust
impl SchemaExtensionTyped for MyExtension {
    fn schemas(&self) -> Vec<SchemaDefinitionTyped> {
        vec![
            SchemaDefinitionTyped::inline("users", include_str!("../schema/users.sql")),
        ]
    }

    fn migration_weight(&self) -> u32 {
        10
    }
}
```

## ApiExtensionTyped

Type-safe API configuration:

```rust
pub trait ApiExtensionTyped: ExtensionMeta {
    fn base_path(&self) -> &'static str;

    fn requires_auth(&self) -> bool {
        true
    }
}

#[cfg(feature = "web")]
pub trait ApiExtensionTypedDyn: ApiExtensionTyped {
    fn build_router(&self) -> Router;
}
```

Implementation:

```rust
impl ApiExtensionTyped for MyExtension {
    fn base_path(&self) -> &'static str {
        "/api/v1/my-extension"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

impl ApiExtensionTypedDyn for MyExtension {
    fn build_router(&self) -> Router {
        Router::new()
            .route("/items", get(list_items))
            .route("/items/:id", get(get_item))
    }
}
```

## JobExtensionTyped

Type-safe job registration:

```rust
pub trait JobExtensionTyped: ExtensionMeta {
    fn jobs(&self) -> Vec<Arc<dyn Job>>;
}
```

Implementation:

```rust
impl JobExtensionTyped for MyExtension {
    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![
            Arc::new(CleanupJob),
            Arc::new(SyncJob),
        ]
    }
}
```

## ProviderExtensionTyped

Type-safe provider registration:

```rust
pub trait ProviderExtensionTyped: ExtensionMeta {
    fn llm_providers(&self) -> Vec<Arc<dyn LlmProvider>> {
        vec![]
    }

    fn tool_providers(&self) -> Vec<Arc<dyn ToolProvider>> {
        vec![]
    }
}
```

## ConfigExtensionTyped

Type-safe configuration:

```rust
pub trait ConfigExtensionTyped: ExtensionMeta {
    fn config_prefix(&self) -> &'static str;

    fn validate_config(&self, config: &JsonValue) -> Result<(), ConfigError> {
        Ok(())
    }

    fn config_schema(&self) -> Option<JsonValue> {
        None
    }
}
```

## Type Erasure

Typed extensions can be erased to dynamic traits:

```rust
pub trait AnyExtension: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    fn priority(&self) -> u32;
    fn as_schema(&self) -> Option<&dyn SchemaExtensionTyped>;
    fn as_api(&self) -> Option<&dyn ApiExtensionTypedDyn>;
    fn as_config(&self) -> Option<&dyn ConfigExtensionTyped>;
    fn as_job(&self) -> Option<&dyn JobExtensionTyped>;
    fn as_provider(&self) -> Option<&dyn ProviderExtensionTyped>;
    fn as_any(&self) -> &dyn Any;
}
```

Wrapper implementations:

```rust
pub struct SchemaExtensionWrapper<T> {
    inner: T,
}

impl<T: ExtensionType + SchemaExtensionTyped> AnyExtension for SchemaExtensionWrapper<T> {
    fn as_schema(&self) -> Option<&dyn SchemaExtensionTyped> {
        Some(&self.inner)
    }
    // ...
}
```

## Benefits

1. **Compile-time validation** - Invalid configurations caught at build
2. **IDE support** - Better autocomplete and error messages
3. **Refactoring safety** - Rename/change detection
4. **Documentation** - Types document intent
5. **Consistency** - Enforced patterns

## When to Use

Use typed extensions when:
- Building core infrastructure
- Need compile-time dependency validation
- Want maximum type safety

Use dynamic Extension trait when:
- Simpler implementation needed
- Runtime flexibility required
- Prototyping