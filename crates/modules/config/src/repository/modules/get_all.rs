use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, JsonRow};

#[derive(Debug, Clone)]
pub struct DatabaseModule {
    pub name: String,
    pub version: String,
    pub display_name: String,
    pub description: Option<String>,
    pub weight: Option<i32>,
    pub schemas: Option<String>,
    pub seeds: Option<String>,
    pub permissions: Option<String>,
    pub enabled: bool,
}

impl DatabaseModule {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing name"))?
            .to_string();

        let version = row
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing version"))?
            .to_string();

        let display_name = row
            .get("display_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing display_name"))?
            .to_string();

        let description = row
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        let weight = row.get("weight").and_then(serde_json::Value::as_i64).map(|i| i as i32);

        let schemas = row
            .get("schemas")
            .and_then(|v| v.as_str())
            .map(String::from);

        let seeds = row.get("seeds").and_then(|v| v.as_str()).map(String::from);

        let permissions = row
            .get("permissions")
            .and_then(|v| v.as_str())
            .map(String::from);

        let enabled = row
            .get("enabled")
            .and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|i| i != 0)))
            .ok_or_else(|| anyhow!("Missing enabled"))?;

        Ok(Self {
            name,
            version,
            display_name,
            description,
            weight,
            schemas,
            seeds,
            permissions,
            enabled,
        })
    }
}

pub async fn get_all_modules(db: &dyn DatabaseProvider) -> Result<Vec<DatabaseModule>> {
    let query = DatabaseQueryEnum::GetAllModules.get(db);
    let rows = db
        .fetch_all(&query, &[])
        .await
        .context("Failed to fetch all modules")?;

    rows.iter()
        .map(DatabaseModule::from_json_row)
        .collect::<Result<Vec<_>>>()
}
