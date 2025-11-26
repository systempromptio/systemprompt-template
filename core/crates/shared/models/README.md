# systemprompt-models

Shared data models, types, and repository patterns for SystemPrompt OS.

## Overview

This crate provides common data models, error types, and repository patterns used throughout SystemPrompt OS. It includes API models, authentication types, configuration, database models, and service-layer error handling.

## Modules

### `api` - API Response Models

Standard JSON API response structures:

```rust
use systemprompt_models::{ApiResponse, ApiError, ErrorCode};

let response = ApiResponse::success(data);
let error = ApiError::not_found("User not found");
```

### `auth` - Authentication Models

```rust
use systemprompt_models::{AuthenticatedUser, BaseRole, AuthError};

let user = AuthenticatedUser {
    id: "user-123".to_string(),
    username: "alice".to_string(),
    roles: vec![BaseRole::Admin],
    scopes: vec!["admin".to_string()],
};
```

### `config` - Configuration Models

```rust
use systemprompt_models::Config;
use systemprompt_traits::ConfigProvider;

let config = Config::from_env()?;
let db_url = config.database_url(); // ConfigProvider trait
```

### `errors` - Error Handling

**`RepositoryError`** - Database/repository layer errors:
```rust
use systemprompt_models::RepositoryError;

let err = RepositoryError::NotFound("user-123".to_string());
// Automatically converts to ApiError
let api_err: ApiError = err.into();
```

**`ServiceError`** - Business logic layer errors:
```rust
use systemprompt_models::ServiceError;

let err = ServiceError::Validation("Invalid email".to_string());
let api_err: ApiError = err.into(); // Converts to HTTP 400
```

**Error Conversion Flow:**
```
RepositoryError → ServiceError → ApiError → HTTP Response
```

### `repository` - Repository Patterns

**Service Lifecycle Trait:**
```rust
use systemprompt_models::ServiceLifecycle;

#[async_trait]
impl ServiceLifecycle for MyServiceRepository {
    async fn get_running_services(&self) -> Result<Vec<ServiceRecord>, RepositoryError> { ... }
    async fn mark_crashed(&self, service_id: &str) -> Result<(), RepositoryError> { ... }
    async fn update_status(&self, service_id: &str, status: &str) -> Result<(), RepositoryError> { ... }
}
```

**Query Builder:**
```rust
use systemprompt_models::WhereClause;

let (clause, params) = WhereClause::new()
    .eq("status", "active")
    .is_not_null("pid")
    .build();

let query = format!("SELECT * FROM services {}", clause);
```

**Repository Macros:**
```rust
use systemprompt_models::impl_repository_base;

impl_repository_base!(MyRepository, DbPool, db_pool);

// Expands to:
// impl Repository for MyRepository {
//     type Pool = DbPool;
//     type Error = RepositoryError;
//     fn pool(&self) -> &Self::Pool { &self.db_pool }
// }
```

### `execution` - Execution Context

```rust
use systemprompt_core_system::RequestContext;

let req_ctx = RequestContext {
    session_id: "session-123".into(),
    trace_id: "trace-456".into(),
    user_id: "user-789".into(),
    context_id: "ctx-000".into(),
    task_id: None,
    ai_tool_call_id: None,
    client_id: None,
    auth_token: None,
    user: None,
    start_time: std::time::Instant::now(),
    user_type: UserType::AdminUser,
};
```

## Error Handling Pattern

SystemPrompt uses a layered error handling approach:

### Layer 1: Repository (Database)

```rust
use systemprompt_traits::RepositoryError;

async fn get_user(&self, id: &str) -> Result<User, RepositoryError> {
    sqlx::query_as(...)
        .fetch_optional(self.pool().pool())
        .await?
        .ok_or_else(|| RepositoryError::NotFound(format!("User {}", id)))
}
```

### Layer 2: Service (Business Logic)

```rust
use systemprompt_models::ServiceError;

async fn create_user(&self, data: CreateUser) -> Result<User, ServiceError> {
    if data.email.is_empty() {
        return Err(ServiceError::Validation("Email required".into()));
    }

    self.repo.create_user(data)
        .await
        .map_err(|e| e.into()) // RepositoryError → ServiceError
}
```

