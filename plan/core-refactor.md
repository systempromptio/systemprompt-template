# Core Refactor Plan

Changes to `systemprompt-core` repository to enable better extension patterns and idiomatic Rust.

---

## 1. Unify Extension Trait Hierarchy

**Location**: `crates/shared/extension/src/lib.rs`

### Current (Fragmented)

```rust
// Separate traits requiring separate registrations
pub trait Extension { ... }
pub trait SchemaExtension { ... }
pub trait ApiExtension { ... }
pub trait ConfigExtension { ... }
pub trait JobExtension { ... }
pub trait ProviderExtension { ... }

// Multiple macros needed
register_extension!(MyExt);
register_schema_extension!(MyExt);
register_api_extension!(MyExt);
```

### Refactored (Unified)

```rust
/// Unified extension trait with optional capabilities via default impls
pub trait Extension: Send + Sync + 'static {
    /// Extension metadata (required)
    fn metadata(&self) -> ExtensionMetadata;

    /// Database schemas to install (optional)
    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![]
    }

    /// HTTP router for API routes (optional)
    fn router(&self, ctx: &ExtensionContext) -> Option<Router> {
        None
    }

    /// Background jobs to register (optional)
    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![]
    }

    /// Custom configuration schema (optional)
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// LLM providers (optional)
    fn llm_providers(&self) -> Vec<Arc<dyn LlmProvider>> {
        vec![]
    }

    /// Tool providers (optional)
    fn tool_providers(&self) -> Vec<Arc<dyn ToolProvider>> {
        vec![]
    }

    /// Extension dependencies (optional)
    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Initialization priority - lower runs first (optional)
    fn priority(&self) -> u32 {
        100
    }
}

/// Single registration macro discovers all capabilities
#[macro_export]
macro_rules! register_extension {
    ($ty:ty) => {
        ::inventory::submit! {
            Box::new(<$ty as Default>::default()) as Box<dyn Extension>
        }
    };
}
```

### Migration Path

Keep old traits as deprecated aliases for one release cycle:

```rust
#[deprecated(since = "0.2.0", note = "Use unified Extension trait instead")]
pub trait SchemaExtension: Extension {}

// Auto-impl for anything implementing Extension with schemas
impl<T: Extension> SchemaExtension for T {}
```

---

## 2. Add ExtensionError Trait

**Location**: `crates/shared/traits/src/error.rs` (new file)

```rust
use axum::http::StatusCode;
use rmcp::ErrorData as McpError;

/// Trait for extension error types to enable consistent error handling
pub trait ExtensionError: std::error::Error + Send + Sync + 'static {
    /// Machine-readable error code (e.g., "CONTENT_NOT_FOUND")
    fn code(&self) -> &'static str;

    /// HTTP status code for API responses
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Whether this error is transient and operation should be retried
    fn is_retryable(&self) -> bool {
        false
    }

    /// User-facing message (defaults to Display impl)
    fn user_message(&self) -> String {
        self.to_string()
    }

    /// Convert to MCP protocol error format
    fn to_mcp_error(&self) -> McpError {
        McpError {
            code: self.status().as_u16() as i32,
            message: self.user_message(),
            data: Some(serde_json::json!({
                "code": self.code(),
                "retryable": self.is_retryable(),
            })),
        }
    }

    /// Convert to API response error
    fn to_api_error(&self) -> ApiError {
        ApiError {
            code: self.code().to_string(),
            message: self.user_message(),
            status: self.status(),
        }
    }
}

/// Derive macro for ExtensionError
/// Usage: #[derive(ExtensionError)]
pub use systemprompt_derive::ExtensionError;
```

### Derive Macro

**Location**: `crates/shared/derive/src/lib.rs`

```rust
#[proc_macro_derive(ExtensionError, attributes(error_code, status, retryable))]
pub fn derive_extension_error(input: TokenStream) -> TokenStream {
    // Generate ExtensionError impl from enum variants
    // #[error_code = "NOT_FOUND"]
    // #[status = "404"]
    // #[retryable]
}
```

---

## 3. Add MCP Tool Proc Macro

**Location**: `crates/shared/mcp-derive/` (new crate)

### Macro Definition

```rust
/// Marks a struct as an MCP server
#[proc_macro_attribute]
pub fn mcp_server(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Generates ServerHandler trait impl
}

/// Marks methods as MCP tools
#[proc_macro_attribute]
pub fn mcp_tools(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Generates:
    // - Tool schema from function signature
    // - list_tools() implementation
    // - handle_tool_call() dispatch
}

/// Marks a single method as an MCP tool
#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parses description, generates input schema
}

/// Marks a parameter with description for schema
#[proc_macro_attribute]
pub fn arg(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Adds description to JSON schema
}
```

### Usage Example

