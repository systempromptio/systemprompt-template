---
title: "Add API Routes"
description: "Add HTTP endpoints to your extension using Axum."
author: "SystemPrompt"
slug: "build-02-library-extensions-add-api-routes"
keywords: "api, routes, http, axum"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Add API Routes

Add HTTP endpoints to your extension. Reference: `extensions/web/src/api/` for examples.

> **Help**: `{ "command": "core playbooks show build_add-api-routes" }`

---

## Structure

```
extensions/my-extension/src/
├── api/
│   ├── mod.rs
│   └── handlers/
│       └── mod.rs
└── extension.rs
```

---

## Create Router

File: `src/api/mod.rs`. See `extensions/web/src/api/mod.rs:1-25` for reference.

```rust
use axum::{Router, routing::{get, post, put, delete}};
use std::sync::Arc;
use sqlx::PgPool;

mod handlers;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
}

pub fn router(pool: Arc<PgPool>) -> Router {
    let state = AppState { pool };

    Router::new()
        .route("/items", get(handlers::list).post(handlers::create))
        .route("/items/:id", get(handlers::get).put(handlers::update).delete(handlers::delete))
        .with_state(state)
}
```

---

## Create Handlers

File: `src/api/handlers/mod.rs`. See `extensions/web/src/api/handlers/` for reference.

```rust
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
};
use uuid::Uuid;

use crate::api::AppState;
use crate::error::MyExtensionError;
use crate::models::Item;

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<Item>>, MyExtensionError> {
    let items = sqlx::query_as!(Item, "SELECT * FROM my_items ORDER BY created_at DESC")
        .fetch_all(state.pool.as_ref())
        .await?;
    Ok(Json(items))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Item>, MyExtensionError> {
    let item = sqlx::query_as!(Item, "SELECT * FROM my_items WHERE id = $1", id)
        .fetch_optional(state.pool.as_ref())
        .await?
        .ok_or_else(|| MyExtensionError::NotFound(id.to_string()))?;
    Ok(Json(item))
}

#[derive(serde::Deserialize)]
pub struct CreateInput {
    name: String,
    description: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateInput>,
) -> Result<(StatusCode, Json<Item>), MyExtensionError> {
    let item = sqlx::query_as!(
        Item,
        "INSERT INTO my_items (name, description) VALUES ($1, $2) RETURNING *",
        input.name,
        input.description
    )
    .fetch_one(state.pool.as_ref())
    .await?;

    Ok((StatusCode::CREATED, Json(item)))
}
```

---

## Register Router

In `src/extension.rs`. See `extensions/web/src/extension.rs:60-70` for reference.

```rust
fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
    let db = ctx.database();
    let pool = db.as_any().downcast_ref::<Database>()?.pool()?;
    Some(ExtensionRouter::new(crate::api::router(pool), "/api/v1/my-extension"))
}
```

---

## Error Response

In `src/error.rs`:

```rust
impl axum::response::IntoResponse for MyExtensionError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({
            "error": {
                "code": self.code(),
                "message": self.to_string(),
            }
        });

        (self.status(), Json(body)).into_response()
    }
}
```

---

## Checklist

- [ ] `src/api/mod.rs` with router function
- [ ] Router function takes `Arc<PgPool>`
- [ ] State struct with pool
- [ ] Handlers use `State` extractor
- [ ] Error type implements `IntoResponse`
- [ ] Uses `sqlx::query_as!` macros
- [ ] Returns appropriate status codes
- [ ] Registered in `router()` method

---

## Quick Reference

| Task | Command/Action |
|------|----------------|
| Build | `cargo build --workspace` |
| Test endpoint | `curl http://localhost:8080/api/v1/my-extension/items` |
| Check routes | `cargo run -- extensions list --routes` |

---

## Related

-> See [Create Library Extension](create-extension.md) for full extension setup
-> See [API Extension](../../documentation/extensions/traits/api-extension.md) for trait reference
-> See [Rust Standards](../06-standards/rust-standards.md) for code style