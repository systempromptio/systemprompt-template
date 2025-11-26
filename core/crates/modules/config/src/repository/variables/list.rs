use super::VariablesRepository;
use crate::models::variables::Variable;
use anyhow::{Context, Result};
use systemprompt_core_database::DatabaseQueryEnum;

impl VariablesRepository {
    /// Retrieves configuration variables, optionally filtered by category.
    ///
    /// # Arguments
    ///
    /// * `category` - Optional category filter. If `None`, returns all variables.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Variable>)` - List of variables matching the criteria
    /// * `Err` - Database operation failed
    ///
    /// # Query Selection
    ///
    /// This method intelligently selects the appropriate query based on input:
    /// * When `category` is `Some`: Uses `ListVariablesByCategory` query with category parameter
    /// * When `category` is `None`: Uses `ListVariables` query without category filter
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Get all variables
    /// let all_vars = repo.list(None).await?;
    ///
    /// // Get variables in specific category
    /// let db_vars = repo.list(Some("database")).await?;
    /// ```
    pub async fn list(&self, category: Option<&str>) -> Result<Vec<Variable>> {
        let rows = if let Some(cat) = category {
            let query = DatabaseQueryEnum::ListVariablesByCategory.get(self.db.as_ref());
            self.db
                .fetch_all(&query, &[&cat])
                .await
                .context(format!("Failed to list variables by category '{cat}'"))?
        } else {
            let query = DatabaseQueryEnum::ListVariables.get(self.db.as_ref());
            self.db
                .fetch_all(&query, &[])
                .await
                .context("Failed to list all variables")?
        };

        rows.into_iter()
            .map(|row| Variable::from_json_row(&row))
            .collect()
    }
}
