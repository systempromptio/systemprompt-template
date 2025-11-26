use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

pub async fn disable_module(db: &dyn DatabaseProvider, module_name: &str) -> Result<()> {
    let query = DatabaseQueryEnum::DisableModule.get(db);
    db.execute(&query, &[&module_name])
        .await
        .context(format!("Failed to disable module '{module_name}'"))?;
    Ok(())
}
