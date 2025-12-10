use crate::{McpServerConfig, ERROR, RUNNING};
use anyhow::Result;
use systemprompt_core_logging::CliService;
use systemprompt_core_system::Config;
use systemprompt_models::repository::{ServiceConfig, ServiceRepository};

#[derive(Debug)]
pub struct McpServiceDisplay {
    service_repo: ServiceRepository,
}

#[allow(dead_code)]
impl McpServiceDisplay {
    pub const fn new(service_repo: ServiceRepository) -> Self {
        Self { service_repo }
    }

    pub async fn show_status(&self, registry_servers: &[McpServerConfig]) -> Result<()> {
        let db_services = match self.service_repo.get_mcp_services().await {
            Ok(services) => services,
            Err(e) => {
                CliService::warning(&format!("⚠️ Failed to query database services: {e}"));
                CliService::info("   Showing registry configuration only");
                Vec::new()
            },
        };

        if registry_servers.is_empty() {
            CliService::info("No MCP services configured in registry");
            return Ok(());
        }

        CliService::section("MCP Services Status");

        let headers = &[
            "Service Name",
            "Description",
            "Port",
            "OAuth",
            "Status",
            "Connection URL",
        ];

        let (rows, running_count, error_count) = registry_servers.iter().fold(
            (Vec::new(), 0, 0),
            |(mut rows, mut running, mut errors), config| {
                let db_service = db_services.iter().find(|s| s.name == config.name);
                let status = match db_service {
                    Some(s) => s.status.as_str(),
                    None => "not_running",
                };

                match status {
                    RUNNING => running += 1,
                    ERROR => errors += 1,
                    _ => {},
                }
                rows.push(Self::format_row_from_config(config, db_service, status));
                (rows, running, errors)
            },
        );

        CliService::table(headers, &rows);

        if running_count > 0 {
            CliService::success(&format!(
                "✅ {running_count} MCP servers running successfully"
            ));
        }
        if error_count > 0 {
            CliService::error(&format!("❌ {error_count} MCP servers failed to start"));
        }

        Ok(())
    }

    fn format_row(service: &ServiceConfig, registry_servers: &[McpServerConfig]) -> Vec<String> {
        let description = registry_servers
            .iter()
            .find(|s| s.name == service.name)
            .map_or_else(|| "[no description]".to_string(), |s| s.description.clone());
        vec![
            service.name.clone(),
            description,
            service.port.to_string(),
            Self::format_auth_status(service, registry_servers),
            Self::format_status(&service.status),
            Self::format_url(service),
        ]
    }

    fn format_row_from_config(
        config: &McpServerConfig,
        db_service: Option<&ServiceConfig>,
        status: &str,
    ) -> Vec<String> {
        let port = match db_service {
            Some(s) => s.port.to_string(),
            None => config.port.to_string(),
        };

        vec![
            config.name.clone(),
            config.description.clone(),
            port,
            Self::format_oauth_from_config(config),
            Self::format_status(status),
            Self::format_url_from_config(&config.name, status),
        ]
    }

    fn format_auth_status(service: &ServiceConfig, registry_servers: &[McpServerConfig]) -> String {
        match registry_servers.iter().find(|s| s.name == service.name) {
            Some(server_config) => {
                if server_config.oauth.required {
                    let client = server_config
                        .oauth
                        .client_id
                        .as_deref()
                        .filter(|c| !c.is_empty())
                        .unwrap_or("[no client ID]");
                    format!("🔒 Required ({client})")
                } else {
                    "🌐 Anonymous".to_string()
                }
            },
            None => "❓ Unknown".to_string(),
        }
    }

    fn format_oauth_from_config(config: &McpServerConfig) -> String {
        if config.oauth.required {
            let client = config
                .oauth
                .client_id
                .as_deref()
                .filter(|c| !c.is_empty())
                .unwrap_or("[no client ID]");
            format!("🔒 Required ({client})")
        } else {
            "🌐 Anonymous".to_string()
        }
    }

    fn format_status(status: &str) -> String {
        match status {
            RUNNING => format!("✅ {status}"),
            ERROR => format!("❌ {status}"),
            "not_running" => "⏸️ stopped".to_string(),
            _ => status.to_string(),
        }
    }

    fn format_url(service: &ServiceConfig) -> String {
        if service.status == RUNNING {
            let config = Config::global();
            format!(
                "{}/api/v1/mcp/{}/mcp",
                config.api_external_url, service.name
            )
        } else {
            "N/A".to_string()
        }
    }

    fn format_url_from_config(name: &str, status: &str) -> String {
        if status == RUNNING {
            let config = Config::global();
            format!("{}/api/v1/mcp/{}/mcp", config.api_external_url, name)
        } else {
            "N/A".to_string()
        }
    }
}