```rust
use systemprompt_mcp_derive::{mcp_server, mcp_tools, tool, arg};

#[mcp_server]
pub struct AdminServer {
    db_pool: DbPool,
    logger: LogService,
}

#[mcp_tools]
impl AdminServer {
    /// Get comprehensive system dashboard
    #[tool(description = "Real-time dashboard with metrics and trends")]
    async fn dashboard(
        &self,
        #[arg(description = "Time range: 24h, 7d, or 30d", default = "24h")]
        range: TimeRange,
    ) -> Result<DashboardArtifact, ToolError> {
        dashboard::handle(&self.db_pool, range).await
    }

    /// Manage users and sessions
    #[tool(description = "User management operations")]
    async fn user(
        &self,
        #[arg(description = "Action: list, get, update_role, revoke_sessions")]
        action: UserAction,
        #[arg(description = "User ID for get/update operations", required = false)]
        user_id: Option<UserId>,
    ) -> Result<UserResponse, ToolError> {
        users::handle(&self.db_pool, action, user_id).await
    }
}
```

### Generated Code

```rust
// Auto-generated by #[mcp_tools]
impl AdminServer {
    fn tool_schemas() -> Vec<Tool> {
        vec![
            Tool {
                name: "dashboard".into(),
                description: Some("Real-time dashboard with metrics and trends".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "range": {
                            "type": "string",
                            "description": "Time range: 24h, 7d, or 30d",
                            "default": "24h"
                        }
                    }
                }).as_object().unwrap().clone()),
                ..Default::default()
            },
            // ... user tool schema
        ]
    }

    async fn dispatch_tool(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        match name {
            "dashboard" => {
                let range: TimeRange = serde_json::from_value(
                    params.get("range").cloned().unwrap_or(json!("24h"))
                )?;
                self.dashboard(range).await.map(|r| r.into())
            }
            "user" => {
                let action: UserAction = serde_json::from_value(
                    params.get("action").cloned().ok_or_else(|| /* required error */)?
                )?;
                let user_id: Option<UserId> = params.get("user_id")
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()?;
                self.user(action, user_id).await.map(|r| r.into())
            }
            _ => Err(McpError::method_not_found())
        }
    }
}

impl ServerHandler for AdminServer {
    async fn list_tools(&self, _: ListToolsRequestParam, _: RequestContext<RoleServer>)
        -> Result<ListToolsResult, McpError>
    {
        Ok(ListToolsResult {
            tools: Self::tool_schemas(),
            next_cursor: None,
        })
    }

    async fn call_tool(&self, req: CallToolRequestParam, ctx: RequestContext<RoleServer>)
        -> Result<CallToolResult, McpError>
    {
        self.dispatch_tool(&req.name, req.arguments.unwrap_or_default()).await
    }
}
```

---

## 4. Add Generic Repository Trait

**Location**: `crates/infra/database/src/repository.rs`

```rust
use sqlx::{PgPool, postgres::PgRow, FromRow};
use std::sync::Arc;

/// Trait for entities that can be stored in a repository
pub trait Entity: for<'r> FromRow<'r, PgRow> + Send + Sync + Unpin + 'static {
    /// The ID type for this entity
    type Id: EntityId;

    /// Database table name
    const TABLE: &'static str;

    /// SQL column list for SELECT queries
    const COLUMNS: &'static str;
}

/// Trait for entity ID types
pub trait EntityId: Send + Sync + 'static {
    fn as_str(&self) -> &str;
    fn from_string(s: String) -> Self;
}

/// Generic repository providing common CRUD operations
pub struct Repository<E: Entity> {
    pool: Arc<PgPool>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: Entity> Repository<E> {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn get(&self, id: &E::Id) -> Result<Option<E>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM {} WHERE id = $1",
            E::COLUMNS,
            E::TABLE
        );
        sqlx::query_as::<_, E>(&query)
            .bind(id.as_str())
            .fetch_optional(&*self.pool)
            .await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<E>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM {} ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            E::COLUMNS,
            E::TABLE
        );
        sqlx::query_as::<_, E>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await
    }

    pub async fn delete(&self, id: &E::Id) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(&format!("DELETE FROM {} WHERE id = $1", E::TABLE))
            .bind(id.as_str())
            .execute(&*self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn exists(&self, id: &E::Id) -> Result<bool, sqlx::Error> {
        let query = format!("SELECT 1 FROM {} WHERE id = $1", E::TABLE);
        let result: Option<(i32,)> = sqlx::query_as(&query)
            .bind(id.as_str())
            .fetch_optional(&*self.pool)
            .await?;
        Ok(result.is_some())
    }
}
```

### Extension Methods via Trait

```rust
/// Extension trait for custom queries
#[async_trait]
pub trait RepositoryExt<E: Entity>: Sized {
    fn pool(&self) -> &PgPool;

    async fn find_by<T: ToString + Send>(
        &self,
        column: &str,
        value: T,
    ) -> Result<Option<E>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM {} WHERE {} = $1",
            E::COLUMNS,
            E::TABLE,
            column
        );
        sqlx::query_as::<_, E>(&query)
            .bind(value.to_string())
            .fetch_optional(self.pool())
            .await
    }
}

impl<E: Entity> RepositoryExt<E> for Repository<E> {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
```

---

## 5. Add Configuration Schema Validation

**Location**: `crates/infra/config/src/validation.rs`

