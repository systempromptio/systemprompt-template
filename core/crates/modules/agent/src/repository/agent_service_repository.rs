use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool, JsonRow};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug)]
pub struct AgentServiceRow {
    pub name: String,
    pub pid: Option<i32>,
    pub port: i32,
    pub status: String,
}

impl AgentServiceRow {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing name"))?
            .to_string();

        let pid = row.get("pid").and_then(|v| v.as_i64()).map(|i| i as i32);

        let port = row
            .get("port")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow!("Missing port"))? as i32;

        let status = row
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing status"))?
            .to_string();

        Ok(Self {
            name,
            pid,
            port,
            status,
        })
    }
}

#[derive(Debug)]
pub struct AgentServerIdRow {
    pub name: String,
}

impl AgentServerIdRow {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing name"))?
            .to_string();

        Ok(Self { name })
    }
}

#[derive(Debug)]
pub struct AgentServerIdPidRow {
    pub name: String,
    pub pid: i32,
}

impl AgentServerIdPidRow {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing name"))?
            .to_string();

        let pid = row
            .get("pid")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow!("Missing pid"))? as i32;

        Ok(Self { name, pid })
    }
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
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn register_agent(
        &self,
        name: &str,
        pid: u32,
        port: u16,
        _auth: &str,
    ) -> Result<String, RepositoryError> {
        // Clean up any existing service entries for this agent first
        self.remove_agent_service(name).await?;

        let pid_i32 = pid as i32;
        let port_i32 = port as i32;

        let query = DatabaseQueryEnum::RegisterAgent.get(self.db_pool.as_ref());
        self.db_pool
            .as_ref()
            .execute(&query, &[&name, &pid_i32, &port_i32])
            .await
            .context("Failed to register agent")?;

        Ok(name.to_string())
    }

    pub async fn get_agent_status(
        &self,
        agent_name: &str,
    ) -> Result<Option<AgentServiceRow>, RepositoryError> {
        let query = DatabaseQueryEnum::GetAgentStatus.get(self.db_pool.as_ref());
        let row = self
            .db_pool
            .as_ref()
            .fetch_optional(&query, &[&agent_name])
            .await
            .context("Failed to get agent status")?;

        match row {
            Some(r) => Ok(Some(AgentServiceRow::from_json_row(&r)?)),
            None => Ok(None),
        }
    }

    pub async fn mark_crashed(&self, agent_name: &str) -> Result<(), RepositoryError> {
        let query = DatabaseQueryEnum::MarkAgentCrashed.get(self.db_pool.as_ref());
        self.db_pool
            .as_ref()
            .execute(&query, &[&agent_name])
            .await
            .context("Failed to mark agent as crashed")?;

        Ok(())
    }

    pub async fn mark_stopped(&self, agent_name: &str) -> Result<(), RepositoryError> {
        let query = DatabaseQueryEnum::MarkAgentStopped.get(self.db_pool.as_ref());
        self.db_pool
            .as_ref()
            .execute(&query, &[&agent_name])
            .await
            .context("Failed to mark agent as stopped")?;

        Ok(())
    }

    pub async fn mark_error(
        &self,
        agent_name: &str,
        _error_message: &str,
    ) -> Result<(), RepositoryError> {
        let query = DatabaseQueryEnum::MarkAgentError.get(self.db_pool.as_ref());
        self.db_pool
            .as_ref()
            .execute(&query, &[&agent_name])
            .await
            .context("Failed to mark agent with error")?;

        Ok(())
    }

    pub async fn list_running_agents(&self) -> Result<Vec<AgentServerIdRow>, RepositoryError> {
        let query = DatabaseQueryEnum::ListRunningAgents.get(self.db_pool.as_ref());
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[])
            .await
            .context("Failed to list running agents")?;

        rows.iter()
            .map(|r| AgentServerIdRow::from_json_row(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| RepositoryError::GenericError(e))
    }

    pub async fn list_running_agent_pids(
        &self,
    ) -> Result<Vec<AgentServerIdPidRow>, RepositoryError> {
        let query = DatabaseQueryEnum::ListRunningAgents.get(self.db_pool.as_ref());
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[])
            .await
            .context("Failed to list running agent PIDs")?;

        rows.iter()
            .map(|r| AgentServerIdPidRow::from_json_row(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| RepositoryError::GenericError(e))
    }

    pub async fn remove_agent_service(&self, agent_name: &str) -> Result<(), RepositoryError> {
        let query = DatabaseQueryEnum::RemoveAgentService.get(self.db_pool.as_ref());
        self.db_pool
            .as_ref()
            .execute(&query, &[&agent_name])
            .await
            .context("Failed to remove agent service")?;

        Ok(())
    }

    pub async fn update_health_status(
        &self,
        agent_name: &str,
        health_status: &str,
    ) -> Result<(), RepositoryError> {
        let query = DatabaseQueryEnum::UpdateAgentHealth.get(self.db_pool.as_ref());
        self.db_pool
            .as_ref()
            .execute(&query, &[&health_status, &agent_name])
            .await
            .context("Failed to update agent health status")?;

        Ok(())
    }
}
