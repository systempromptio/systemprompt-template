---
title: "Database Access Patterns"
description: "How to access the database in Rust code across different contexts (MCP servers, jobs, page providers)."
keywords:
  - database
  - rust
  - repository
  - DbPool
  - PgPool
category: build
---

# Database Access Patterns

How to access the database in Rust code across different contexts.

---

## Core Types

| Type | Location | Use Case |
|------|----------|----------|
| `DbPool` | `systemprompt::database::DbPool` | MCP servers, jobs |
| `Database` | `systemprompt::database::Database` | Page data providers, extensions |
| `Arc<PgPool>` | `sqlx::PgPool` | Repository constructors |

---

## Pattern 1: MCP Server Handlers

MCP handlers receive `&DbPool`. Extract the underlying `Arc<PgPool>` with `.pool()`.

```rust
use systemprompt::database::DbPool;
use systemprompt_web_extension::ContentRepository;

pub async fn handle(
    db_pool: &DbPool,
    // ... other params
) -> Result<CallToolResult, McpError> {
    // Extract Arc<PgPool> from DbPool
    let pg_pool = db_pool.pool().ok_or_else(|| {
        McpError::internal_error("Database pool not available", None)
    })?;

    // Create repository with the pool
    let content_repo = ContentRepository::new(pg_pool);

    // Use repository
    let content = content_repo.create(&params).await.map_err(|e| {
        McpError::internal_error(format!("Database error: {e}"), None)
    })?;

    Ok(result)
}
```

**Key points:**
- `DbPool.pool()` returns `Option<Arc<PgPool>>`
- Always handle the `None` case (pool not initialized)
- Repositories take `Arc<PgPool>` in their constructors

---

## Pattern 2: Job Handlers

Jobs access the pool via `JobContext.db_pool()`.

```rust
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

#[async_trait::async_trait]
impl Job for MyJob {
    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let db_pool = ctx
            .db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        // For direct sqlx queries, clone the pool
        let result = sqlx::query("SELECT COUNT(*) FROM users")
            .fetch_one(&**db_pool.pool().unwrap())
            .await?;

        // Or create a repository
        let repo = ContentRepository::new(db_pool.pool().unwrap());

        Ok(JobResult::success())
    }
}
```

---

## Pattern 3: Page Data Providers

Page providers access `Arc<Database>` from `PageContext`.

```rust
use std::sync::Arc;
use systemprompt::database::Database;
use systemprompt::template_provider::{PageContext, PageDataProvider};

#[async_trait]
impl PageDataProvider for MyProvider {
    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        // Get Database wrapper
        let Some(db) = ctx.db_pool::<Arc<Database>>() else {
            return Ok(json!({ "data": "" }));
        };

        // Extract PgPool
        let Some(pool) = db.pool() else {
            return Ok(json!({ "data": "" }));
        };

        // Use sqlx directly
        let rows = sqlx::query_as!(
            MyRow,
            "SELECT * FROM my_table WHERE active = true"
        )
        .fetch_all(&*pool)
        .await?;

        Ok(json!({ "data": rows }))
    }
}
```

---

## Creating Repositories

Repositories wrap database access with typed methods.

### Repository Pattern

```rust
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MyRepository {
    pool: Arc<PgPool>,
}

impl MyRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

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

    pub async fn get_by_id(&self, id: &str) -> Result<Option<MyModel>, sqlx::Error> {
        sqlx::query_as!(
            MyModel,
            "SELECT * FROM my_table WHERE id = $1",
            id
        )
        .fetch_optional(&*self.pool)
        .await
    }
}
```

---

## Using Extension Repositories

Extensions export their repositories. Add the extension as a dependency.

### Cargo.toml

```toml
[dependencies]
systemprompt-web-extension = { path = "../../web" }
systemprompt-soul-extension = { path = "../../soul" }
```

### Usage

```rust
use systemprompt_web_extension::{ContentRepository, CreateContentParams, ContentKind};
use systemprompt_soul_extension::{MemoryService, CreateMemoryParams};

// Content repository (web extension)
let content_repo = ContentRepository::new(pg_pool.clone());
let content = content_repo.create(&CreateContentParams::new(
    slug,
    title,
    description,
    body,
    author,
    Utc::now(),
    SourceId::new("blog".to_string()),
)).await?;

// Memory service (soul extension)
let memory_service = MemoryService::new(pg_pool);
let memory = memory_service.store(&CreateMemoryParams::new(
    memory_type,
    category,
    subject,
    content,
)).await?;
```

---

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `expected &Arc<Pool<Postgres>>, found &Arc<Database>` | Wrong type passed to repository | Use `db_pool.pool()` to extract `Arc<PgPool>` |
| `Database pool not available` | Pool not initialized | Check service startup, ensure DB connection |
| `Pool not initialized` | Lazy init not triggered | Database may not be connected yet |

---

## Quick Reference

| Context | Get Pool | Type |
|---------|----------|------|
| MCP handler | `db_pool.pool()?` | `Option<Arc<PgPool>>` |
| Job | `ctx.db_pool::<DbPool>()?.pool()?` | `Option<Arc<PgPool>>` |
| Page provider | `ctx.db_pool::<Arc<Database>>()?.pool()?` | `Option<Arc<PgPool>>` |
| Repository constructor | `Repository::new(pool)` | `Arc<PgPool>` |
