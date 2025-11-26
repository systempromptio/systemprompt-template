use crate::services::monitoring::health::{perform_health_check, HealthStatus};
use crate::{McpServerConfig, ERROR, RUNNING, STOPPED};
use anyhow::Result;
use std::collections::HashMap;
use std::hash::BuildHasher;
use systemprompt_core_logging::CliService;

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub state: String,
    pub pid: Option<u32>,
    pub health: String,
    pub uptime_seconds: Option<i64>,
    pub tools_count: usize,
    pub latency_ms: Option<u32>,
    pub auth_required: bool,
}

pub async fn get_all_service_status(
    servers: &[McpServerConfig],
) -> Result<HashMap<String, ServiceStatus>> {
    let mut status_map = HashMap::new();

    for server in servers {
        let status = get_service_status(server).await?;
        status_map.insert(server.name.clone(), status);
    }

    Ok(status_map)
}

async fn get_service_status(config: &McpServerConfig) -> Result<ServiceStatus> {
    match perform_health_check(config).await {
        Ok(health_result) => {
            let state = match health_result.status {
                HealthStatus::Healthy | HealthStatus::Degraded => RUNNING.to_string(),
                HealthStatus::Unhealthy => STOPPED.to_string(),
                HealthStatus::Unknown => ERROR.to_string(),
            };

            Ok(ServiceStatus {
                state,
                pid: None,
                health: health_result.status.as_str().to_string(),
                uptime_seconds: None,
                tools_count: health_result.details.tools_available,
                latency_ms: Some(health_result.latency_ms),
                auth_required: config.oauth.required,
            })
        },
        Err(_) => Ok(ServiceStatus {
            state: STOPPED.to_string(),
            pid: None,
            health: "unreachable".to_string(),
            uptime_seconds: None,
            tools_count: 0,
            latency_ms: None,
            auth_required: config.oauth.required,
        }),
    }
}

pub async fn display_service_status<S: BuildHasher>(
    servers: &[McpServerConfig],
    status_data: &HashMap<String, ServiceStatus, S>,
) -> Result<()> {
    if servers.is_empty() {
        CliService::info("No MCP services configured");
        return Ok(());
    }

    CliService::section("MCP Services Status");

    let headers = &[
        "Service", "Port", "Health", "Tools", "Latency", "Auth", "URL",
    ];
    let mut rows = Vec::new();
    let mut running_count = 0;
    let mut error_count = 0;
    let mut auth_required_count = 0;

    for server in servers {
        let status = status_data.get(&server.name);

        if let Some(s) = status {
            match s.state.as_str() {
                RUNNING => {
                    running_count += 1;
                    if s.auth_required {
                        auth_required_count += 1;
                    }
                },
                ERROR => error_count += 1,
                _ => {},
            }
        }

        rows.push(format_status_row(server, status));
    }

    CliService::table(headers, &rows);

    let mut summary_lines = Vec::new();
    if running_count > 0 {
        summary_lines.push(format!("✅ {running_count} running"));
    }
    if auth_required_count > 0 {
        summary_lines.push(format!("🔐 {auth_required_count} require auth"));
    }
    if error_count > 0 {
        summary_lines.push(format!("❌ {error_count} failed"));
    }

    if !summary_lines.is_empty() {
        CliService::info(&format!("Summary: {}", summary_lines.join(" | ")));
    }

    Ok(())
}

fn format_status_row(config: &McpServerConfig, status: Option<&ServiceStatus>) -> Vec<String> {
    let Some(status) = status else {
        return vec![
            config.name.clone(),
            config.port.to_string(),
            "❓ unknown".to_string(),
            "-".to_string(),
            "-".to_string(),
            if config.oauth.required {
                "🔒 Yes".to_string()
            } else {
                "🌐 No".to_string()
            },
            "N/A".to_string(),
        ];
    };
    let health_emoji = match status.health.as_str() {
        "healthy" => "✅",
        "degraded" => "⚠️",
        "unhealthy" => "❌",
        "unreachable" => "🔌",
        _ => "❓",
    };

    vec![
        config.name.clone(),
        config.port.to_string(),
        format!("{} {}", health_emoji, status.health),
        if status.tools_count > 0 {
            status.tools_count.to_string()
        } else {
            "-".to_string()
        },
        if let Some(ms) = status.latency_ms {
            format!("{ms}ms")
        } else {
            "-".to_string()
        },
        if config.oauth.required {
            "🔒 Yes".to_string()
        } else {
            "🌐 No".to_string()
        },
        format_connection_url(config, &status.state),
    ]
}

fn format_connection_url(config: &McpServerConfig, status: &str) -> String {
    if status == RUNNING {
        format!("http://{}:{}/mcp", config.host, config.port)
    } else {
        "N/A".to_string()
    }
}
