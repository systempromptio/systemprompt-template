---
title: "Extension Trait Reference"
description: "Complete reference for the Extension trait with all 30+ methods for database, API, jobs, providers, and web rendering."
author: "SystemPrompt Team"
slug: "extensions/traits/extension-trait"
keywords: "extension, trait, reference, methods, hooks"
image: "/files/images/docs/extension-trait.svg"
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Extension Trait Reference

The `Extension` trait is the core interface for library extensions. All methods have default implementations, so you only override what your extension needs.

## Trait Definition

```rust
pub trait Extension: Send + Sync + 'static {
    // Required
    fn metadata(&self) -> ExtensionMetadata;

    // All other methods have defaults
    // ...
}
```

---

## Metadata Methods

### `metadata()` (Required)

Returns extension identity information. This is the only required method.

```rust
fn metadata(&self) -> ExtensionMetadata {
    ExtensionMetadata {
        id: "my-extension",
        name: "My Extension",
        version: env!("CARGO_PKG_VERSION"),
    }
}
```

### `id()`, `name()`, `version()`

Convenience methods that delegate to `metadata()`.

```rust
fn id(&self) -> &'static str { self.metadata().id }
fn name(&self) -> &'static str { self.metadata().name }
fn version(&self) -> &'static str { self.metadata().version }
```

### `priority()`

Controls loading order. Lower values load first.

```rust
fn priority(&self) -> u32 {
    100  // Default
}
```

### `is_required()`

If true, the runtime fails to start without this extension.

```rust
fn is_required(&self) -> bool {
    false  // Default
}
```

### `dependencies()`

List extension IDs that must load before this one.

```rust
fn dependencies(&self) -> Vec<&'static str> {
    vec!["users", "oauth"]
}
```

---

## Database Methods

### `schemas()`

Returns database table definitions to execute at startup.

```rust
fn schemas(&self) -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::inline("users", include_str!("../schema/001_users.sql")),
        SchemaDefinition::file("sessions", "schema/002_sessions.sql"),
    ]
}
```

### `migration_weight()`

Controls schema execution order. Lower values run first.

```rust
fn migration_weight(&self) -> u32 {
    100  // Default
}
```

### `migrations()`

Returns versioned migrations for schema evolution.

```rust
fn migrations(&self) -> Vec<Migration> {
    vec![
        Migration::new(1, "add_email_column", "ALTER TABLE users ADD COLUMN email TEXT"),
        Migration::new(2, "add_email_index", "CREATE INDEX idx_users_email ON users(email)"),
    ]
}
```

### `has_schemas()`, `has_migrations()`

Check if extension provides schemas or migrations.

---

## API Methods

### `router()` (requires `web` feature)

Returns an Axum router to mount at startup.

```rust
fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
    let db = ctx.database();
    let pool = db.as_any().downcast_ref::<Database>()?.pool()?;

    let router = Router::new()
        .route("/items", get(list_items))
        .with_state(pool);

    Some(ExtensionRouter::new(router, "/api/v1/my-extension"))
}
```

### `router_config()`

Returns router configuration without building the router.

```rust
fn router_config(&self) -> Option<ExtensionRouterConfig> {
    Some(ExtensionRouterConfig::new("/api/v1/my-extension"))
}
```

### `has_router()`

Check if extension provides a router.

---

## Job Methods

### `jobs()`

Returns background job implementations.

```rust
fn jobs(&self) -> Vec<Arc<dyn Job>> {
    vec![
        Arc::new(CleanupJob),
        Arc::new(SyncJob),
    ]
}
```

### `has_jobs()`

Check if extension provides jobs.

---

## Configuration Methods

### `config_prefix()`

Returns the configuration namespace for this extension.

```rust
fn config_prefix(&self) -> Option<&str> {
    Some("my_extension")
}
```

Configuration is loaded from `profile.yaml` under `extensions.my_extension`.

### `config_schema()`

Returns JSON Schema for configuration validation.

```rust
fn config_schema(&self) -> Option<JsonValue> {
    Some(json!({
        "type": "object",
        "properties": {
            "enabled": { "type": "boolean" },
            "max_items": { "type": "integer", "minimum": 1 }
        }
    }))
}
```

### `validate_config()`

Custom validation for configuration values.

```rust
fn validate_config(&self, config: &JsonValue) -> Result<(), ConfigError> {
    if let Some(max) = config.get("max_items").and_then(|v| v.as_i64()) {
        if max > 10000 {
            return Err(ConfigError::InvalidValue {
                key: "max_items".into(),
                message: "Value cannot exceed 10000".into(),
            });
        }
    }
    Ok(())
}
```

### `has_config()`

Check if extension provides configuration.

---

## Provider Methods

### `llm_providers()`

Returns LLM provider implementations.

