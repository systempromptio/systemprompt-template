use anyhow::Result;
use systemprompt_core_logging::CliService;
use systemprompt_models::mcp::RegistryConfig;

pub async fn discover_server_capabilities(config: &RegistryConfig) -> Result<()> {
    CliService::info("🔍 Discovering server capabilities...");

    for server_config in &config.servers {
        if server_config.enabled {
            CliService::info(&format!(
                "   📋 {}: {}",
                server_config.name, server_config.display_name
            ));
            CliService::info(&format!("      🌐 Port: {}", server_config.port));
            CliService::info(&format!(
                "      📁 Path: {}",
                server_config.crate_path.display()
            ));

            if server_config.oauth.required {
                CliService::info(&format!(
                    "      🔒 OAuth: Required ({})",
                    server_config
                        .oauth
                        .scopes
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            } else {
                CliService::info("      🌐 OAuth: Anonymous access");
            }

            if !server_config.capabilities.is_empty() {
                CliService::info(&format!(
                    "      🔧 Capabilities: {}",
                    server_config.capabilities.join(", ")
                ));
            }
        }
    }

    CliService::success("✅ Server capability discovery completed");
    Ok(())
}

pub async fn validate_server_availability(config: &RegistryConfig) -> Result<Vec<String>> {
    use systemprompt_core_system::Config;
    let cargo_target_dir = &Config::global().cargo_target_dir;
    let mut unavailable = Vec::new();

    for server_config in &config.servers {
        if server_config.enabled {
            let binary_path = format!(
                "{}/{}/release/{}",
                server_config.crate_path.display(),
                cargo_target_dir,
                server_config.name
            );

            if !std::path::Path::new(&binary_path).exists() {
                unavailable.push(format!(
                    "{}: binary not found at {}",
                    server_config.name, binary_path
                ));
            }
        }
    }

    if unavailable.is_empty() {
        CliService::success("✅ All enabled servers have available binaries");
    } else {
        CliService::warning(&format!(
            "⚠️ {} servers missing binaries",
            unavailable.len()
        ));
        for issue in &unavailable {
            CliService::warning(&format!("   {issue}"));
        }
    }

    Ok(unavailable)
}
