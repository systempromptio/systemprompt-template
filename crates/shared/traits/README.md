# systemprompt-traits

Shared traits and contracts for SystemPrompt OS.

## Overview

This crate provides the core trait definitions that enable polymorphism, dependency injection, and consistent patterns across the SystemPrompt OS codebase. It has minimal dependencies and no dependencies on other SystemPrompt crates.

## Traits

### Repository Traits

**`Repository`** - Base trait for all repository implementations
```rust
use systemprompt_traits::{Repository, RepositoryError};

impl Repository for MyRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}
```

**`CrudRepository<T>`** - Generic CRUD operations trait
```rust
use systemprompt_traits::CrudRepository;

impl CrudRepository<User> for UserRepository {
    type Id = String;

    async fn create(&self, entity: User) -> Result<User, Self::Error> { ... }
    async fn get(&self, id: Self::Id) -> Result<Option<User>, Self::Error> { ... }
    async fn update(&self, entity: User) -> Result<User, Self::Error> { ... }
    async fn delete(&self, id: Self::Id) -> Result<(), Self::Error> { ... }
    async fn list(&self) -> Result<Vec<User>, Self::Error> { ... }
}
```

**`RepositoryError`** - Standard error type for repository operations
```rust
pub enum RepositoryError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    SerializationError(serde_json::Error),
    InvalidData(String),
    ConstraintViolation(String),
    GenericError(anyhow::Error),
}
```

### Context Traits

**`AppContext`** - Application context trait for dependency injection
```rust
use systemprompt_traits::AppContext;

impl AppContext for MyAppContext {
    fn config(&self) -> Arc<dyn ConfigProvider> { ... }
    fn module_registry(&self) -> Arc<dyn ModuleRegistry> { ... }
    fn database_handle(&self) -> Arc<dyn DatabaseHandle> { ... }
}
```

**`ConfigProvider`** - Configuration provider trait
```rust
impl ConfigProvider for Config {
    fn get(&self, key: &str) -> Option<String> { ... }
    fn database_url(&self) -> &str { ... }
    fn system_path(&self) -> &str { ... }
    fn jwt_secret(&self) -> &str { ... }
    fn api_port(&self) -> u16 { ... }
}
```

**`ModuleRegistry`** - Module registry trait for dynamic module management
```rust
impl ModuleRegistry for MyModuleRegistry {
    fn get_module(&self, name: &str) -> Option<Arc<dyn Module>> { ... }
    fn list_modules(&self) -> Vec<String> { ... }
}
```

### Service Traits

**`Service`** - Base service trait with lifecycle methods
```rust
use systemprompt_traits::Service;

#[async_trait]
impl Service for MyService {
    fn name(&self) -> &str { "my-service" }

    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ... }
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ... }
    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { ... }
}
```

**`AsyncService`** - Async service trait for long-running background tasks
```rust
use systemprompt_traits::AsyncService;

#[async_trait]
impl AsyncService for MyAsyncService {
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Long-running task
    }
}
```

### Module Traits

**`Module`** - Core module trait for SystemPrompt modules
```rust
#[async_trait]
impl Module for MyModule {
    fn name(&self) -> &str { "my-module" }
    fn version(&self) -> &str { "1.0.0" }
    fn display_name(&self) -> &str { "My Module" }
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> { ... }
}
```

**`ApiModule`** - Module trait with REST API support
```rust
#[async_trait]
impl ApiModule for MyApiModule {
    fn router(&self, ctx: Arc<dyn AppContext>) -> axum::Router { ... }
}
```

## Usage Patterns

### When to Use Traits vs Concrete Types

**Use Traits When:**
- You need dependency injection for testing
- You want to support multiple implementations
- You're defining interfaces between modules
- You need polymorphic behavior

**Use Concrete Types When:**
- Performance is critical and trait objects add overhead
- There's only one implementation
- The API is module-internal
- Type inference is important

### Testing with Traits

Traits enable easy mocking:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepository {
        pool: MockPool,
    }

    impl Repository for MockRepository {
        type Pool = MockPool;
        type Error = RepositoryError;

        fn pool(&self) -> &Self::Pool { &self.pool }
    }

    #[tokio::test]
    async fn test_with_mock() {
        let repo = MockRepository { pool: MockPool::new() };
        // Test using trait methods
    }
}
```

### Error Handling

All repository errors automatically convert to `ApiError`:

```rust
use systemprompt_models::{ApiError, RepositoryError};

let result: Result<User, RepositoryError> = repo.get_user("id").await;
let api_result: Result<User, ApiError> = result.map_err(|e| e.into());
```

## Architecture

This crate follows the **Interface Segregation Principle** from SOLID:
- Traits are small and focused
- Clients depend only on the methods they use
- No fat interfaces or forced implementations

## Dependencies

Minimal dependencies to avoid circular deps:
- `async-trait` - Async trait support
- `anyhow` - Error handling
- `axum` - Router type for ApiModule
- `inventory` - Module registration
- `thiserror` - Error derive macros
- `sqlx` - Database types for Repository
- `serde_json` - Serialization errors

**No dependencies on other SystemPrompt crates** - this is intentional to prevent circular dependencies.
