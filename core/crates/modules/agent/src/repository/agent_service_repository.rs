use anyhow::{Context, Result};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug)]
pub struct AgentServiceRow {
    pub name: String,
    pub pid: Option<i32>,
    pub port: i32,
    pub status: String,
}

#[derive(Debug)]
pub struct AgentServerIdRow {
    pub name: String,
}

#[derive(Debug)]
pub struct AgentServerIdPidRow {
    pub name: String,
    pub pid: i32,
}

#[derive(Debug, Clone)]
pub struct AgentServiceRepository {
    db_pool: DbPool,
}

impl RepositoryTrait for AgentServiceRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl AgentServiceRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    fn get_pg_pool(&self) -> Result<Arc<PgPool>> {
        self.db_pool
            .as_ref()
            .get_postgres_pool()
            .context("PostgreSQL pool not available")
    }

    pub async fn register_agent(
        &self,
        name: &str,
        pid: u32,
        port: u16,
        _auth: &str,
    ) -> Result<String, RepositoryError> {
        self.remove_agent_service(name).await?;

        let pool = self.get_pg_pool().map_err(RepositoryError::GenericError)?;
        let pid_i32 = pid as i32;
        let port_i32 = i32::from(port);

        sqlx::query!(
            "INSERT INTO services (name, module_name, pid, port, status, updated_at)
             VALUES ($1, 'agent', $2, $3, 'running', CURRENT_TIMESTAMP)
             ON CONFLICT (name) DO UPDATE SET pid = $2, port = $3, status = 'running', updated_at \
             = CURRENT_TIMESTAMP",
            name,
            pid_i32,
            port_i32
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to register agent")
        .map_err(RepositoryError::GenericError)?;

        Ok(name.to_string())
    }

    pub async fn get_agent_status(
        &self,
        agent_name: &str,
    ) -> Result<Option<AgentServiceRow>, RepositoryError> {
        let pool = self.get_pg_pool().map_err(RepositoryError::GenericError)?;

        let row = sqlx::query!(
            "SELECT name, pid, port, status FROM services WHERE name = $1",
            agent_name
        )
        .fetch_optional(pool.as_ref())
        .await
        .context("Failed to get agent status")
        .map_err(RepositoryError::GenericError)?;

        Ok(row.map(|r| AgentServiceRow {
            name: r.name,
            pid: r.pid,
            port: r.port,
            status: r.status,
        }))
    }

    pub async fn mark_crashed(&self, agent_name: &str) -> Result<(), RepositoryError> {
        let pool = self.get_pg_pool().map_err(RepositoryError::GenericError)?;

        sqlx::query!(
            "UPDATE services SET status = 'error', pid = NULL, updated_at = CURRENT_TIMESTAMP \
             WHERE name = $1",
            agent_name
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to mark agent as crashed")
        .map_err(RepositoryError::GenericError)?;

        Ok(())
    }

    pub async fn mark_stopped(&self, agent_name: &str) -> Result<(), RepositoryError> {
        let pool = self.get_pg_pool().map_err(RepositoryError::GenericError)?;

        sqlx::query!(
            "UPDATE services SET status = 'stopped', pid = NULL, updated_at = CURRENT_TIMESTAMP \
             WHERE name = $1",
            agent_name
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to mark agent as stopped")
        .map_err(RepositoryError::GenericError)?;

        Ok(())
    }

    pub async fn mark_error(
        &self,
        agent_name: &str,
        _error_message: &str,
    ) -> Result<(), RepositoryError> {
        let pool = self.get_pg_pool().map_err(RepositoryError::GenericError)?;

        sqlx::query!(
            "UPDATE services SET status = 'error', pid = NULL, updated_at = CURRENT_TIMESTAMP \
             WHERE name = $1",
            agent_name
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to mark agent with error")
        .map_err(RepositoryError::GenericError)?;

        Ok(())
    }

    pub async fn list_running_agents(&self) -> Result<Vec<AgentServerIdRow>, RepositoryError> {
        let pool = self.get_pg_pool().map_err(RepositoryError::GenericError)?;

        let rows = sqlx::query!("SELECT name FROM services WHERE status = 'running'")
            .fetch_all(pool.as_ref())
            .await
            .context("Failed to list running agents")
            .map_err(RepositoryError::GenericError)?;

        Ok(rows
            .into_iter()
            .map(|r| AgentServerIdRow { name: r.name })
            .collect())
    }

    pub async fn list_running_agent_pids(
        &self,
    ) -> Result<Vec<AgentServerIdPidRow>, RepositoryError> {
        let pool = self.get_pg_pool().map_err(RepositoryError::GenericError)?;

        let rows = sqlx::query!(
            "SELECT name, pid FROM services WHERE status = 'running' AND pid IS NOT NULL"
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to list running agent PIDs")
        .map_err(RepositoryError::GenericError)?;

        Ok(rows
            .into_iter()
            .filter_map(|r| r.pid.map(|pid| AgentServerIdPidRow { name: r.name, pid }))
            .collect())
    }

    pub async fn remove_agent_service(&self, agent_name: &str) -> Result<(), RepositoryError> {
        let pool = self.get_pg_pool().map_err(RepositoryError::GenericError)?;

        sqlx::query!("DELETE FROM services WHERE name = $1", agent_name)
            .execute(pool.as_ref())
            .await
            .context("Failed to remove agent service")
            .map_err(RepositoryError::GenericError)?;

        Ok(())
    }

    pub async fn update_health_status(
        &self,
        agent_name: &str,
        health_status: &str,
    ) -> Result<(), RepositoryError> {
        let pool = self.get_pg_pool().map_err(RepositoryError::GenericError)?;

        sqlx::query!(
            "UPDATE services SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE name = $2",
            health_status,
            agent_name
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to update agent health status")
        .map_err(RepositoryError::GenericError)?;

        Ok(())
    }
}
