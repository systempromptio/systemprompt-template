use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use systemprompt_models::Config;

use crate::services::deployment::DeploymentService;
use crate::ServerManifest;

fn resolve_mcp_server_path(
    deployment: &systemprompt_models::mcp::deployment::Deployment,
    services_path: &str,
    server_name: &str,
) -> PathBuf {
    match &deployment.path {
        Some(explicit_path) => PathBuf::from(explicit_path),
        None => Path::new(services_path).join("mcp").join(server_name),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegistryService;

impl RegistryService {
    fn registry_dir() -> PathBuf {
        let config = Config::global();
        PathBuf::from(&config.system_path).join("metadata/mcp")
    }

    pub async fn load_manifest(name: &str) -> Result<ServerManifest> {
        let registry_dir = Self::registry_dir();
        let path = registry_dir.join(format!("{name}.json"));

        let content = fs::read_to_string(&path)
            .map_err(|e| anyhow!("Failed to read manifest file {}: {}", path.display(), e))?;

        let manifest: ServerManifest = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse manifest JSON {}: {}", path.display(), e))?;

        Ok(manifest)
    }

    pub async fn list_servers() -> Result<Vec<String>> {
        let registry_dir = Self::registry_dir();

        // Manifests are optional - if directory doesn't exist, return empty list
        let Ok(entries) = fs::read_dir(&registry_dir) else {
            return Ok(Vec::new());
        };

        let servers: Vec<String> = entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.extension()?.to_str()? == "json" {
                        path.file_stem()?.to_str().map(ToString::to_string)
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(servers)
    }

    pub async fn get_enabled_servers() -> Result<Vec<(ServerManifest, u16)>> {
        let servers = Self::list_servers().await?;
        let mut enabled = Vec::new();

        for server_name in servers {
            if DeploymentService::is_server_enabled(&server_name).await? {
                let manifest = Self::load_manifest(&server_name).await?;
                let port = DeploymentService::get_server_port(&server_name).await?;
                enabled.push((manifest, port));
            }
        }

        Ok(enabled)
    }

    pub async fn get_enabled_servers_as_config() -> Result<Vec<crate::McpServerConfig>> {
        use systemprompt_core_config::services::ConfigLoader;

        let services_config = ConfigLoader::load().await?;
        let mut enabled = Vec::new();

        let mut server_names: Vec<_> = services_config.mcp_servers.keys().collect();
        server_names.sort();

        for server_name in server_names {
            let deployment = &services_config.mcp_servers[server_name];
            if !deployment.enabled {
                continue;
            }

            let path = deployment
                .path
                .as_ref()
                .ok_or_else(|| anyhow!("Missing path for MCP server: {server_name}"))?;
            let package = deployment
                .package
                .as_ref()
                .ok_or_else(|| anyhow!("Missing package for MCP server: {server_name}"))?;

            let crate_path = PathBuf::from(path);

            let config = crate::McpServerConfig {
                name: server_name.clone(),
                enabled: deployment.enabled,
                display_in_web: deployment.display_in_web,
                port: deployment.port,
                crate_path,
                display_name: package.clone(),
                description: format!("{package} MCP Server"),
                capabilities: vec![],
                schemas: deployment.schemas.clone(),
                oauth: deployment.oauth.clone(),
                tools: deployment.tools.clone(),
                model_config: deployment.model_config.clone(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                host: "0.0.0.0".to_string(),
                module_name: "mcp".to_string(),
                protocol: "mcp".to_string(),
            };
            enabled.push(config);
        }

        Ok(enabled)
    }

    pub async fn get_server_with_deployment(name: &str) -> Result<(ServerManifest, u16)> {
        let manifest = Self::load_manifest(name).await?;
        let port = DeploymentService::get_server_port(name).await?;
        Ok((manifest, port))
    }

    pub async fn validate_registry() -> Result<()> {
        // MCP servers are now configured in services.yaml
        // Just validate that the config is loadable
        DeploymentService::validate_config().await?;
        Ok(())
    }
}

// Legacy compatibility functions
pub async fn get_enabled_servers(
    _config: &systemprompt_models::ServicesConfig,
) -> Result<Vec<crate::McpServerConfig>> {
    RegistryService::get_enabled_servers_as_config().await
}

pub async fn get_all_servers(
    config: &systemprompt_models::ServicesConfig,
) -> Result<Vec<crate::McpServerConfig>> {
    let services_path = config
        .settings
        .services_path
        .as_deref()
        .unwrap_or("crates/services");
    let servers = RegistryService::list_servers().await?;
    let mut configs = Vec::new();

    for server_name in servers {
        let manifest = RegistryService::load_manifest(&server_name).await?;
        let deployment = config
            .mcp_servers
            .get(&server_name)
            .ok_or_else(|| anyhow!("No deployment config for {server_name}"))?;

        let crate_path = resolve_mcp_server_path(deployment, services_path, &server_name);

        let server_config = crate::McpServerConfig::from_manifest_and_deployment(
            server_name,
            &manifest,
            deployment,
            crate_path,
        );
        configs.push(server_config);
    }

    Ok(configs)
}

pub async fn get_server_by_name(
    config: &systemprompt_models::ServicesConfig,
    name: &str,
) -> Result<Option<crate::McpServerConfig>> {
    let services_path = config
        .settings
        .services_path
        .as_deref()
        .unwrap_or("crates/services");

    if let Some(deployment) = config.mcp_servers.get(name) {
        let manifest = RegistryService::load_manifest(name).await?;

        let crate_path = resolve_mcp_server_path(deployment, services_path, name);

        let server_config = crate::McpServerConfig::from_manifest_and_deployment(
            name.to_string(),
            &manifest,
            deployment,
            crate_path,
        );
        Ok(Some(server_config))
    } else {
        Ok(None)
    }
}

pub async fn count_enabled_servers(config: &systemprompt_models::ServicesConfig) -> usize {
    config.mcp_servers.values().filter(|d| d.enabled).count()
}

pub async fn get_servers_by_oauth_requirement(
    config: &systemprompt_models::ServicesConfig,
    oauth_required: bool,
) -> Result<Vec<crate::McpServerConfig>> {
    let services_path = config
        .settings
        .services_path
        .as_deref()
        .unwrap_or("crates/services");
    let mut configs = Vec::new();

    for (server_name, deployment) in &config.mcp_servers {
        if deployment.enabled && deployment.oauth.required == oauth_required {
            let manifest = RegistryService::load_manifest(server_name).await?;

            let crate_path = resolve_mcp_server_path(deployment, services_path, server_name);

            let server_config = crate::McpServerConfig::from_manifest_and_deployment(
                server_name.clone(),
                &manifest,
                deployment,
                crate_path,
            );
            configs.push(server_config);
        }
    }

    Ok(configs)
}
