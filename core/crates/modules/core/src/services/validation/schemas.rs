use anyhow::{Context, Result};

use crate::models::modules::{Module, ModuleSchema};
use crate::services::shared::{ModulePaths, SqlExecutor};
use systemprompt_core_database::{Database, DatabaseProvider};

pub async fn validate_module_schemas(module: &Module) -> Result<Vec<ModuleSchema>> {
    let Some(schemas) = &module.schemas else {
        return Ok(Vec::new());
    };

    let app_context = crate::AppContext::new().await.unwrap();
    let db = app_context.db_pool();
    let mut missing_schemas = Vec::new();

    for schema in schemas {
        if !is_schema_applied(db.as_ref(), schema).await? {
            missing_schemas.push(schema.clone());
        }
    }

    Ok(missing_schemas)
}

async fn is_schema_applied(db: &Database, schema: &ModuleSchema) -> Result<bool> {
    if !table_exists(db, &schema.table).await? {
        return Ok(false);
    }

    validate_required_columns(db, schema).await
}

async fn table_exists(db: &Database, table_name: &str) -> Result<bool> {
    let query = "SELECT name FROM sqlite_master WHERE type='table' AND name = ?";
    let row = db.fetch_optional(&query, &[&table_name]).await?;

    Ok(row.is_some())
}

async fn validate_required_columns(db: &Database, schema: &ModuleSchema) -> Result<bool> {
    let existing_columns = get_table_columns(db, &schema.table).await?;

    for required_col in &schema.required_columns {
        if !existing_columns.contains(required_col) {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn get_table_columns(db: &Database, table_name: &str) -> Result<Vec<String>> {
    let query = format!("PRAGMA table_info({table_name})");
    let rows = db.fetch_all(&query, &[]).await?;

    let columns = rows
        .iter()
        .filter_map(|row| {
            row.get("name")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
        })
        .collect();

    Ok(columns)
}

pub async fn apply_schema(module: &Module, schema: &ModuleSchema) -> Result<()> {
    let schema_path = ModulePaths::schema_path(module, schema)?;
    let schema_content = std::fs::read_to_string(&schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

    let app_context = crate::AppContext::new().await.unwrap();
    let db = app_context.db_pool();
    SqlExecutor::execute_statements(db.as_ref(), &schema_content).await?;

    Ok(())
}