### Layer 3: API (HTTP)

```rust
use systemprompt_models::ApiError;

async fn create_user_handler(
    State(service): State<UserService>,
    Json(data): Json<CreateUser>,
) -> Result<Json<User>, ApiError> {
    let user = service.create_user(data)
        .await
        .map_err(|e| e.into())?; // ServiceError → ApiError

    Ok(Json(user))
}
```

## Repository Pattern

All repositories should implement the `Repository` trait from `systemprompt-traits`:

```rust
use systemprompt_traits::{Repository, RepositoryError};
use systemprompt_core_database::DbPool;

pub struct UserRepository {
    db_pool: DbPool,
}

impl Repository for UserRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl UserRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_user(&self, id: &str) -> Result<Option<User>, RepositoryError> {
        sqlx::query_as::<_, User>(GET_USER_QUERY)
            .bind(id)
            .fetch_optional(self.pool().pool())
            .await
            .map_err(|e| e.into())
    }
}

const GET_USER_QUERY: &str = "SELECT * FROM users WHERE id = ?";
```

## Query Helpers

### WhereClause Builder

```rust
use systemprompt_models::WhereClause;

let (clause, params) = WhereClause::new()
    .eq("status", "active")
    .is_not_null("deleted_at")
    .like("name", "%john%")
    .in_list("role", vec!["admin".into(), "user".into()])
    .build();

// clause = "WHERE status = ? AND deleted_at IS NOT NULL AND name LIKE ? AND role IN (?, ?)"
// params = vec!["active", "%john%", "admin", "user"]
```

### Repository Macros

```rust
// Base trait implementation
impl_repository_base!(UserRepository, DbPool, db_pool);

// Query execution
let users = repository_query!(
    self.pool(),
    "SELECT * FROM users WHERE status = ?",
    "active"
)?;

// Execute statement
repository_execute!(
    self.pool(),
    "UPDATE users SET status = ? WHERE id = ?",
    "inactive",
    user_id
)?;
```

## Module Models

```rust
use systemprompt_models::{Module, ModuleType, ServiceCategory};

let module = Module {
    id: "mod-123".to_string(),
    name: "my-module".to_string(),
    version: "1.0.0".to_string(),
    display_name: "My Module".to_string(),
    category: ServiceCategory::Core,
    module_type: ModuleType::Regular,
    enabled: true,
    config: HashMap::new(),
    ..Default::default()
};
```

## Dependencies

- `serde` / `serde_json` - Serialization
- `sqlx` - Database types
- `anyhow` / `thiserror` - Error handling
- `chrono` / `uuid` - Common types
- `axum` - Request types for analytics
- `async-trait` - Async traits
- `systemprompt-traits` - Core trait definitions
- `systemprompt-core-logging` - Logging context

## Best Practices

### 1. Use Shared Error Types

```rust
// ✅ Good
async fn my_repo_method(&self) -> Result<Data, RepositoryError> { ... }

// ❌ Bad
async fn my_repo_method(&self) -> Result<Data, anyhow::Error> { ... }
```

### 2. Layer Your Errors

```rust
// Repository layer
Result<T, RepositoryError>

// Service layer
Result<T, ServiceError>

// API layer
Result<T, ApiError>
```

### 3. Use Query Builders

```rust
// ✅ Good
let (clause, params) = WhereClause::new().eq("status", status).build();

// ❌ Bad
let clause = format!("WHERE status = '{}'", status); // SQL injection risk!
```

### 4. Implement Repository Trait

```rust
// ✅ Good - Consistent pattern
impl Repository for MyRepository { ... }

// ❌ Bad - No trait, inconsistent
impl MyRepository {
    pub fn get_pool(&self) -> &DbPool { ... } // Different name
}
```

## Testing

Mock repositories using traits:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockUserRepository {
        users: Vec<User>,
    }

    impl Repository for MockUserRepository {
        type Pool = ();
        type Error = RepositoryError;
        fn pool(&self) -> &Self::Pool { &() }
    }

    #[tokio::test]
    async fn test_user_service() {
        let repo = MockUserRepository { users: vec![] };
        let service = UserService::new(repo);
        // Test service logic
    }
}
```
