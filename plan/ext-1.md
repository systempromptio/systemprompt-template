# Phase 1: Extension Framework Core

**Objective**: Build the type-safe extension infrastructure in systemprompt-core that enables downstream projects to add custom functionality.

---

## 1. Overview

This phase creates the foundational extension system with:
- Type-safe dependency resolution at compile time
- Explicit builder pattern (no magic inventory)
- Capability traits for principle of least privilege
- Unified extension storage with runtime introspection

---

## 2. File Structure

Create new module structure in `crates/shared/extension/`:

```
crates/shared/extension/src/
├── lib.rs              # Main exports, feature flags
├── types.rs            # ExtensionType, Dependencies, DependencyList
├── hlist.rs            # Type-level HList and Subset trait
├── capabilities.rs     # HasDatabase, HasConfig, HasExtension<E>
├── builder.rs          # ExtensionBuilder<Registered>
├── registry.rs         # ExtensionRegistry (runtime storage)
├── any.rs              # AnyExtension trait for type erasure
├── traits/
│   ├── mod.rs
│   ├── schema.rs       # SchemaExtension
│   ├── api.rs          # ApiExtension
│   ├── config.rs       # ConfigExtension
│   ├── job.rs          # JobExtension
│   └── provider.rs     # ProviderExtension
├── schema.rs           # SchemaDefinition, SchemaSource
├── error.rs            # ExtensionError types
└── tests/
    ├── mod.rs
    ├── builder_tests.rs
    ├── dependency_tests.rs
    └── capability_tests.rs
```

---

## 3. Core Trait Definitions

### 3.1 ExtensionType Trait

**File**: `crates/shared/extension/src/types.rs`

```rust
//! Type-safe extension type definitions.

use std::any::TypeId;

/// Marker trait for extension types.
///
/// Unlike the old `Extension` trait which used runtime strings,
/// this uses const generics for compile-time verification.
pub trait ExtensionType: Default + Send + Sync + 'static {
    /// Unique identifier (kebab-case)
    const ID: &'static str;

    /// Human-readable name
    const NAME: &'static str;

    /// SemVer version
    const VERSION: &'static str;

    /// Priority for initialization (lower = earlier)
    const PRIORITY: u32 = 100;

    /// Get TypeId for runtime type checking
    fn type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Type-level dependency declaration.
pub trait Dependencies: ExtensionType {
    /// HList of extension types this depends on.
    type Deps: DependencyList;
}

/// Default implementation for extensions with no dependencies.
impl<T: ExtensionType> Dependencies for T
where
    T: NoDependencies,
{
    type Deps = ();
}

/// Marker trait for extensions with no dependencies.
pub trait NoDependencies {}

/// Trait for validating dependency lists at runtime.
pub trait DependencyList: 'static {
    /// Validate all dependencies are present in registry.
    fn validate(registry: &ExtensionRegistry) -> Result<(), MissingDependency>;

    /// Get list of dependency IDs for debugging.
    fn dependency_ids() -> Vec<&'static str>;
}

// Base case: empty list
impl DependencyList for () {
    fn validate(_: &ExtensionRegistry) -> Result<(), MissingDependency> {
        Ok(())
    }

    fn dependency_ids() -> Vec<&'static str> {
        vec![]
    }
}

// Recursive case: (Head, Tail)
impl<H: ExtensionType, T: DependencyList> DependencyList for (H, T) {
    fn validate(registry: &ExtensionRegistry) -> Result<(), MissingDependency> {
        if !registry.has_type::<H>() {
            return Err(MissingDependency {
                extension_id: H::ID,
                extension_name: H::NAME,
            });
        }
        T::validate(registry)
    }

    fn dependency_ids() -> Vec<&'static str> {
        let mut ids = vec![H::ID];
        ids.extend(T::dependency_ids());
        ids
    }
}
```

### 3.2 HList Type-Level Operations

**File**: `crates/shared/extension/src/hlist.rs`

