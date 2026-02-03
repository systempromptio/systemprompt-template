---
title: "Extension Builder"
description: "Type-safe extension registration with compile-time dependency checking."
author: "SystemPrompt Team"
slug: "extensions/internals/extension-builder"
keywords: "builder, type-safe, dependencies, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Extension Builder

`ExtensionBuilder` provides type-safe extension registration with compile-time dependency checking using Rust's type system.

## Basic Usage

```rust
use systemprompt::extension::prelude::*;

let registry = ExtensionBuilder::new()
    .extension(UsersExtension::default())
    .extension(OAuthExtension::default())
    .extension(MyExtension::default())
    .build()?;
```

## Type-Level Tracking

The builder tracks registered types at compile time:

```rust
pub struct ExtensionBuilder<Registered: TypeList = ()> {
    extensions: Vec<Box<dyn AnyExtension>>,
    _marker: PhantomData<Registered>,
}
```

Each `extension()` call extends the type list:

```rust
impl<R: TypeList> ExtensionBuilder<R> {
    pub fn extension<E>(self, ext: E) -> ExtensionBuilder<(E, R)>
    where
        E: ExtensionType + Dependencies,
        E::Deps: Subset<R>,  // Compile-time check!
    {
        // ...
    }
}
```

## Dependency Validation

Dependencies are validated at compile time:

```rust
// UsersExtension has no dependencies
impl Dependencies for UsersExtension {
    type Deps = ();
}

// OAuthExtension depends on UsersExtension
impl Dependencies for OAuthExtension {
    type Deps = (UsersExtension,);
}

// This compiles:
ExtensionBuilder::new()
    .extension(UsersExtension::default())  // Registered: (Users, ())
    .extension(OAuthExtension::default())  // Deps (Users) subset of (Users, ()) ✓
    .build()

// This fails at compile time:
ExtensionBuilder::new()
    .extension(OAuthExtension::default())  // Deps (Users) NOT subset of () ✗
    .build()
```

## Specialized Builders

### Schema Extensions

```rust
ExtensionBuilder::new()
    .schema_extension(DatabaseExtension::default())
    .schema_extension(UsersExtension::default())
    .build()
```

### API Extensions

```rust
ExtensionBuilder::new()
    .api_extension(ContentApiExtension::default())
    .api_extension(FilesApiExtension::default())
    .build()
```

### Mixed Extensions

```rust
ExtensionBuilder::new()
    .extension(DatabaseExtension::default())
    .schema_extension(UsersExtension::default())
    .api_extension(ContentApiExtension::default())
    .build()
```

## TypeList

The type list tracks registered extensions:

```rust
pub trait TypeList: 'static {
    fn contains_type<T: 'static>() -> bool;
    fn type_ids() -> Vec<TypeId>;
    fn len() -> usize;
}

// Empty list
impl TypeList for () {
    fn contains_type<T: 'static>() -> bool { false }
    fn type_ids() -> Vec<TypeId> { vec![] }
    fn len() -> usize { 0 }
}

// Cons list
impl<Head: 'static, Tail: TypeList> TypeList for (Head, Tail) {
    fn contains_type<T: 'static>() -> bool {
        TypeId::of::<Head>() == TypeId::of::<T>() || Tail::contains_type::<T>()
    }
    // ...
}
```

## Subset Trait

Validates dependency satisfaction:

```rust
pub trait Subset<B: TypeList>: TypeList {
    fn is_subset_of() -> bool;
}

// Empty is subset of anything
impl<B: TypeList> Subset<B> for () {
    fn is_subset_of() -> bool { true }
}

// (Head, Tail) is subset of B if Head is in B and Tail is subset of B
impl<Head: 'static, Tail: TypeList + Subset<B>, B: TypeList + Contains<Head>> Subset<B> for (Head, Tail) {
    fn is_subset_of() -> bool {
        B::contains_type::<Head>() && Tail::is_subset_of()
    }
}
```

## TypedExtensionRegistry

The builder produces a typed registry:

```rust
pub struct TypedExtensionRegistry {
    extensions: Vec<Box<dyn AnyExtension>>,
    by_id: HashMap<&'static str, usize>,
}

impl TypedExtensionRegistry {
    pub fn get<E: ExtensionType>(&self) -> Option<&E>;
    pub fn get_by_id(&self, id: &str) -> Option<&dyn AnyExtension>;
    pub fn iter(&self) -> impl Iterator<Item = &dyn AnyExtension>;
}
```

## Error Handling

Runtime validation can still fail:

```rust
pub fn build(self) -> Result<TypedExtensionRegistry, LoaderError> {
    let mut registry = TypedExtensionRegistry::new();

    for ext in self.extensions {
        // Check for duplicates
        if registry.by_id.contains_key(ext.id()) {
            return Err(LoaderError::DuplicateExtension(ext.id().to_string()));
        }

        registry.add(ext);
    }

    // Validate API paths
    registry.validate_api_paths()?;

    Ok(registry)
}
```

## Example: Complete Setup

```rust
use systemprompt::extension::prelude::*;

// Define extensions with dependencies
#[derive(Debug, Default)]
struct DatabaseExtension;

impl ExtensionType for DatabaseExtension {
    const ID: &'static str = "database";
    const NAME: &'static str = "Database";
    const VERSION: &'static str = "0.1.0";
    const PRIORITY: u32 = 1;
}

impl Dependencies for DatabaseExtension {
    type Deps = ();
}

#[derive(Debug, Default)]
struct UsersExtension;

impl ExtensionType for UsersExtension {
    const ID: &'static str = "users";
    const NAME: &'static str = "Users";
    const VERSION: &'static str = "0.1.0";
    const PRIORITY: u32 = 10;
}

impl Dependencies for UsersExtension {
    type Deps = (DatabaseExtension,);
}

// Build registry
fn create_registry() -> Result<TypedExtensionRegistry, LoaderError> {
    ExtensionBuilder::new()
        .extension(DatabaseExtension)
        .extension(UsersExtension)
        .build()
}
```

## When to Use

Use `ExtensionBuilder` when:
- Building a fixed set of extensions
- Want compile-time dependency validation
- Need type-safe extension access

Use `ExtensionRegistry::discover()` when:
- Extensions register dynamically
- Using `register_extension!` macro
- Building plugin systems