```rust
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

/// Validate YAML config against Rust type at build time
pub fn validate_config<T: DeserializeOwned + JsonSchema>(
    yaml_path: &str,
) -> Result<T, ConfigValidationError> {
    let content = std::fs::read_to_string(yaml_path)?;
    let config: T = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// Generate JSON schema from Rust type
pub fn generate_schema<T: JsonSchema>() -> serde_json::Value {
    schemars::schema_for!(T).into()
}

/// Build script helper for config validation
pub fn build_validate_configs(configs: &[(&str, fn(&str) -> Result<(), String>)]) {
    for (path, validator) in configs {
        println!("cargo:rerun-if-changed={}", path);
        if let Err(e) = validator(path) {
            panic!("Config validation failed for {}: {}", path, e);
        }
    }
}
```

---

## 6. Update Job Scheduler to Use References

**Location**: `crates/app/scheduler/src/lib.rs`

### Current

Jobs can be defined in YAML or Rust independently.

### Refactored

```rust
/// Job configuration from YAML
#[derive(Debug, Deserialize)]
pub struct JobConfig {
    /// Extension that provides this job
    pub extension: String,
    /// Job name within the extension
    pub job: String,
    /// Cron schedule (overrides default)
    pub schedule: Option<String>,
    /// Whether job is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Scheduler that resolves job references
impl Scheduler {
    pub fn new(
        configs: Vec<JobConfig>,
        extension_registry: &ExtensionRegistry,
    ) -> Result<Self, SchedulerError> {
        let mut jobs = Vec::new();

        for config in configs {
            let extension = extension_registry
                .get(&config.extension)
                .ok_or_else(|| SchedulerError::ExtensionNotFound(config.extension.clone()))?;

            let job = extension
                .jobs()
                .into_iter()
                .find(|j| j.name() == config.job)
                .ok_or_else(|| SchedulerError::JobNotFound {
                    extension: config.extension.clone(),
                    job: config.job.clone(),
                })?;

            if config.enabled {
                let schedule = config.schedule
                    .as_deref()
                    .unwrap_or_else(|| job.schedule());
                jobs.push(ScheduledJob::new(job, schedule)?);
            }
        }

        Ok(Self { jobs })
    }
}
```

---

## 7. Export Unified Prelude

**Location**: `systemprompt/src/prelude.rs`

```rust
//! Prelude for extension authors
//!
//! ```rust
//! use systemprompt::prelude::*;
//! ```

// Extension framework
pub use systemprompt_extension::{
    Extension,
    ExtensionContext,
    ExtensionMetadata,
    SchemaDefinition,
    register_extension,
};

// Error handling
pub use systemprompt_traits::{
    ExtensionError,
    ExtensionError as DeriveExtensionError,  // derive macro
};

// Jobs
pub use systemprompt_traits::{Job, JobContext, JobResult};

// Repository
pub use systemprompt_database::{Entity, EntityId, Repository, RepositoryExt};

// MCP tools (when feature enabled)
#[cfg(feature = "mcp")]
pub use systemprompt_mcp_derive::{mcp_server, mcp_tools, tool, arg};

// Common re-exports
pub use axum::Router;
pub use sqlx::PgPool;
pub use std::sync::Arc;
```

---

## Implementation Order

1. **ExtensionError trait** - No breaking changes, additive
2. **Generic Repository** - Additive, extensions can adopt gradually
3. **Unified Extension trait** - Breaking change, deprecate old traits
4. **MCP proc macros** - New crate, opt-in adoption
5. **Config validation** - Build script helpers
6. **Scheduler references** - Config format change

---

## New Crates

| Crate | Purpose |
|-------|---------|
| `systemprompt-derive` | ExtensionError derive macro |
| `systemprompt-mcp-derive` | MCP tool proc macros |

---

## Files Modified

| File | Change Type |
|------|-------------|
| `crates/shared/extension/src/lib.rs` | Unify Extension trait |
| `crates/shared/traits/src/lib.rs` | Add ExtensionError |
| `crates/shared/traits/src/error.rs` | New - ExtensionError trait |
| `crates/infra/database/src/repository.rs` | New - Generic repository |
| `crates/infra/config/src/validation.rs` | New - Schema validation |
| `crates/app/scheduler/src/lib.rs` | Reference-based job config |
| `systemprompt/src/prelude.rs` | Unified prelude |
| `Cargo.toml` | Add new crates to workspace |

---

## Breaking Changes

1. **Extension trait unification** - Old `SchemaExtension`, `ApiExtension`, etc. deprecated
2. **Job config format** - YAML now references extensions instead of defining jobs
3. **ExtensionContext** replaces direct pool injection in routers

### Migration Guide

```rust
// Before (0.1.x)
register_extension!(MyExtension);
register_schema_extension!(MyExtension);
register_api_extension!(MyExtension);

impl SchemaExtension for MyExtension { ... }
impl ApiExtension for MyExtension { ... }

// After (0.2.x)
register_extension!(MyExtension);

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata { ... }
    fn schemas(&self) -> Vec<SchemaDefinition> { ... }
    fn router(&self, ctx: &ExtensionContext) -> Option<Router> { ... }
}
```