```rust
//! Type-level heterogeneous list operations.

use std::marker::PhantomData;

/// Marker trait for type-level lists.
pub trait TypeList: 'static {
    /// Check if type T is in this list.
    fn contains<T: 'static>() -> bool;
}

impl TypeList for () {
    fn contains<T: 'static>() -> bool {
        false
    }
}

impl<H: 'static, T: TypeList> TypeList for (H, T) {
    fn contains<X: 'static>() -> bool {
        std::any::TypeId::of::<H>() == std::any::TypeId::of::<X>()
            || T::contains::<X>()
    }
}

/// Compile-time check: is A a subset of B?
pub trait Subset<B: TypeList>: TypeList {}

// Empty set is subset of everything
impl<B: TypeList> Subset<B> for () {}

// (H, T) ⊆ B if H ∈ B and T ⊆ B
impl<H, T, B> Subset<B> for (H, T)
where
    H: 'static,
    T: TypeList + Subset<B>,
    B: TypeList + Contains<H>,
{
}

/// Type-level membership check.
pub trait Contains<T: 'static>: TypeList {}

impl<H: 'static, T: TypeList> Contains<H> for (H, T) {}

impl<H: 'static, X: 'static, T: TypeList + Contains<X>> Contains<X> for (H, T)
where
    // Ensure we only match when H != X
    (): AssertNe<H, X>,
{
}

// Helper for asserting types are not equal
pub trait AssertNe<A, B> {}
impl<A, B> AssertNe<A, B> for ()
where
    // This is always satisfied but helps with inference
{}
```

### 3.3 Capability Traits

**File**: `crates/shared/extension/src/capabilities.rs`

```rust
//! Capability traits for dependency injection.
//!
//! Instead of a god-object ExtensionContext, extensions declare
//! exactly what capabilities they need via trait bounds.

use std::sync::Arc;
use systemprompt_traits::{ConfigProvider, DatabaseHandle};

/// Capability: access to configuration.
pub trait HasConfig: Send + Sync {
    type Config: ConfigProvider;

    fn config(&self) -> &Self::Config;
}

/// Capability: access to database.
pub trait HasDatabase: Send + Sync {
    type Database: DatabaseHandle;

    fn database(&self) -> &Self::Database;
}

/// Capability: access to a specific extension.
pub trait HasExtension<E: ExtensionType>: Send + Sync {
    fn extension(&self) -> &E;
}

/// Capability: access to HTTP client (for extensions that call external APIs).
pub trait HasHttpClient: Send + Sync {
    fn http_client(&self) -> &reqwest::Client;
}

/// Capability: access to event bus for publishing events.
pub trait HasEventBus: Send + Sync {
    fn event_bus(&self) -> &dyn EventPublisher;
}

/// Combined capabilities for common use cases.
pub trait FullContext: HasConfig + HasDatabase + HasEventBus {}
impl<T: HasConfig + HasDatabase + HasEventBus> FullContext for T {}
```

---

## 4. Extension Builder

**File**: `crates/shared/extension/src/builder.rs`

