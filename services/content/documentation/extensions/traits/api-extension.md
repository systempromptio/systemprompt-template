---
title: "API Extension"
description: "Add HTTP routes and API endpoints to your extension."
author: "SystemPrompt Team"
slug: "extensions/traits/api-extension"
keywords: "api, routes, http, axum, endpoints"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# API Extension

Extensions add HTTP endpoints via the `router()` method using Axum.

## Router Method

```rust
fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
    let db = ctx.database();
    let pool = db.as_any().downcast_ref::<Database>()?.pool()?;

    let router = Router::new()
        .route("/items", get(list_items).post(create_item))
        .route("/items/:id", get(get_item).put(update_item).delete(delete_item))
        .with_state(AppState { pool });

    Some(ExtensionRouter::new(router, "/api/v1/my-extension"))
}
```

## ExtensionRouter

```rust
// Authenticated route (default)
ExtensionRouter::new(router, "/api/v1/my-extension")

// Public route (no auth required)
ExtensionRouter::public(router, "/api/v1/public")
```

## Router Config

Return configuration without building the router:

```rust
fn router_config(&self) -> Option<ExtensionRouterConfig> {
    Some(ExtensionRouterConfig::new("/api/v1/my-extension"))
}
```

## Reserved Paths

Extensions cannot use these paths:

```rust
pub const RESERVED_PATHS: &[&str] = &[
    "/api/v1/oauth",
    "/api/v1/users",
    "/api/v1/agents",
    "/api/v1/mcp",
    "/api/v1/stream",
    "/api/v1/content",
    "/api/v1/files",
    "/api/v1/analytics",
    "/api/v1/scheduler",
    "/api/v1/core",
    "/api/v1/admin",
    "/.well-known",
];
```

## Handler Pattern

```rust
use axum::{extract::{Path, State, Json}, response::IntoResponse};

#[derive(Clone)]
struct AppState {
    pool: Arc<PgPool>,
}

async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Item>, AppError> {
    let item = sqlx::query_as!(Item, "SELECT * FROM items WHERE id = $1", id)
        .fetch_one(state.pool.as_ref())
        .await?;
    Ok(Json(item))
}

async fn create_item(
    State(state): State<AppState>,
    Json(input): Json<CreateItemInput>,
) -> Result<(StatusCode, Json<Item>), AppError> {
    let item = sqlx::query_as!(
        Item,
        "INSERT INTO items (name) VALUES ($1) RETURNING *",
        input.name
    )
    .fetch_one(state.pool.as_ref())
    .await?;

    Ok((StatusCode::CREATED, Json(item)))
}
```

## Typed Extension

For compile-time type safety:

```rust
use systemprompt::extension::prelude::{ApiExtensionTyped, ApiExtensionTypedDyn};

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
    }
}
```