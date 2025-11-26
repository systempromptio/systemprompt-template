use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt_core_database::{DatabaseProvider, DatabaseQuery, DbPool, JsonRow};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

const GET_SERVICE_BY_NAME: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/get_service_by_name.sql"
));

const GET_ALL_MCP_SERVERS: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/get_all_mcp_servers.sql"
));

const GET_MCP_SERVICES: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/get_mcp_services.sql"
));

const CREATE_SERVICE: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/create_service.sql"
));

const UPDATE_SERVICE_STATUS: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/update_service_status.sql"
));

const DELETE_SERVICE: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/delete_service.sql"
));

const UPDATE_SERVICE_RUNNING: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/update_service_running.sql"
));

const UPDATE_SERVICE_STOPPED: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/update_service_stopped.sql"
));

const UPDATE_SERVICE_PID: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/update_service_pid.sql"
));

const CLEAR_SERVICE_PID: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/clear_service_pid.sql"
));

const UPDATE_SERVICE_ERROR: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/update_service_error.sql"
));

const GET_SERVICE_BY_PID: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/get_service_by_pid.sql"
));

const GET_SERVICES_BY_PORT: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/get_services_by_port.sql"
));

const GET_ALL_RUNNING_SERVICES: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/get_all_running_services.sql"
));

const COUNT_RUNNING_SERVICES: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/count_running_services.sql"
));

const UPDATE_SERVICE_PORT: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/update_service_port.sql"
));

const GET_RUNNING_SERVICES_WITH_PID: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/get_running_services_with_pid.sql"
));

const MARK_SERVICE_CRASHED: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/mark_service_crashed.sql"
));

const GET_ALL_AGENT_SERVICE_NAMES: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/services/postgres/get_all_agent_service_names.sql"
));

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub module_name: String,
    pub status: String,
    pub pid: Option<i32>,
    pub port: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl ServiceConfig {
    fn extract_timestamp(value: &serde_json::Value) -> Option<String> {
        // String format (most common for timestamps)
        if let Some(s) = value.as_str() {
            return Some(s.to_string());
        }

        // Numeric formats
        if let Some(n) = value.as_i64() {
            return Some(n.to_string());
        }
        if let Some(n) = value.as_u64() {
            return Some(n.to_string());
        }
        if let Some(n) = value.as_f64() {
            return Some(n.to_string());
        }

        // Object format (datetime objects)
        if let Some(obj) = value.as_object() {
            if let Some(v) = obj
                .get("secs_since_epoch")
                .or_else(|| obj.get("timestamp"))
                .and_then(serde_json::Value::as_i64)
            {
                return Some(v.to_string());
            }
        }

        // If it's null, return None
        None
    }

    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing name"))?
            .to_string();

        let module_name = row
            .get("module_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing module_name"))?
            .to_string();

        let status = row
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing status"))?
            .to_string();

        let pid = row
            .get("pid")
            .and_then(serde_json::Value::as_i64)
            .and_then(|i| i32::try_from(i).ok());

        let port = row
            .get("port")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing port"))
            .and_then(|i| i32::try_from(i).map_err(|_| anyhow!("Port out of range")))?;

        let created_at = row
            .get("created_at")
            .and_then(Self::extract_timestamp)
            .unwrap_or_else(|| "unknown".to_string());

        let updated_at = row
            .get("updated_at")
            .and_then(Self::extract_timestamp)
            .unwrap_or_else(|| "unknown".to_string());

        Ok(Self {
            name,
            module_name,
            status,
            pid,
            port,
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub config: String,
}

impl McpServer {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing name"))?
            .to_string();

        let config = row
            .get("config")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing config"))?
            .to_string();

        Ok(Self { id, name, config })
    }
}

#[derive(Debug, Clone)]
pub struct ServiceRepository {
    db_pool: DbPool,
}