```rust
//! Type-safe extension builder with compile-time dependency validation.

use std::marker::PhantomData;
use crate::{
    AnyExtension, Dependencies, DependencyList, ExtensionError,
    ExtensionRegistry, ExtensionType, Subset, TypeList,
};

/// Builder for constructing an extension registry with validated dependencies.
///
/// The type parameter `Registered` tracks which extensions have been registered,
/// enabling compile-time dependency validation.
pub struct ExtensionBuilder<Registered: TypeList = ()> {
    extensions: Vec<Box<dyn AnyExtension>>,
    _marker: PhantomData<Registered>,
}

impl ExtensionBuilder<()> {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<R: TypeList> ExtensionBuilder<R> {
    /// Add an extension to the registry.
    ///
    /// This method enforces at compile time that all dependencies
    /// of `E` are already registered (present in `R`).
    ///
    /// # Type-Level Constraints
    ///
    /// - `E::Deps: Subset<R>` - all dependencies must be registered
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // This compiles - AuthExtension has no deps
    /// ExtensionBuilder::new()
    ///     .extension(AuthExtension::default())
    ///     .extension(BlogExtension::default())  // BlogExtension depends on Auth
    ///     .build()
    ///
    /// // This fails to compile - BlogExtension's deps not satisfied
    /// ExtensionBuilder::new()
    ///     .extension(BlogExtension::default())  // ERROR!
    ///     .build()
    /// ```
    pub fn extension<E>(mut self, ext: E) -> ExtensionBuilder<(E, R)>
    where
        E: ExtensionType + Dependencies + AnyExtension + 'static,
        E::Deps: Subset<R>,
    {
        self.extensions.push(Box::new(ext));
        ExtensionBuilder {
            extensions: self.extensions,
            _marker: PhantomData,
        }
    }

    /// Enable inventory-based plugin discovery (opt-in).
    ///
    /// This scans for extensions registered via the `register!` macro.
    /// Discovered extensions are validated at runtime since their
    /// dependencies cannot be checked at compile time.
    #[cfg(feature = "plugin-discovery")]
    pub fn discover_plugins(mut self) -> Self {
        for registration in inventory::iter::<ExtensionRegistration> {
            let ext = (registration.factory)();
            self.extensions.push(ext);
        }
        self
    }

    /// Build the registry, performing final validation.
    pub fn build(self) -> Result<ExtensionRegistry, ExtensionError> {
        let mut registry = ExtensionRegistry::new();

        // Sort by priority
        let mut sorted = self.extensions;
        sorted.sort_by_key(|e| e.priority());

        for ext in sorted {
            // Validate no duplicate IDs
            if registry.has(ext.id()) {
                return Err(ExtensionError::DuplicateExtension(ext.id().to_string()));
            }

            // Validate API paths
            if let Some(api) = ext.as_api() {
                registry.validate_api_path(api.base_path())?;
            }

            registry.add(ext);
        }

        // Runtime dependency validation (for discovered plugins)
        registry.validate_all_dependencies()?;

        Ok(registry)
    }
}

impl Default for ExtensionBuilder<()> {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## 5. Extension Traits

### 5.1 SchemaExtension

**File**: `crates/shared/extension/src/traits/schema.rs`

```rust
//! Database schema extension trait.

use crate::{ExtensionType, SchemaDefinition};

/// Extension that provides database schemas.
///
/// # Example
///
/// ```rust,ignore
/// impl SchemaExtension for BlogExtension {
///     fn schemas(&self) -> Vec<SchemaDefinition> {
///         vec![
///             SchemaDefinition::embedded(
///                 "markdown_content",
///                 include_str!("../schema/markdown_content.sql"),
///             ),
///             SchemaDefinition::embedded(
///                 "markdown_categories",
///                 include_str!("../schema/markdown_categories.sql"),
///             ),
///         ]
///     }
///
///     fn migration_weight(&self) -> u32 {
///         100  // User extensions: 100+
///     }
/// }
/// ```
pub trait SchemaExtension: ExtensionType {
    /// Get schema definitions for this extension.
    ///
    /// Use `include_str!` to embed SQL at compile time.
    fn schemas(&self) -> Vec<SchemaDefinition>;

    /// Weight for migration ordering (lower = earlier).
    ///
    /// - Core tables: 1-10
    /// - Infrastructure: 11-50
    /// - User extensions: 100+
    fn migration_weight(&self) -> u32 {
        100
    }
}
```

### 5.2 ApiExtension

**File**: `crates/shared/extension/src/traits/api.rs`

```rust
//! API route extension trait.

use axum::Router;
use crate::{ExtensionType, HasDatabase, HasConfig};

/// Extension that provides HTTP API routes.
///
/// # Capability-Based Context
///
/// Instead of a god-object context, declare exactly what you need:
///
/// ```rust,ignore
/// impl ApiExtension for BlogExtension {
///     type Db = PgPool;
///     type Config = BlogConfig;
///
///     fn router(&self, db: &PgPool, config: &BlogConfig) -> Router {
///         Router::new()
///             .route("/posts", get(list_posts))
///             .route("/posts/:id", get(get_post))
///     }
/// }
/// ```
pub trait ApiExtension: ExtensionType {
    /// Database type this extension needs (use `()` if none).
    type Db: Send + Sync = ();

