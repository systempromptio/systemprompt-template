use anyhow::Result;
use std::collections::HashSet;
use systemprompt_core_logging::CliService;
use systemprompt_models::mcp::RegistryConfig;

pub async fn validate_registry(config: &RegistryConfig) -> Result<()> {
    CliService::info("Validating registry configuration...");

    validate_port_conflicts(config).await?;
    validate_server_configs(config).await?;
    validate_oauth_configs(config).await?;

    CliService::success("Registry validation passed");
    Ok(())
}

async fn validate_port_conflicts(config: &RegistryConfig) -> Result<()> {
    let mut ports = HashSet::new();
    let mut conflicts = Vec::new();

    for server_config in &config.servers {
        if server_config.enabled {
            if ports.contains(&server_config.port) {
                conflicts.push((server_config.name.clone(), server_config.port));
            } else {
                ports.insert(server_config.port);
            }
        }
    }

    if !conflicts.is_empty() {
        let conflict_str = conflicts
            .iter()
            .map(|(name, port)| format!("{name}:{port}"))
            .collect::<Vec<_>>()
            .join(", ");

        return Err(anyhow::anyhow!("Port conflicts detected: {conflict_str}"));
    }

    CliService::success(&format!(
        "No port conflicts found ({} enabled servers)",
        ports.len()
    ));
    Ok(())
}

async fn validate_server_configs(config: &RegistryConfig) -> Result<()> {
    let mut invalid_servers = Vec::new();

    for server_config in &config.servers {
        if server_config.enabled {
            if server_config.port < 1024 {
                invalid_servers.push(format!(
                    "{}: invalid port {}",
                    server_config.name, server_config.port
                ));
                continue;
            }

            if !std::path::Path::new(&server_config.crate_path).exists() {
                invalid_servers.push(format!(
                    "{}: crate path does not exist: {}",
                    server_config.name,
                    server_config.crate_path.display()
                ));
                continue;
            }

            if server_config.display_name.is_empty() {
                invalid_servers.push(format!("{}: missing display_name", server_config.name));
            }

            if server_config.description.is_empty() {
                invalid_servers.push(format!("{}: missing description", server_config.name));
            }
        }
    }

    if !invalid_servers.is_empty() {
        return Err(anyhow::anyhow!(
            "Invalid server configurations:\n{}",
            invalid_servers.join("\n")
        ));
    }

    CliService::success("All server configurations valid");
    Ok(())
}

async fn validate_oauth_configs(config: &RegistryConfig) -> Result<()> {
    let mut oauth_issues = Vec::new();

    for server_config in &config.servers {
        if server_config.enabled
            && server_config.oauth.required
            && server_config.oauth.scopes.is_empty()
        {
            oauth_issues.push(format!(
                "{}: OAuth enabled but no scopes defined",
                server_config.name
            ));
        }
    }

    if !oauth_issues.is_empty() {
        return Err(anyhow::anyhow!(
            "OAuth configuration issues:\n{}",
            oauth_issues.join("\n")
        ));
    }

    CliService::success("OAuth configurations valid");
    Ok(())
}
