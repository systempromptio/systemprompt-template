use anyhow::Result;
use systemprompt_core_logging::CliService;

use crate::services::client::validate_connection_with_auth;
use crate::services::database::DatabaseManager;
use crate::services::registry::RegistryManager;

pub async fn validate_service(
    service_name: &str,
    registry: &RegistryManager,
    database: &DatabaseManager,
) -> Result<()> {
    let servers = registry.get_enabled_servers().await?;
    let server = servers
        .iter()
        .find(|s| s.name == service_name)
        .ok_or_else(|| anyhow::anyhow!("Service '{service_name}' not found in registry"))?;

    CliService::section(&format!("Validating MCP Service: {service_name}"));

    CliService::info(&format!("Service '{service_name}' found in registry"));
    CliService::info(&format!("   Port: {}", server.port));
    CliService::info(&format!("   Enabled: {}", server.enabled));
    CliService::info(&format!("   OAuth required: {}", server.oauth.required));

    let service_info = database.get_service_by_name(service_name).await?;

    let is_running = service_info
        .as_ref()
        .is_some_and(|info| info.status == "running");

    if !is_running {
        CliService::warning(&format!(
            "Service '{service_name}' is not currently running"
        ));
        CliService::info("   Start the service first with: just mcp start systemprompt-admin");
        return Ok(());
    }

    CliService::info("Connecting to MCP service...");

    let validation_result = validate_connection_with_auth(
        &server.name,
        "127.0.0.1",
        server.port,
        server.oauth.required,
    )
    .await?;

    if validation_result.success {
        CliService::success("Successfully connected to MCP service");

        if let Some(server_info) = &validation_result.server_info {
            CliService::info(&format!("   Server: {}", server_info.server_name));
            CliService::info(&format!("   Version: {}", server_info.version));
            CliService::info(&format!("   Protocol: {}", server_info.protocol_version));
        }

        if server.oauth.required && validation_result.validation_type == "auth_required" {
            CliService::info("Service requires OAuth authentication");
            CliService::info("Full validation skipped (requires user JWT token)");
            CliService::info("   Port is responding - service appears healthy");
        } else if validation_result.validation_type == "mcp_validated" {
            CliService::success("MCP Protocol validated");
            CliService::info(&format!(
                "   Tools available: {}",
                validation_result.tools_count
            ));
            CliService::info("Tool listing requires user authentication");
        }

        CliService::info(&format!(
            "Connection time: {}ms",
            validation_result.connection_time_ms
        ));
    } else {
        let error = validation_result
            .error_message
            .as_deref()
            .filter(|e| !e.is_empty())
            .unwrap_or("[no error message]");
        CliService::error(&format!("Failed to connect: {error}"));
    }

    Ok(())
}