    /// Config type this extension needs (use `()` if none).
    type Config: Send + Sync = ();

    /// Build the router for this extension.
    fn router(&self, db: &Self::Db, config: &Self::Config) -> Router;

    /// Base path for routes (e.g., "/api/v1/blog").
    fn base_path(&self) -> &'static str;

    /// Whether routes require authentication.
    fn requires_auth(&self) -> bool {
        true
    }
}

// For extensions that need full context access
pub trait ApiExtensionFull: ExtensionType {
    /// Build router with full context access.
    fn router<Ctx>(&self, ctx: &Ctx) -> Router
    where
        Ctx: HasDatabase + HasConfig + Send + Sync;

    fn base_path(&self) -> &'static str;

    fn requires_auth(&self) -> bool {
        true
    }
}
```

### 5.3 JobExtension

**File**: `crates/shared/extension/src/traits/job.rs`

```rust
//! Scheduled job extension trait.

use std::sync::Arc;
use systemprompt_traits::Job;
use crate::ExtensionType;

/// Extension that provides scheduled background jobs.
pub trait JobExtension: ExtensionType {
    /// Get jobs provided by this extension.
    fn jobs(&self) -> Vec<Arc<dyn Job>>;
}
```

---

## 6. AnyExtension Trait (Runtime Type Erasure)

**File**: `crates/shared/extension/src/any.rs`

```rust
//! Runtime type erasure for extensions.

use crate::{
    ApiExtension, ConfigExtension, ExtensionType, JobExtension,
    ProviderExtension, SchemaExtension,
};
use std::any::Any;

/// Type-erased extension for runtime storage.
///
/// Enables storing heterogeneous extensions in a single collection
/// while preserving the ability to query capabilities.
pub trait AnyExtension: Send + Sync + 'static {
    // From ExtensionType
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn priority(&self) -> u32;

    // Capability queries
    fn as_schema(&self) -> Option<&dyn SchemaExtension> { None }
    fn as_api(&self) -> Option<&dyn ApiExtensionDyn> { None }
    fn as_config(&self) -> Option<&dyn ConfigExtension> { None }
    fn as_job(&self) -> Option<&dyn JobExtension> { None }
    fn as_provider(&self) -> Option<&dyn ProviderExtension> { None }

    // Type checking
    fn as_any(&self) -> &dyn Any;
    fn type_name(&self) -> &'static str;
}

// Blanket implementation for all ExtensionType
impl<T: ExtensionType + 'static> AnyExtension for T {
    fn id(&self) -> &'static str { T::ID }
    fn name(&self) -> &'static str { T::NAME }
    fn version(&self) -> &'static str { T::VERSION }
    fn priority(&self) -> u32 { T::PRIORITY }

    fn as_any(&self) -> &dyn Any { self }
    fn type_name(&self) -> &'static str { std::any::type_name::<T>() }
}

// Specialized implementations for extensions with capabilities
impl<T: ExtensionType + SchemaExtension + 'static> AnyExtension for T {
    fn as_schema(&self) -> Option<&dyn SchemaExtension> { Some(self) }
    // ... other methods
}
```

---

## 7. Extension Registry

**File**: `crates/shared/extension/src/registry.rs`

```rust
//! Runtime extension registry.

use std::any::TypeId;
use std::collections::HashMap;
use crate::{AnyExtension, ExtensionError, ExtensionType};

/// Reserved API paths that extensions cannot use.
const RESERVED_PATHS: &[&str] = &[
    "/api/v1/oauth",
    "/api/v1/users",
    "/api/v1/agents",
    "/api/v1/mcp",
    "/api/v1/stream",
    "/api/v1/files",
    "/api/v1/analytics",
    "/api/v1/scheduler",
    "/api/v1/core",
    "/api/v1/admin",
    "/.well-known",
];

