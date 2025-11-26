use super::VariablesRepository;
use crate::models::variables::Variable;
use anyhow::{anyhow, Context, Result};
use systemprompt_core_database::DatabaseQueryEnum;
use uuid::Uuid;

/// Request structure for creating a new configuration variable.
#[derive(Debug)]
pub struct CreateVariable {
    pub name: String,
    pub value: Option<String>,
    pub r#type: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_secret: bool,
    pub is_required: bool,
    pub default_value: Option<String>,
}

impl VariablesRepository {
    /// Creates a new configuration variable in the database.
    ///
    /// # Arguments
    ///
    /// * `data` - Configuration variable data to create
    ///
    /// # Returns
    ///
    /// * `Ok(Variable)` - The created variable
    /// * `Err` - Database operation failed or validation error
    ///
    /// # Validation
    ///
    /// * Variable name must be non-empty
    /// * Variable name must contain only alphanumeric characters, underscores, and hyphens
    /// * Variable type must be non-empty
    /// * Variable name must be unique
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let data = CreateVariable {
    ///     name: "database_url".to_string(),
    ///     value: Some("localhost:5432".to_string()),
    ///     r#type: "string".to_string(),
    ///     is_secret: false,
    ///     is_required: true,
    ///     ..Default::default()
    /// };
    /// let variable = repo.create(data).await?;
    /// ```
    pub async fn create(&self, data: CreateVariable) -> Result<Variable> {
        if data.name.trim().is_empty() {
            return Err(anyhow!("Variable name cannot be empty"));
        }

        if !data
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(anyhow!(
                "Variable name '{}' contains invalid characters. Only alphanumeric, underscores, and hyphens allowed",
                data.name
            ));
        }

        if data.r#type.trim().is_empty() {
            return Err(anyhow!("Variable type cannot be empty"));
        }
        let query = DatabaseQueryEnum::CreateVariable.get(self.db.as_ref());
        let id = Uuid::new_v4().to_string();

        self.db
            .execute(
                &query,
                &[
                    &id,
                    &data.name,
                    &data.value,
                    &data.r#type,
                    &data.description,
                    &data.category,
                    &data.is_secret,
                    &data.is_required,
                    &data.default_value,
                ],
            )
            .await
            .context(format!("Failed to create variable '{}'", data.name))?;

        let variable = self
            .get_by_name(&data.name)
            .await
            .context(format!(
                "Failed to retrieve created variable '{}'",
                data.name
            ))?
            .ok_or_else(|| anyhow::anyhow!("Variable was not found after creation"))?;

        Ok(variable)
    }
}
