use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

pub async fn update_module(
    db: &dyn DatabaseProvider,
    module: &systemprompt_core_system::Module,
) -> Result<()> {
    let schemas_json = module
        .schemas
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .context("Failed to serialize module schemas")?;
    let seeds_json = module
        .seeds
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .context("Failed to serialize module seeds")?;
    let permissions_json = module
        .permissions
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .context("Failed to serialize module permissions")?;

    let query = DatabaseQueryEnum::UpdateModule.get(db);

    db.execute(
        &query,
        &[
            &module.version,
            &module.display_name,
            &module.description.as_deref(),
            &module.weight,
            &schemas_json.as_deref(),
            &seeds_json.as_deref(),
            &permissions_json.as_deref(),
            &module.name,
        ],
    )
    .await
    .context(format!("Failed to update module '{}'", module.name))?;

    Ok(())
}
