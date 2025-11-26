use anyhow::{anyhow, Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

/// Deletes a module configuration from the database.
///
/// # Arguments
///
/// * `db` - Database provider for executing queries
/// * `module_name` - Name of the module to delete
///
/// # Returns
///
/// * `Ok(())` - Module successfully deleted
/// * `Err` - Database operation failed or validation error
///
/// # Validation
///
/// * Module name must be non-empty
///
/// # Note
///
/// This operation is permanent and cannot be undone. Ensure the module
/// is not in use before deletion.
pub async fn delete_module(db: &dyn DatabaseProvider, module_name: &str) -> Result<()> {
    if module_name.trim().is_empty() {
        return Err(anyhow!("Module name cannot be empty"));
    }

    let query = DatabaseQueryEnum::DeleteModule.get(db);
    db.execute(&query, &[&module_name]).await.context(format!(
        "Failed to delete module '{module_name}'. Ensure the module exists and is not in use"
    ))?;
    Ok(())
}
