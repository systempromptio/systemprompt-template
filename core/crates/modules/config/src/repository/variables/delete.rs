use super::VariablesRepository;
use anyhow::{Context, Result};
use systemprompt_core_database::DatabaseQueryEnum;

impl VariablesRepository {
    pub async fn delete(&self, id: i32) -> Result<bool> {
        let query = DatabaseQueryEnum::DeleteVariable.get(self.db.as_ref());
        let rows_affected = self
            .db
            .execute(&query, &[&id])
            .await
            .context(format!("Failed to delete variable with id {id}"))?;
        Ok(rows_affected > 0)
    }
}
