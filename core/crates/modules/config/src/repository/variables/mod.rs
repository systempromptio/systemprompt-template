use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::variables::ConfigVariable;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug, Clone)]
pub struct VariablesRepository {
    pool: PgPool,
}

impl RepositoryTrait for VariablesRepository {
    type Pool = PgPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }
}

impl VariablesRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        name: &str,
        value: Option<&str>,
        variable_type: &str,
        category: Option<&str>,
        description: Option<&str>,
        is_secret: Option<bool>,
        is_required: Option<bool>,
        default_value: Option<&str>,
    ) -> Result<ConfigVariable> {
        if name.trim().is_empty() {
            return Err(anyhow!("Variable name cannot be empty"));
        }

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(anyhow!(
                "Variable name '{}' contains invalid characters. Only alphanumeric, underscores, \
                 and hyphens allowed",
                name
            ));
        }

        let id = Uuid::new_v4().to_string();
        let cat = category.unwrap_or("system");
        let is_req = is_required.unwrap_or(true);
        let now = Utc::now();

        sqlx::query_as!(
            ConfigVariable,
            r#"
            INSERT INTO variables (id, name, value, type, category, description, is_secret, is_required, default_value, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
            RETURNING id, name, value, type as "variable_type", description, category, is_secret, is_required, default_value, created_at, updated_at
            "#,
            id,
            name,
            value,
            variable_type,
            cat,
            description,
            is_secret,
            is_req,
            default_value,
            now
        )
        .fetch_one(&self.pool)
        .await
        .context(format!("Failed to create variable '{}'", name))
    }

    pub async fn get(&self, id: &str) -> Result<Option<ConfigVariable>> {
        sqlx::query_as!(
            ConfigVariable,
            r#"
            SELECT id, name, value, type as "variable_type", description, category, is_secret, is_required, default_value, created_at, updated_at
            FROM variables
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context(format!("Failed to get variable by id {id}"))
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<ConfigVariable>> {
        sqlx::query_as!(
            ConfigVariable,
            r#"
            SELECT id, name, value, type as "variable_type", description, category, is_secret, is_required, default_value, created_at, updated_at
            FROM variables
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
        .context(format!("Failed to get variable by name '{name}'"))
    }

    pub async fn list(&self) -> Result<Vec<ConfigVariable>> {
        sqlx::query_as!(
            ConfigVariable,
            r#"
            SELECT id, name, value, type as "variable_type", description, category, is_secret, is_required, default_value, created_at, updated_at
            FROM variables
            ORDER BY category, name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list all variables")
    }

    pub async fn list_by_category(&self, category: &str) -> Result<Vec<ConfigVariable>> {
        sqlx::query_as!(
            ConfigVariable,
            r#"
            SELECT id, name, value, type as "variable_type", description, category, is_secret, is_required, default_value, created_at, updated_at
            FROM variables
            WHERE category = $1
            ORDER BY name ASC
            "#,
            category
        )
        .fetch_all(&self.pool)
        .await
        .context(format!("Failed to list variables by category '{category}'"))
    }

    pub async fn update(&self, name: &str, value: Option<&str>) -> Result<ConfigVariable> {
        let now = Utc::now();
        sqlx::query_as!(
            ConfigVariable,
            r#"
            UPDATE variables
            SET value = $1, updated_at = $2
            WHERE name = $3
            RETURNING id, name, value, type as "variable_type", description, category, is_secret, is_required, default_value, created_at, updated_at
            "#,
            value,
            now,
            name
        )
        .fetch_one(&self.pool)
        .await
        .context(format!("Failed to update variable '{name}'"))
    }

    pub async fn delete(&self, id: &str) -> Result<bool> {
        let rows_affected = sqlx::query!("DELETE FROM variables WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .context(format!("Failed to delete variable with id {id}"))?
            .rows_affected();
        Ok(rows_affected > 0)
    }
}