```rust
fn llm_providers(&self) -> Vec<Arc<dyn LlmProvider>> {
    vec![Arc::new(OpenAIProvider::new(self.api_key.clone()))]
}
```

### `tool_providers()`

Returns MCP tool provider implementations.

```rust
fn tool_providers(&self) -> Vec<Arc<dyn ToolProvider>> {
    vec![Arc::new(DatabaseToolProvider::new())]
}
```

### `has_llm_providers()`, `has_tool_providers()`

Check if extension provides these.

---

## Template Methods

### `template_providers()`

Returns template definition providers.

```rust
fn template_providers(&self) -> Vec<Arc<dyn TemplateProvider>> {
    vec![Arc::new(NavigationTemplateProvider)]
}
```

### `has_template_providers()`

Check if extension provides template definitions.

---

## Web Rendering Methods

### `page_data_providers()`

Returns providers that supply template variables.

```rust
fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
    vec![
        Arc::new(ContentPageDataProvider),
        Arc::new(MetadataProvider),
    ]
}
```

### `component_renderers()`

Returns HTML fragment generators.

```rust
fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
    vec![
        Arc::new(CardRenderer),
        Arc::new(NavigationRenderer),
    ]
}
```

### `template_data_extenders()`

Returns providers that modify template data after rendering.

```rust
fn template_data_extenders(&self) -> Vec<Arc<dyn TemplateDataExtender>> {
    vec![Arc::new(CanonicalUrlExtender)]
}
```

### `page_prerenderers()`

Returns static page generators.

```rust
fn page_prerenderers(&self) -> Vec<Arc<dyn PagePrerenderer>> {
    vec![
        Arc::new(HomepagePrerenderer),
        Arc::new(BlogListPrerenderer),
    ]
}
```

### `content_data_providers()`

Returns content enrichment providers.

```rust
fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
    vec![Arc::new(RelatedContentProvider)]
}
```

### `frontmatter_processors()`

Returns frontmatter parsing providers.

```rust
fn frontmatter_processors(&self) -> Vec<Arc<dyn FrontmatterProcessor>> {
    vec![Arc::new(CustomFieldProcessor)]
}
```

### `has_*()` methods

Each provider type has a corresponding `has_*()` method.

---

## Feed Methods

### `rss_feed_providers()`

Returns RSS feed generators.

```rust
fn rss_feed_providers(&self) -> Vec<Arc<dyn RssFeedProvider>> {
    vec![Arc::new(BlogRssProvider)]
}
```

### `sitemap_providers()`

Returns sitemap entry generators.

```rust
fn sitemap_providers(&self) -> Vec<Arc<dyn SitemapProvider>> {
    vec![Arc::new(ContentSitemapProvider)]
}
```

### `has_rss_feed_providers()`, `has_sitemap_providers()`

Check if extension provides these.

---

## Asset Methods

### `declares_assets()`

Returns true if extension declares static assets.

```rust
fn declares_assets(&self) -> bool {
    true
}
```

### `required_assets()`

Returns CSS, JS, font, and image asset declarations.

```rust
fn required_assets(&self, paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(
            paths.storage_files().join("css/main.css"),
            "css/main.css"
        ),
        AssetDefinition::js(
            paths.storage_files().join("js/app.js"),
            "js/app.js"
        ),
    ]
}
```

---

## Role Methods

### `roles()`

Returns RBAC role definitions.

```rust
fn roles(&self) -> Vec<ExtensionRole> {
    vec![
        ExtensionRole::new("editor", "Editor", "Can edit content")
            .with_permissions(vec!["content.edit".into()]),
        ExtensionRole::new("admin", "Administrator", "Full access")
            .with_permissions(vec!["*".into()]),
    ]
}
```

### `has_roles()`

Check if extension defines roles.

---

## Storage Methods

### `required_storage_paths()`

Returns paths that must exist for the extension to function.

```rust
fn required_storage_paths(&self) -> Vec<&'static str> {
    vec!["uploads", "cache/thumbnails"]
}
```

### `has_storage_paths()`

Check if extension requires storage paths.

---

## Complete Example

```rust
use systemprompt::extension::prelude::*;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct MyExtension;

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "my-extension",
            name: "My Extension",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn priority(&self) -> u32 {
        50
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["users"]
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::inline("my_table", include_str!("../schema/001_table.sql")),
        ]
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        let db = ctx.database();
        let pool = db.as_any().downcast_ref::<Database>()?.pool()?;
        Some(ExtensionRouter::new(crate::api::router(pool), "/api/v1/my"))
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(CleanupJob)]
    }

    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        vec![Arc::new(MyDataProvider)]
    }

    fn declares_assets(&self) -> bool {
        true
    }

    fn required_assets(&self, paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
        vec![
            AssetDefinition::css(paths.storage_files().join("css/my.css"), "css/my.css"),
        ]
    }
}

register_extension!(MyExtension);
```