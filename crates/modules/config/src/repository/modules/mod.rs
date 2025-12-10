use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Module;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug)]
pub struct ModuleRepository {
    pool: PgPool,
}

impl RepositoryTrait for ModuleRepository {
    type Pool = PgPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }
}

impl ModuleRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        name: &str,
        version: &str,
        display_name: &str,
        description: Option<&str>,
        weight: Option<i32>,
        schemas: Option<&str>,
        seeds: Option<&str>,
        permissions: Option<&str>,
    ) -> Result<Module> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let weight = weight.unwrap_or(100);
        sqlx::query_as!(
            Module,
            r#"
            INSERT INTO modules (
                id, name, version, display_name, description,
                weight, schemas, seeds, permissions, enabled, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10, $10)
            RETURNING id, name, version, display_name, description, weight,
                      schemas, seeds, permissions, enabled, created_at, updated_at
            "#,
            id,
            name,
            version,
            display_name,
            description,
            weight,
            schemas,
            seeds,
            permissions,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to insert module: {}", e))
    }

    pub async fn get_all(&self) -> Result<Vec<Module>> {
        sqlx::query_as!(
            Module,
            r#"
            SELECT id, name, version, display_name, description, weight,
                   schemas, seeds, permissions, enabled, created_at, updated_at
            FROM modules
            ORDER BY name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch all modules: {}", e))
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<Module>> {
        sqlx::query_as!(
            Module,
            r#"
            SELECT id, name, version, display_name, description, weight,
                   schemas, seeds, permissions, enabled, created_at, updated_at
            FROM modules
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch module by name: {}", e))
    }

    pub async fn enable(&self, name: &str) -> Result<Module> {
        let now = Utc::now();
        sqlx::query_as!(
            Module,
            r#"
            UPDATE modules
            SET enabled = true, updated_at = $1
            WHERE name = $2
            RETURNING id, name, version, display_name, description, weight,
                      schemas, seeds, permissions, enabled, created_at, updated_at
            "#,
            now,
            name
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to enable module: {}", e))
    }

    pub async fn disable(&self, name: &str) -> Result<Module> {
        let now = Utc::now();
        sqlx::query_as!(
            Module,
            r#"
            UPDATE modules
            SET enabled = false, updated_at = $1
            WHERE name = $2
            RETURNING id, name, version, display_name, description, weight,
                      schemas, seeds, permissions, enabled, created_at, updated_at
            "#,
            now,
            name
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to disable module: {}", e))
    }

    pub async fn update(
        &self,
        name: &str,
        version: &str,
        display_name: &str,
        description: Option<&str>,
        weight: Option<i32>,
        schemas: Option<&str>,
        seeds: Option<&str>,
        permissions: Option<&str>,
    ) -> Result<Module> {
        let now = Utc::now();
        sqlx::query_as!(
            Module,
            r#"
            UPDATE modules
            SET version = $1, display_name = $2, description = $3,
                weight = $4, schemas = $5, seeds = $6, permissions = $7,
                updated_at = $8
            WHERE name = $9
            RETURNING id, name, version, display_name, description, weight,
                      schemas, seeds, permissions, enabled, created_at, updated_at
            "#,
            version,
            display_name,
            description,
            weight,
            schemas,
            seeds,
            permissions,
            now,
            name
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to update module: {}", e))
    }

    pub async fn delete(&self, name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(anyhow!("Module name cannot be empty"));
        }

        sqlx::query!("DELETE FROM modules WHERE name = $1", name)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to delete module: {}", e))?;
        Ok(())
    }

    pub async fn insert_module_config(
        &self,
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

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        sqlx::query!(
            r#"
            INSERT INTO modules (
                id, name, version, display_name, description,
                weight, schemas, seeds, permissions, enabled, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)
            ON CONFLICT(name) DO UPDATE SET
                version = excluded.version,
                display_name = excluded.display_name,
                description = excluded.description,
                weight = excluded.weight,
                schemas = excluded.schemas,
                seeds = excluded.seeds,
                permissions = excluded.permissions,
                updated_at = CURRENT_TIMESTAMP
            "#,
            id,
            module.name,
            module.version,
            module.display_name,
            module.description,
            module.weight,
            schemas_json,
            seeds_json,
            permissions_json,
            true,
            now
        )
        .execute(&self.pool)
        .await
        .context(format!("Failed to insert module '{}'", module.name))?;

        Ok(())
    }

    pub async fn update_module_version(
        &self,
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

        let now = Utc::now();
        sqlx::query!(
            r#"
            UPDATE modules
            SET version = $1, display_name = $2, description = $3,
                weight = $4, schemas = $5, seeds = $6, permissions = $7,
                updated_at = $8
            WHERE name = $9
            "#,
            module.version,
            module.display_name,
            module.description,
            module.weight,
            schemas_json,
            seeds_json,
            permissions_json,
            now,
            module.name
        )
        .execute(&self.pool)
        .await
        .context(format!("Failed to update module '{}'", module.name))?;

        Ok(())
    }

    pub async fn delete_module_config(&self, module_name: &str) -> Result<()> {
        if module_name.trim().is_empty() {
            return Err(anyhow!("Module name cannot be empty"));
        }

        sqlx::query!("DELETE FROM modules WHERE name = $1", module_name)
            .execute(&self.pool)
            .await
            .context(format!(
                "Failed to delete module '{module_name}'. Ensure the module exists and is not in \
                 use"
            ))?;
        Ok(())
    }

    pub async fn enable_module_config(&self, module_name: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query!(
            "UPDATE modules SET enabled = true, updated_at = $1 WHERE name = $2",
            now,
            module_name
        )
        .execute(&self.pool)
        .await
        .context(format!("Failed to enable module '{module_name}'"))?;
        Ok(())
    }

    pub async fn disable_module_config(&self, module_name: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query!(
            "UPDATE modules SET enabled = false, updated_at = $1 WHERE name = $2",
            now,
            module_name
        )
        .execute(&self.pool)
        .await
        .context(format!("Failed to disable module '{module_name}'"))?;
        Ok(())
    }
}
