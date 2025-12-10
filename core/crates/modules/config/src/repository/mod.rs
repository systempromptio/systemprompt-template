pub mod modules;
pub mod variables;

pub use modules::ModuleRepository;
pub use systemprompt_models::repository::{McpServer, ServiceConfig, ServiceRepository};
pub use variables::VariablesRepository;

use anyhow::{Context, Result};
use sqlx::PgPool;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug)]
pub struct ConfigRepository {
    pool: PgPool,
}

impl RepositoryTrait for ConfigRepository {
    type Pool = PgPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }
}

impl ConfigRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn is_config_table_available(&self) -> Result<bool> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM information_schema.tables
                WHERE table_name = 'config_variables'
            ) as "exists!"
            "#
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to check if config table exists")?;
        Ok(result)
    }

    pub async fn seed_default_modules(&self) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO modules (id, name, version, display_name, description, enabled, created_at, updated_at)
            VALUES
                (gen_random_uuid(), 'agent', '1.0.0', 'Agent', NULL, true, NOW(), NOW()),
                (gen_random_uuid(), 'ai', '1.0.0', 'AI', NULL, true, NOW(), NOW()),
                (gen_random_uuid(), 'blog', '1.0.0', 'Blog', NULL, true, NOW(), NOW()),
                (gen_random_uuid(), 'oauth', '1.0.0', 'OAuth', NULL, true, NOW(), NOW()),
                (gen_random_uuid(), 'mcp', '1.0.0', 'MCP', NULL, true, NOW(), NOW())
            ON CONFLICT (name) DO NOTHING
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to seed default modules")?;
        Ok(())
    }

    pub async fn list_configs(
        &self,
        _module_name: Option<&str>,
        _limit: Option<u32>,
        _offset: Option<u32>,
    ) -> Result<Vec<ConfigRow>> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct ConfigRow {
    pub module_name: Option<String>,
}
