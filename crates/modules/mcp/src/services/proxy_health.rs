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
        let Some(service) = self.service_repo.get_service_by_name(service_name).await? else {
            return Ok(false);
        };

        if service.status != "running" {
            return Ok(false);
        }

        if !Self::is_port_responsive(port).await {
            self.service_repo
                .update_service_status(service_name, "stopped")
                .await?;
            return Ok(false);
        }

        if !Self::can_connect_mcp(port).await {
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
        )
        .is_ok()
    }

    async fn can_connect_mcp(port: u16) -> bool {
        use crate::services::client::validate_connection;

        match tokio::time::timeout(
            Duration::from_millis(500),
            validate_connection("proxy_check", "127.0.0.1", port),
        )
        .await
        {
            Ok(Ok(result)) => result.success || result.validation_type == "auth_required",
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
