---
title: "Extension Initialization"
description: "How extensions integrate with AppContext during runtime startup."
author: "SystemPrompt Team"
slug: "extensions/lifecycle/initialization"
keywords: "initialization, appcontext, startup, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Extension Initialization

After discovery, extensions integrate with `AppContext` during runtime startup.

## Startup Sequence

```
1. ProfileBootstrap::init()
   |
2. SecretsBootstrap::init()
   |
3. CredentialsBootstrap::init()
   |
4. Config::init()
   |
5. ExtensionRegistry::discover()
   |
6. ExtensionRegistry::validate()
   |
7. AppContext::new(registry)
   |
8. Schema Installation
   |
9. Migration Execution
   |
10. Router Mounting
   |
11. Job Registration
   |
12. Server Start
```

## AppContext Integration

```rust
pub struct AppContext {
    config: Arc<Config>,
    db_pool: Arc<PgPool>,
    extension_registry: Arc<ExtensionRegistry>,
    // ...
}

impl AppContext {
    pub async fn new() -> Result<Self> {
        let registry = ExtensionRegistry::discover();
        registry.validate()?;

        Self::new_with_registry(registry).await
    }

    pub async fn new_with_registry(registry: ExtensionRegistry) -> Result<Self> {
        let config = Config::get();
        let db_pool = create_pool(&config.database).await?;

        // Install schemas
        for ext in registry.iter() {
            if ext.has_schemas() {
                install_schemas(&db_pool, ext.schemas()).await?;
            }
        }

        // Run migrations
        for ext in registry.iter() {
            if ext.has_migrations() {
                run_migrations(&db_pool, ext.migrations()).await?;
            }
        }

        Ok(Self {
            config: Arc::new(config),
            db_pool: Arc::new(db_pool),
            extension_registry: Arc::new(registry),
        })
    }
}
```

## ExtensionContext

Extensions access runtime services via `ExtensionContext`:

```rust
pub trait ExtensionContext: Send + Sync {
    fn config(&self) -> Arc<dyn ConfigProvider>;
    fn database(&self) -> Arc<dyn DatabaseHandle>;
    fn get_extension(&self, id: &str) -> Option<Arc<dyn Extension>>;
    fn has_extension(&self, id: &str) -> bool;
}

impl ExtensionContext for AppContext {
    fn config(&self) -> Arc<dyn ConfigProvider> {
        self.config.clone()
    }

    fn database(&self) -> Arc<dyn DatabaseHandle> {
        self.db_pool.clone()
    }

    fn get_extension(&self, id: &str) -> Option<Arc<dyn Extension>> {
        self.extension_registry.get(id).cloned()
    }

    fn has_extension(&self, id: &str) -> bool {
        self.extension_registry.get(id).is_some()
    }
}
```

## Schema Installation

Schemas execute in `migration_weight()` order:

```rust
async fn install_schemas(pool: &PgPool, schemas: Vec<SchemaDefinition>) -> Result<()> {
    for schema in schemas {
        let sql = match &schema.sql {
            SchemaSource::Inline(sql) => sql.clone(),
            SchemaSource::File(path) => std::fs::read_to_string(path)?,
        };

        sqlx::raw_sql(&sql).execute(pool).await?;

        // Validate required columns
        for column in &schema.required_columns {
            validate_column_exists(pool, &schema.table, column).await?;
        }
    }
    Ok(())
}
```

## Router Mounting

After context creation, routers mount to the server:

```rust
async fn build_router(ctx: Arc<AppContext>) -> Router {
    let mut router = Router::new();

    for ext in ctx.extension_registry.iter() {
        if let Some(ext_router) = ext.router(&*ctx) {
            router = router.nest(ext_router.base_path, ext_router.router);
        }
    }

    router.with_state(ctx)
}
```

## Job Registration

Jobs register with the scheduler:

```rust
async fn register_jobs(ctx: &AppContext, scheduler: &Scheduler) {
    for ext in ctx.extension_registry.iter() {
        for job in ext.jobs() {
            scheduler.register(job.clone()).await;
        }
    }
}
```

## Provider Collection

Providers are collected for the generator:

```rust
fn collect_page_providers(registry: &ExtensionRegistry) -> Vec<Arc<dyn PageDataProvider>> {
    registry.iter()
        .flat_map(|ext| ext.page_data_providers())
        .collect()
}

fn collect_component_renderers(registry: &ExtensionRegistry) -> Vec<Arc<dyn ComponentRenderer>> {
    registry.iter()
        .flat_map(|ext| ext.component_renderers())
        .collect()
}
```

## Error Handling

Initialization errors:

```rust
pub enum LoaderError {
    MissingDependency { extension: String, dependency: String },
    DuplicateExtension(String),
    InitializationFailed { extension: String, message: String },
    SchemaInstallationFailed { extension: String, message: String },
    MigrationFailed { extension: String, message: String },
    ConfigValidationFailed { extension: String, message: String },
    ReservedPathCollision { extension: String, path: String },
    InvalidBasePath { extension: String, path: String },
    CircularDependency { chain: String },
}
```

## Graceful Shutdown

On shutdown:

1. Stop accepting new requests
2. Wait for in-flight requests
3. Stop scheduler
4. Close database connections
5. Log final state

## Debugging

### Startup Logs

```
INFO Starting SystemPrompt
INFO Loading profile: local
INFO Discovering extensions...
INFO Found 12 extensions
INFO Validating dependencies...
INFO Installing schemas...
INFO Running migrations...
INFO Mounting routers...
INFO Registering jobs...
INFO Server listening on 0.0.0.0:8080
```

### Extension Status

```bash
systemprompt extensions status
```

### Database Status

```bash
systemprompt infra db status
```