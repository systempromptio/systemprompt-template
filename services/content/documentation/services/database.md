---
title: "Database Service"
description: "Database access patterns in Rust code - connecting to PostgreSQL, using repositories, and accessing the pool in different contexts"
author: "systemprompt.io"
slug: "database"
keywords: "database, postgresql, sqlx, repository, DbPool, PgPool, rust"
image: "/files/images/docs/database.svg"
kind: "guide"
public: true
tags: ["services", "database", "rust", "patterns"]
published_at: "2026-02-02"
updated_at: "2026-02-02"
after_reading_this:
  - "Access the database pool in MCP handlers, jobs, and page providers"
  - "Create and use repositories for typed database access"
  - "Understand the difference between DbPool and Database types"
  - "Use extension repositories from other crates"
related_playbooks:
  - title: "Database Operations Playbook"
    url: "/playbooks/cli-database"
  - title: "Database Access Patterns"
    url: "/playbooks/build-database-access"
  - title: "MCP Server Checklist"
    url: "/playbooks/build-mcp-checklist"
related_code:
  - title: "ContentRepository"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/web/src/repository/content/mod.rs"
  - title: "MemoryService"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/soul/src/services/memory.rs"
related_docs:
  - title: "Services Overview"
    url: "/documentation/services"
  - title: "MCP Extensions"
    url: "/documentation/extensions/domains/mcp"
links:
  - title: "SQLx Documentation"
    url: "https://docs.rs/sqlx/latest/sqlx/"
  - title: "PostgreSQL"
    url: "https://www.postgresql.org/docs/"
---

# Database Service

SystemPrompt uses PostgreSQL with SQLx for compile-time checked queries. This guide covers how to access the database in different contexts.

## Core Types

SystemPrompt provides two main database wrapper types:

| Type | Import | Use Case |
|------|--------|----------|
| `DbPool` | `systemprompt::database::DbPool` | MCP servers, background jobs |
| `Database` | `systemprompt::database::Database` | Page data providers, extensions |

Both types wrap `Arc<PgPool>` and provide a `.pool()` method to access the underlying connection pool.

## Accessing the Pool

### In MCP Handlers

MCP tool handlers receive `&DbPool` as a parameter. Extract the `Arc<PgPool>` with `.pool()`:

```rust
use systemprompt::database::DbPool;
use rmcp::ErrorData as McpError;

pub async fn handle(
    db_pool: &DbPool,
    request: CallToolRequestParams,
    // ... other params
) -> Result<CallToolResult, McpError> {
    // Extract the pool - returns Option<Arc<PgPool>>
    let pg_pool = db_pool.pool().ok_or_else(|| {
        McpError::internal_error("Database pool not available", None)
    })?;

    // Create a repository
    let content_repo = ContentRepository::new(pg_pool);

    // Use the repository
    let content = content_repo.create(&params).await.map_err(|e| {
        McpError::internal_error(format!("Database error: {e}"), None)
    })?;

    Ok(result)
}
```

### In Background Jobs

Jobs access the pool via `JobContext`:

```rust
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

#[async_trait::async_trait]
impl Job for MyJob {
    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        // Get DbPool from context
        let db_pool = ctx
            .db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        // Get the underlying pool
        let pool = db_pool.pool()
            .ok_or_else(|| anyhow::anyhow!("Pool not initialized"))?;

        // Use directly with sqlx
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&*pool)
            .await?;

        Ok(JobResult::success())
    }
}
```

### In Page Data Providers

Page providers receive `Arc<Database>` from `PageContext`:

```rust
use std::sync::Arc;
use systemprompt::database::Database;
use systemprompt::template_provider::{PageContext, PageDataProvider};

#[async_trait]
impl PageDataProvider for MyProvider {
    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        // Get Database wrapper
        let Some(db) = ctx.db_pool::<Arc<Database>>() else {
            return Ok(json!({ "data": [] }));
        };

        // Extract PgPool
        let Some(pool) = db.pool() else {
            return Ok(json!({ "data": [] }));
        };

        // Use with sqlx query macros
        let rows = sqlx::query_as!(
            MyRow,
            "SELECT id, name FROM my_table WHERE active = true"
        )
        .fetch_all(&*pool)
        .await?;

        Ok(json!({ "data": rows }))
    }
}
```

## Repository Pattern

Repositories provide typed, reusable database access. They encapsulate queries and mutations for a specific domain.

### Creating a Repository

