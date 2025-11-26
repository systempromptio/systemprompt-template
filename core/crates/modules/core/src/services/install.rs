use crate::models::modules::{Module, ModuleSchema, ModuleSeed};
use crate::models::AppContext;
use crate::services::shared::{ModulePaths, SqlExecutor};
use anyhow::Result;
use systemprompt_core_database::DatabaseProvider;

pub async fn install_module(module: &Module) -> Result<()> {
    let app_context = AppContext::new().await?;
    let db = app_context.db_pool();
    install_module_with_db(module, db.as_ref()).await
}

pub async fn install_module_with_db(module: &Module, db: &dyn DatabaseProvider) -> Result<()> {
    // Step 1: Install schemas (base tables)
    install_module_schemas(module, db).await?;

    // Step 2: Insert module record (for tracking)
    insert_module_record(module).await?;

    // Step 3: Install seed data
    install_module_seeds(module, db).await?;

    Ok(())
}

async fn insert_module_record(_module: &Module) -> Result<()> {
    // Module registration is now handled by ModuleManager
    // This function is kept for backward compatibility but does nothing
    Ok(())
}

async fn install_module_schemas(module: &Module, db: &dyn DatabaseProvider) -> Result<()> {
    if let Some(schemas) = &module.schemas {
        for schema in schemas {
            // Check if table already exists before applying schema
            if check_table_exists(db, &schema.table).await.is_err() {
                // Table doesn't exist, apply schema
                apply_schema_local(module, schema, db).await?;
            }
            // If table exists, skip schema application
        }
    }

    Ok(())
}

async fn check_table_exists(db: &dyn DatabaseProvider, table_name: &str) -> Result<()> {
    const CHECK_QUERY: &str = "SELECT 1 FROM {} LIMIT 1";
    let query = CHECK_QUERY.replace("{}", table_name);
    db.fetch_optional(&query, &[]).await?;
    Ok(())
}

async fn install_module_seeds(module: &Module, db: &dyn DatabaseProvider) -> Result<()> {
    if let Some(seeds) = &module.seeds {
        for seed in seeds {
            apply_seed_local(module, seed, db).await?;
        }
    }

    Ok(())
}

async fn apply_schema_local(
    module: &Module,
    schema: &ModuleSchema,
    db: &dyn DatabaseProvider,
) -> Result<()> {
    let schema_path = ModulePaths::schema_path(module, schema)?;
    let schema_content = std::fs::read_to_string(&schema_path)?;
    SqlExecutor::execute_statements(db, &schema_content).await?;

    Ok(())
}

async fn apply_seed_local(
    module: &Module,
    seed: &ModuleSeed,
    db: &dyn DatabaseProvider,
) -> Result<()> {
    let seed_path = ModulePaths::seed_path(module, seed)?;
    let seed_content = std::fs::read_to_string(&seed_path)?;
    SqlExecutor::execute_statements(db, &seed_content).await?;

    Ok(())
}