pub struct ExtensionRegistry {
    extensions: Vec<Box<dyn AnyExtension>>,
    by_id: HashMap<String, usize>,
    by_type: HashMap<TypeId, usize>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
            by_id: HashMap::new(),
            by_type: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, ext: Box<dyn AnyExtension>) {
        let idx = self.extensions.len();
        self.by_id.insert(ext.id().to_string(), idx);
        self.extensions.push(ext);
    }

    /// Check if extension type is registered.
    pub fn has_type<E: ExtensionType>(&self) -> bool {
        self.by_type.contains_key(&TypeId::of::<E>())
    }

    /// Get extension by string ID.
    pub fn get(&self, id: &str) -> Option<&dyn AnyExtension> {
        self.by_id.get(id).map(|&idx| self.extensions[idx].as_ref())
    }

    /// Get typed extension reference.
    pub fn get_typed<E: ExtensionType + 'static>(&self) -> Option<&E> {
        self.by_type
            .get(&TypeId::of::<E>())
            .and_then(|&idx| self.extensions[idx].as_any().downcast_ref())
    }

    /// Iterate schema extensions in migration order.
    pub fn schema_extensions(&self) -> impl Iterator<Item = &dyn SchemaExtension> {
        let mut schemas: Vec<_> = self.extensions
            .iter()
            .filter_map(|e| e.as_schema())
            .collect();
        schemas.sort_by_key(|s| s.migration_weight());
        schemas.into_iter()
    }

    /// Iterate API extensions.
    pub fn api_extensions(&self) -> impl Iterator<Item = &dyn ApiExtensionDyn> {
        self.extensions.iter().filter_map(|e| e.as_api())
    }

    /// Validate API path is not reserved.
    pub(crate) fn validate_api_path(&self, path: &str) -> Result<(), ExtensionError> {
        for reserved in RESERVED_PATHS {
            if path.starts_with(reserved) {
                return Err(ExtensionError::ReservedPathCollision {
                    path: path.to_string(),
                });
            }
        }
        Ok(())
    }
}
```

---

## 8. Error Types

**File**: `crates/shared/extension/src/error.rs`

```rust
//! Extension error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtensionError {
    #[error("Duplicate extension ID: {0}")]
    DuplicateExtension(String),

    #[error("Missing dependency: extension '{extension_id}' requires '{dependency_id}'")]
    MissingDependency {
        extension_id: String,
        dependency_id: String,
    },

    #[error("Reserved path collision: '{path}' is reserved for core")]
    ReservedPathCollision { path: String },

    #[error("Invalid base path: '{path}' must start with /api/")]
    InvalidBasePath { path: String },

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Schema installation failed: {0}")]
    SchemaInstallation(String),
}

#[derive(Debug)]
pub struct MissingDependency {
    pub extension_id: &'static str,
    pub extension_name: &'static str,
}
```

---

## 9. Cargo.toml Updates

**File**: `crates/shared/extension/Cargo.toml`

```toml
[package]
name = "systemprompt-extension"
version.workspace = true
edition.workspace = true
description = "Type-safe extension framework for SystemPrompt"

[features]
default = []
plugin-discovery = ["inventory"]

[dependencies]
# Type-safe traits
systemprompt-traits = { path = "../traits" }

# Web framework (for Router type)
axum = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Error handling
thiserror = { workspace = true }

