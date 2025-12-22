# Template Refactor Plan

Changes to `systemprompt-template` repository to improve architecture and idiomatic Rust patterns.

---

## 1. Relocate MCP Servers to Extensions

**Current Location**: `services/mcp/`
**New Location**: `extensions/mcp/`

### Files to Move

```bash
# Move MCP servers from services to extensions
mv services/mcp/systemprompt-admin extensions/mcp/admin
mv services/mcp/system-tools extensions/mcp/system-tools
mv services/mcp/systemprompt-infrastructure extensions/mcp/infrastructure
```

### Update Cargo.toml

```toml
# Root Cargo.toml - update workspace members
[workspace]
members = [
    "extensions/blog",
    "extensions/mcp/admin",
    "extensions/mcp/system-tools",
    "extensions/mcp/infrastructure",
]
```

### Update Justfile

Update `mcp-build-submodules` recipe to point to new locations.

---

## 2. Implement Extension Trait for BlogExtension

**File**: `extensions/blog/src/extension.rs`

### Current (inherent methods)

```rust
impl BlogExtension {
    pub const fn id() -> &'static str { "blog" }
    pub fn schemas() -> Vec<(&'static str, &'static str)> { ... }
    pub fn router(&self, pool: Arc<PgPool>, config: BlogConfig) -> Router { ... }
}
```

### Refactored (trait implementation)

```rust
use systemprompt_traits::{Extension, ExtensionContext, ExtensionMetadata, SchemaDefinition};

impl Extension for BlogExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "blog",
            name: "Blog & Content Management",
            version: env!("CARGO_PKG_VERSION"),
            priority: 100,
            dependencies: vec![],
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::inline("001_markdown_content", SCHEMA_MARKDOWN_CONTENT),
            SchemaDefinition::inline("002_markdown_categories", SCHEMA_MARKDOWN_CATEGORIES),
            // ... remaining schemas
        ]
    }

    fn router(&self, ctx: &ExtensionContext) -> Option<Router> {
        let pool = ctx.database().postgres_pool()?;
        let config = ctx.config().get::<BlogConfig>("blog")?;
        Some(api::router(pool, config))
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(ContentIngestionJob)]
    }
}

// Single registration replaces multiple macros
register_extension!(BlogExtension);
```

---

## 3. Reduce Repository SQL Repetition

**File**: `extensions/blog/src/repository/content.rs`

### Add Column Constant

```rust
impl Content {
    /// SQL column list for SELECT queries with type annotations
    pub const COLUMNS: &'static str = r#"
        id as "id: ContentId",
        slug,
        title,
        description,
        body,
        author,
        published_at,
        keywords,
        kind,
        image,
        category_id as "category_id: CategoryId",
        source_id as "source_id: SourceId",
        version_hash,
        COALESCE(links, '[]'::jsonb) as "links!",
        updated_at
    "#;
}
```

### Refactor Repository Methods

```rust
impl ContentRepository {
    pub async fn get_by_id(&self, id: &ContentId) -> Result<Option<Content>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM markdown_content WHERE id = $1",
            Content::COLUMNS
        );
        sqlx::query_as::<_, Content>(&query)
            .bind(id.as_str())
            .fetch_optional(&*self.pool)
            .await
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM markdown_content WHERE slug = $1",
            Content::COLUMNS
        );
        sqlx::query_as::<_, Content>(&query)
            .bind(slug)
            .fetch_optional(&*self.pool)
            .await
    }

    // Apply same pattern to: list, list_by_source, get_by_source_and_slug, update
}
```

---

## 4. Standardize Error Types

**File**: `extensions/blog/src/error.rs`

### Implement ExtensionError Trait

```rust
use systemprompt_traits::ExtensionError;
use axum::http::StatusCode;

#[derive(Error, Debug)]
pub enum BlogError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Content not found: {0}")]
    ContentNotFound(String),

    #[error("Link not found: {0}")]
    LinkNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Validation error: {0}")]
    Validation(String),

    // ... other variants
}

impl ExtensionError for BlogError {
    fn code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::ContentNotFound(_) => "CONTENT_NOT_FOUND",
            Self::LinkNotFound(_) => "LINK_NOT_FOUND",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::Validation(_) => "VALIDATION_ERROR",
            // ...
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::ContentNotFound(_) | Self::LinkNotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // ...
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::Database(_))
    }
}
```

---

## 5. Update Scheduler Config to Reference Extensions

**File**: `services/scheduler/config.yml`

### Current

```yaml
scheduler:
  enabled: true
  jobs:
    - name: cleanup_anonymous_users
      enabled: true
      schedule: "0 0 * * * *"
```

### Refactored

```yaml
scheduler:
  enabled: true
  jobs:
    # Core jobs (defined in systemprompt-core)
    - extension: core
      job: cleanup_anonymous_users
      schedule: "0 0 * * * *"
      enabled: true

    - extension: core
      job: cleanup_empty_contexts
      schedule: "0 */6 * * * *"
      enabled: true

    # Blog extension jobs
    - extension: blog
      job: content_ingestion
      schedule: "0 * * * * *"  # Override default hourly
      enabled: true
```

---

## 6. Add Build-Time Config Validation

**File**: `build.rs` (new file in root)

```rust
use std::fs;

fn main() {
    // Validate agent configs at compile time
    validate_yaml::<AgentConfig>("services/agents/assistant.yml");
    validate_yaml::<AgentConfig>("services/agents/admin.yml");

    // Validate scheduler config
    validate_yaml::<SchedulerConfig>("services/scheduler/config.yml");

    // Validate web config
    validate_yaml::<WebConfig>("services/web/config.yml");
}

fn validate_yaml<T: serde::de::DeserializeOwned>(path: &str) {
    println!("cargo:rerun-if-changed={}", path);
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
    let _: T = serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("Invalid config {}: {}", path, e));
}
```

---

## 7. Update Directory Structure

### Final Layout

```
systemprompt-template/
├── core/                          # READ-ONLY submodule
├── extensions/
│   ├── blog/                      # Existing
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── extension.rs       # Implements Extension trait
│   │   │   ├── error.rs           # Implements ExtensionError
│   │   │   └── ...
│   │   └── Cargo.toml
│   └── mcp/                       # NEW - moved from services/mcp
│       ├── admin/
│       ├── system-tools/
│       └── infrastructure/
├── services/                      # Config only - no .rs files
│   ├── agents/
│   ├── ai/
│   ├── content/
│   ├── scheduler/
│   ├── skills/
│   └── web/
├── config/
├── infrastructure/
├── build.rs                       # NEW - config validation
└── Cargo.toml                     # Updated workspace members
```

---

## Implementation Order

1. **Move MCP servers** - Structural change, do first
2. **Update Cargo.toml** - Fix workspace after move
3. **Implement Extension trait** - BlogExtension upgrade
4. **Add COLUMNS constant** - Repository DRY improvement
5. **Implement ExtensionError** - Error standardization
6. **Update scheduler config** - Reference-based jobs
7. **Add build.rs** - Compile-time validation

---

## Files Modified

| File | Change Type |
|------|-------------|
| `Cargo.toml` | Modify workspace members |
| `justfile` | Update MCP build paths |
| `extensions/blog/src/extension.rs` | Implement Extension trait |
| `extensions/blog/src/error.rs` | Implement ExtensionError trait |
| `extensions/blog/src/models/content.rs` | Add COLUMNS constant |
| `extensions/blog/src/repository/content.rs` | Use COLUMNS constant |
| `services/scheduler/config.yml` | Reference-based job config |
| `build.rs` | New file - config validation |
| `services/mcp/*` | Move to `extensions/mcp/` |