impl RepositoryTrait for ServiceRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl ServiceRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_service_by_name(&self, name: &str) -> Result<Option<ServiceConfig>> {
        let row = self
            .db_pool
            .as_ref()
            .fetch_optional(&GET_SERVICE_BY_NAME, &[&name])
            .await?;
        row.map(|r| ServiceConfig::from_json_row(&r)).transpose()
    }

    pub async fn get_all_agent_service_names(&self) -> Result<Vec<String>> {
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&GET_ALL_AGENT_SERVICE_NAMES, &[])
            .await?;

        Ok(rows
            .iter()
            .filter_map(|row| row.get("name").and_then(|v| v.as_str()).map(String::from))
            .collect())
    }

    pub async fn get_all_mcp_servers(&self) -> Result<Vec<McpServer>> {
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&GET_ALL_MCP_SERVERS, &[])
            .await?;

        rows.iter()
            .map(McpServer::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_mcp_services(&self) -> Result<Vec<ServiceConfig>> {
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&GET_MCP_SERVICES, &[])
            .await?;

        rows.iter()
            .map(ServiceConfig::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn create_service_for_mcp_server(
        &self,
        mcp_server: &McpServer,
        port: u16,
    ) -> Result<()> {
        let name = &mcp_server.name;
        let module_name = "mcp";
        let port_i32 = i32::from(port);
        let status = "running";

        self.db_pool
            .as_ref()
            .execute(&CREATE_SERVICE, &[&name, &module_name, &status, &port_i32])
            .await?;
        Ok(())
    }

    pub async fn create_service(
        &self,
        name: &str,
        module_name: &str,
        status: &str,
        port: u16,
    ) -> Result<()> {
        let port_i32 = i32::from(port);
        self.db_pool
            .as_ref()
            .execute(&CREATE_SERVICE, &[&name, &module_name, &status, &port_i32])
            .await?;
        Ok(())
    }

    pub async fn update_service_status(&self, service_name: &str, status: &str) -> Result<()> {
        self.db_pool
            .as_ref()
            .execute(&UPDATE_SERVICE_STATUS, &[&status, &service_name])
            .await?;
        Ok(())
    }

    pub async fn delete_service(&self, service_name: &str) -> Result<()> {
        self.db_pool
            .as_ref()
            .execute(&DELETE_SERVICE, &[&service_name])
            .await?;
        Ok(())
    }

    pub async fn update_service_running(&self, service_name: &str, pid: i32) -> Result<()> {
        self.db_pool
            .as_ref()
            .execute(&UPDATE_SERVICE_RUNNING, &[&pid, &service_name])
            .await?;
        Ok(())
    }

    pub async fn update_service_stopped(&self, service_name: &str) -> Result<()> {
        self.db_pool
            .as_ref()
            .execute(&UPDATE_SERVICE_STOPPED, &[&service_name])
            .await?;
        Ok(())
    }

    pub async fn update_service_pid(&self, service_name: &str, pid: i32) -> Result<()> {
        self.db_pool
            .as_ref()
            .execute(&UPDATE_SERVICE_PID, &[&pid, &service_name])
            .await?;
        Ok(())
    }

    pub async fn clear_service_pid(&self, service_name: &str) -> Result<()> {
        self.db_pool
            .as_ref()
            .execute(&CLEAR_SERVICE_PID, &[&service_name])
            .await?;
        Ok(())
    }

    pub async fn update_service_error(&self, service_name: &str) -> Result<()> {
        self.db_pool
            .as_ref()
            .execute(&UPDATE_SERVICE_ERROR, &[&service_name])
            .await?;
        Ok(())
    }

    pub async fn get_service_by_pid(&self, pid: i32) -> Result<Option<ServiceConfig>> {
        let row = self
            .db_pool
            .as_ref()
            .fetch_optional(&GET_SERVICE_BY_PID, &[&pid])
            .await?;
        row.map(|r| ServiceConfig::from_json_row(&r)).transpose()
    }

    pub async fn get_services_by_port(&self, port: u16) -> Result<Vec<ServiceConfig>> {
        let port_i32 = i32::from(port);
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&GET_SERVICES_BY_PORT, &[&port_i32])
            .await?;

        rows.iter()
            .map(ServiceConfig::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_all_running_services(&self) -> Result<Vec<ServiceConfig>> {
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&GET_ALL_RUNNING_SERVICES, &[])
            .await?;

        rows.iter()
            .map(ServiceConfig::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn count_running_services(&self, module_name: &str) -> Result<usize> {
        use systemprompt_core_database::DbValue;

        let value = self
            .db_pool
            .as_ref()
            .fetch_scalar_value(&COUNT_RUNNING_SERVICES, &[&module_name])
            .await?;

        match value {
            DbValue::Int(n) => Ok(usize::try_from(n).unwrap_or(0)),
            _ => Ok(0),
        }
    }

    pub async fn update_service_port(&self, service_name: &str, port: u16) -> Result<()> {
        let port_i32 = i32::from(port);
        self.db_pool
            .as_ref()
            .execute(&UPDATE_SERVICE_PORT, &[&port_i32, &service_name])
            .await?;
        Ok(())
    }

    pub async fn get_running_services_with_pid(
        &self,
    ) -> Result<Vec<crate::repository::ServiceRecord>> {
        use crate::repository::ServiceRecord;

        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&GET_RUNNING_SERVICES_WITH_PID, &[])
            .await?;

        rows.iter()
            .map(ServiceRecord::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn mark_service_crashed(&self, service_name: &str) -> Result<()> {
        self.db_pool
            .as_ref()
            .execute(&MARK_SERVICE_CRASHED, &[&service_name])
            .await?;
        Ok(())
    }
}
