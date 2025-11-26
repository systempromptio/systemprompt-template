use super::VariablesRepository;
use crate::models::variables::Variable;
use anyhow::{Context, Result};
use systemprompt_core_database::DatabaseQueryEnum;

impl VariablesRepository {
    pub async fn get(&self, id: i32) -> Result<Option<Variable>> {
        let query = DatabaseQueryEnum::GetVariableById.get(self.db.as_ref());
        let row = self
            .db
            .fetch_optional(&query, &[&id])
            .await
            .context(format!("Failed to get variable by id {id}"))?;
        row.map(|r| Variable::from_json_row(&r)).transpose()
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<Variable>> {
        let query = DatabaseQueryEnum::GetVariable.get(self.db.as_ref());
        let row = self
            .db
            .fetch_optional(&query, &[&name])
            .await
            .context(format!("Failed to get variable by name '{name}'"))?;
        row.map(|r| Variable::from_json_row(&r)).transpose()
    }
}