# Optional: plugin discovery
inventory = { workspace = true, optional = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
```

---

## 10. Tests

**File**: `crates/shared/extension/src/tests/builder_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test extensions
    #[derive(Default)]
    struct AuthExtension;

    impl ExtensionType for AuthExtension {
        const ID: &'static str = "auth";
        const NAME: &'static str = "Authentication";
        const VERSION: &'static str = "1.0.0";
    }

    impl NoDependencies for AuthExtension {}

    #[derive(Default)]
    struct BlogExtension;

    impl ExtensionType for BlogExtension {
        const ID: &'static str = "blog";
        const NAME: &'static str = "Blog";
        const VERSION: &'static str = "1.0.0";
    }

    impl Dependencies for BlogExtension {
        type Deps = (AuthExtension, ());
    }

    #[test]
    fn test_builder_compiles_with_satisfied_deps() {
        // This should compile - AuthExtension registered before BlogExtension
        let registry = ExtensionBuilder::new()
            .extension(AuthExtension::default())
            .extension(BlogExtension::default())
            .build()
            .unwrap();

        assert!(registry.has_type::<AuthExtension>());
        assert!(registry.has_type::<BlogExtension>());
    }

    // This test verifies the compile-time check works
    // Uncomment to verify it fails to compile:
    // #[test]
    // fn test_builder_fails_compile_with_missing_deps() {
    //     ExtensionBuilder::new()
    //         .extension(BlogExtension::default())  // ERROR: AuthExtension not in ()
    //         .build();
    // }

    #[test]
    fn test_duplicate_id_rejected() {
        let result = ExtensionBuilder::new()
            .extension(AuthExtension::default())
            .extension(AuthExtension::default())
            .build();

        assert!(matches!(result, Err(ExtensionError::DuplicateExtension(_))));
    }
}
```

---

## 11. Migration from Old Extension System

### 11.1 Old API (Deprecated)

```rust
// OLD - trait object based, string dependencies
impl Extension for MyExtension {
    fn id(&self) -> &str { "my-ext" }
    fn dependencies(&self) -> &[&str] { &["auth"] }
}
register_extension!(MyExtension);
register_api_extension!(MyExtension);
```

### 11.2 New API

```rust
// NEW - type-safe, compile-time checked
#[derive(Default)]
struct MyExtension;

impl ExtensionType for MyExtension {
    const ID: &'static str = "my-ext";
    const NAME: &'static str = "My Extension";
    const VERSION: &'static str = "1.0.0";
}

impl Dependencies for MyExtension {
    type Deps = (AuthExtension, ());
}

impl ApiExtension for MyExtension {
    type Db = PgPool;
    type Config = ();

    fn router(&self, db: &PgPool, _: &()) -> Router { ... }
    fn base_path(&self) -> &'static str { "/api/v1/my-ext" }
}
```

---

## 12. Execution Checklist

- [ ] Create `crates/shared/extension/src/types.rs` - ExtensionType, Dependencies traits
- [ ] Create `crates/shared/extension/src/hlist.rs` - TypeList, Subset, Contains
- [ ] Create `crates/shared/extension/src/capabilities.rs` - HasDatabase, HasConfig, etc.
- [ ] Create `crates/shared/extension/src/builder.rs` - ExtensionBuilder<R>
- [ ] Create `crates/shared/extension/src/any.rs` - AnyExtension trait
- [ ] Create `crates/shared/extension/src/registry.rs` - ExtensionRegistry
- [ ] Create `crates/shared/extension/src/traits/` - All extension trait files
- [ ] Create `crates/shared/extension/src/error.rs` - Error types
- [ ] Update `crates/shared/extension/src/lib.rs` - New exports
- [ ] Update `crates/shared/extension/Cargo.toml` - Dependencies
- [ ] Create `crates/shared/extension/src/tests/` - Comprehensive tests
- [ ] Update `crates/app/runtime/` to use new builder pattern
- [ ] Deprecate old `register_*_extension!` macros

---

## 13. Output Artifacts

After executing this phase:

1. **New crate structure** in `crates/shared/extension/`
2. **Type-safe extension traits** with compile-time dependency checking
3. **ExtensionBuilder** for explicit registration
4. **Comprehensive test suite** proving the design works
5. **Migration path** documented for existing code

---

## 14. Dependencies on This Phase

Phase 2 (Blog Extension Extraction) depends on this phase being complete.
Phase 3 (Template Integration) depends on this phase being complete.
