use super::VariablesRepository;
use crate::models::variables::Variable;
use anyhow::{Context, Result};
use systemprompt_core_database::DatabaseQueryEnum;

#[derive(Debug)]
pub struct UpdateVariable {
    pub value: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_secret: Option<bool>,
    pub is_required: Option<bool>,
    pub default_value: Option<String>,
}

impl VariablesRepository {
    pub async fn update(&self, id: i32, data: UpdateVariable) -> Result<Option<Variable>> {
        let query = DatabaseQueryEnum::UpdateVariable.get(self.db.as_ref());

        let rows_affected = self
            .db
            .execute(
                &query,
                &[
                    &data.value,
                    &data.description,
                    &data.category,
                    &data.is_secret,
                    &data.is_required,
                    &data.default_value,
                    &id,
                ],
            )
            .await
            .context(format!("Failed to update variable with id {id}"))?;

        if rows_affected == 0 {
            return Ok(None);
        }

        self.get(id).await
    }
}
