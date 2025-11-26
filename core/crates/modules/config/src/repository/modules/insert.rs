use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

/// Inserts a new module configuration into the database.
///
/// # Arguments
///
/// * `db` - Database provider for executing queries
/// * `module` - Module configuration to insert
///
/// # Returns
///
/// * `Ok(())` - Module successfully inserted
/// * `Err` - Database operation failed or validation error
///
/// # Validation
///
/// * Module name must be non-empty
/// * Module name must contain only alphanumeric characters, hyphens, and underscores
///
/// # Examples
///
/// ```ignore
/// let module = Module {
///     name: "example-module".to_string(),
///     version: "1.0.0".to_string(),
///     // ... other fields
/// };
/// insert_module(&db, &module).await?;
/// ```
pub async fn insert_module(
    db: &dyn DatabaseProvider,
    module: &systemprompt_core_system::Module,
) -> Result<()> {
    if module.name.trim().is_empty() {
        return Err(anyhow!("Module name cannot be empty"));
    }

    if !module
        .name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "Module name '{}' contains invalid characters. Only alphanumeric, hyphens, and underscores allowed",
            module.name
        ));
    }
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

    let created_at = Utc::now();
    let query = DatabaseQueryEnum::InsertModule.get(db);

    db.execute(
        &query,
        &[
            &module.name,
            &module.version,
            &module.display_name,
            &module.description.as_deref(),
            &module.weight,
            &schemas_json.as_deref(),
            &seeds_json.as_deref(),
            &permissions_json.as_deref(),
            &true,
            &created_at,
        ],
    )
    .await
    .context(format!("Failed to insert module '{}'", module.name))?;

    Ok(())
}
