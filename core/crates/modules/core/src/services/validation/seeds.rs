use anyhow::{Context, Result};

use crate::models::modules::{Module, ModuleSeed};
use crate::services::shared::{ModulePaths, SqlExecutor};
use systemprompt_core_database::DatabaseProvider;

pub async fn validate_module_seeds(module: &Module) -> Result<Vec<ModuleSeed>> {
    let Some(seeds) = &module.seeds else {
        return Ok(Vec::new());
    };

    let app_context = crate::AppContext::new().await.unwrap();
    let db = app_context.db_pool();
    let mut missing_seeds = Vec::new();

    for seed in seeds {
        if !is_seed_applied(db.as_ref(), seed).await? {
            missing_seeds.push(seed.clone());
        }
    }

    Ok(missing_seeds)
}

async fn is_seed_applied(db: &dyn DatabaseProvider, seed: &ModuleSeed) -> Result<bool> {
    let query = build_seed_check_query(seed);

    let result = db
        .fetch_optional(&query, &[&seed.check_value.as_str()])
        .await
        .with_context(|| format!("Failed to check seed for table '{}'", seed.table))?;

    Ok(result.is_some())
}

fn build_seed_check_query(seed: &ModuleSeed) -> String {
    format!(
        "SELECT {} FROM {} WHERE {} = ? LIMIT 1",
        seed.check_column, seed.table, seed.check_column
    )
}

pub async fn apply_seed(module: &Module, seed: &ModuleSeed) -> Result<()> {
    let seed_path = ModulePaths::seed_path(module, seed)?;
    let seed_content = std::fs::read_to_string(&seed_path)
        .with_context(|| format!("Failed to read seed file: {}", seed_path.display()))?;

    let app_context = crate::AppContext::new().await.unwrap();
    let db = app_context.db_pool();
    SqlExecutor::execute_statements(db.as_ref(), &seed_content).await?;

    Ok(())
}
