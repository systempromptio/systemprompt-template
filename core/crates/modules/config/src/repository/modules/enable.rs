use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

pub async fn enable_module(db: &dyn DatabaseProvider, module_name: &str) -> Result<()> {
    let query = DatabaseQueryEnum::EnableModule.get(db);
    db.execute(&query, &[&module_name])
        .await
        .context(format!("Failed to enable module '{module_name}'"))?;
    Ok(())
}
