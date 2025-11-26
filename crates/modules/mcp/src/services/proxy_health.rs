use anyhow::Result;
use std::net::TcpStream;
use std::time::Duration;
use systemprompt_models::repository::ServiceRepository;

#[derive(Debug)]
pub struct ProxyHealthCheck {
    service_repo: ServiceRepository,
}

impl ProxyHealthCheck {
    pub const fn new(db_pool: systemprompt_core_database::DbPool) -> Self {
        Self {
            service_repo: ServiceRepository::new(db_pool),
        }
    }

    pub async fn can_route_traffic(&self, service_name: &str, port: u16) -> Result<bool> {
        // Step 1: Check if service exists and get PID
        let Some(service) = self.service_repo.get_service_by_name(service_name).await? else {
            return Ok(false); // Service not registered
        };

        // Step 2: Check if marked as running (this also verifies PID)
        if service.status != "running" {
            return Ok(false);
        }

        // Step 3: Check if port is actually open and responsive
        if !Self::is_port_responsive(port).await {
            // Port not responsive, update database
            self.service_repo
                .update_service_status(service_name, "stopped")
                .await?;
            return Ok(false);
        }

        // Step 4: For MCP services, verify protocol handshake (quick check)
        if !Self::can_connect_mcp(port).await {
            // Can't establish MCP connection
            self.service_repo
                .update_service_status(service_name, "error")
                .await?;
            return Ok(false);
        }

        Ok(true)
    }

    async fn is_port_responsive(port: u16) -> bool {
        TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            Duration::from_millis(100),
        ).is_ok()
    }

    async fn can_connect_mcp(port: u16) -> bool {
        use crate::services::client::McpClient;

        // Quick MCP protocol check without full validation
        match tokio::time::timeout(
            Duration::from_millis(500),
            McpClient::validate_connection("proxy_check", "127.0.0.1", port),
        )
        .await
        {
            Ok(Ok(result)) => {
                // Service responded to MCP protocol
                result.success || result.validation_type == "auth_required"
            },
            _ => false,
        }
    }

    pub async fn get_routable_services(&self) -> Result<Vec<RoutableService>> {
        let running_services = self.service_repo.get_all_running_services().await?;

        let mut routable = Vec::new();

        for service in running_services {
            if let Some(port) = Self::parse_port_from_service(&service) {
                if Self::is_port_responsive(port).await {
                    routable.push(RoutableService {
                        name: service.name.clone(),
                        port,
                        pid: service.pid,
                        health: "healthy".to_string(),
                    });
                } else {
                    // Update database to reflect reality
                    self.service_repo
                        .update_service_status(&service.name, "stopped")
                        .await?;
                }
            }
        }

        Ok(routable)
    }

    const fn parse_port_from_service(
        service: &systemprompt_models::repository::ServiceConfig,
    ) -> Option<u16> {
        Some(service.port as u16)
    }
}

#[derive(Debug, Clone)]
pub struct RoutableService {
    pub name: String,
    pub port: u16,
    pub pid: Option<i32>,
    pub health: String,
}