```rust
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MyRepository {
    pool: Arc<PgPool>,
}

impl MyRepository {
    /// Create a new repository with the given pool
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Create a new record
    pub async fn create(&self, params: &CreateParams) -> Result<MyModel, sqlx::Error> {
        sqlx::query_as!(
            MyModel,
            r#"
            INSERT INTO my_table (id, name, created_at)
            VALUES ($1, $2, NOW())
            RETURNING id, name, created_at
            "#,
            params.id,
            params.name
        )
        .fetch_one(&*self.pool)
        .await
    }

    /// Get a record by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<MyModel>, sqlx::Error> {
        sqlx::query_as!(
            MyModel,
            "SELECT id, name, created_at FROM my_table WHERE id = $1",
            id
        )
        .fetch_optional(&*self.pool)
        .await
    }

    /// List records with pagination
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<MyModel>, sqlx::Error> {
        sqlx::query_as!(
            MyModel,
            "SELECT id, name, created_at FROM my_table ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
    }
}
```

### Query and Mutation Separation

For larger repositories, separate queries and mutations:

```rust
// repository/mod.rs
mod queries;
mod mutations;

pub use mutations::UpdateParams;

pub struct ContentRepository {
    queries: ContentQueryRepository,
    mutations: ContentMutationRepository,
}

impl ContentRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            queries: ContentQueryRepository::new(Arc::clone(&pool)),
            mutations: ContentMutationRepository::new(pool),
        }
    }

    pub async fn get_by_id(&self, id: &ContentId) -> Result<Option<Content>, sqlx::Error> {
        self.queries.get_by_id(id).await
    }

    pub async fn create(&self, params: &CreateParams) -> Result<Content, sqlx::Error> {
        self.mutations.create(params).await
    }
}
```

## Using Extension Repositories

Extensions export their repositories. Add dependencies in `Cargo.toml`:

```toml
[dependencies]
systemprompt-web-extension = { path = "../../web" }
systemprompt-soul-extension = { path = "../../soul" }
```

Then use the repositories:

```rust
use systemprompt_web_extension::{ContentRepository, CreateContentParams, ContentKind};
use systemprompt_soul_extension::{MemoryService, CreateMemoryParams};

// Get the pool
let pg_pool = db_pool.pool().ok_or_else(|| {
    McpError::internal_error("Database pool not available", None)
})?;

// Create content using web extension's repository
let content_repo = ContentRepository::new(pg_pool.clone());
let content = content_repo.create(&CreateContentParams::new(
    "my-slug".to_string(),
    "My Title".to_string(),
    "Description".to_string(),
    body_content,
    "Edward".to_string(),
    Utc::now(),
    SourceId::new("blog".to_string()),
)).await?;

// Store memory using soul extension's service
let memory_service = MemoryService::new(pg_pool);
memory_service.store(&CreateMemoryParams::new(
    MemoryType::LongTerm,
    MemoryCategory::Fact,
    "blog: my-slug",
    "Created blog post about...",
)).await?;
```

## SQLx Query Macros

SQLx provides compile-time checked query macros. Set `DATABASE_URL` for compile-time verification:

```bash
export DATABASE_URL="postgres://user:pass@localhost/dbname"
cargo build
```

### Query Patterns

```rust
// Fetch one row (errors if not found)
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_one(&*pool)
    .await?;

// Fetch optional row
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_optional(&*pool)
    .await?;

// Fetch multiple rows
let users = sqlx::query_as!(User, "SELECT * FROM users WHERE active = true")
    .fetch_all(&*pool)
    .await?;

// Execute without returning (INSERT, UPDATE, DELETE)
sqlx::query!("DELETE FROM users WHERE id = $1", id)
    .execute(&*pool)
    .await?;
```

## Error Handling

Convert sqlx errors appropriately for your context:

```rust
// In MCP handlers
content_repo.create(&params).await.map_err(|e| {
    tracing::error!(error = %e, "Database operation failed");
    McpError::internal_error(format!("Database error: {e}"), None)
})?;

// In jobs
content_repo.create(&params).await
    .context("Failed to create content")?;

// In services with custom error types
content_repo.create(&params).await
    .map_err(BlogError::from)?;
```

## Best Practices

1. **Always handle pool unavailability** - The pool may not be initialized in all contexts
2. **Use typed repositories** - Avoid raw SQL scattered throughout the codebase
3. **Clone the Arc, not the pool** - `Arc::clone(&pool)` is cheap
4. **Log database errors** - Include context for debugging
5. **Use transactions for multi-step operations** - Ensure atomicity
