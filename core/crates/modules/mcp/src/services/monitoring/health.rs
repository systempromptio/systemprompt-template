use crate::services::client::McpConnectionResult;
use crate::McpServerConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::time::Duration;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use tokio::time::{interval, timeout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthStatus {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Unknown => "unknown",
        }
    }

    pub const fn emoji(&self) -> &str {
        match self {
            Self::Healthy => "✅",
            Self::Degraded => "⚠️",
            Self::Unhealthy => "❌",
            Self::Unknown => "❓",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub connection_result: Option<McpConnectionResult>,
    pub latency_ms: u32,
    pub details: HealthCheckDetails,
}

#[derive(Debug, Clone)]
pub struct HealthCheckDetails {
    pub service_name: String,
    pub tools_available: usize,
    pub requires_auth: bool,
    pub validation_type: String,
    pub error_message: Option<String>,
    pub server_version: Option<String>,
}

impl HealthCheckResult {
    pub fn from_connection_result(result: McpConnectionResult, config: &McpServerConfig) -> Self {
        // Determine health based on actual connectivity, not auth requirements
        let status = if result.success {
            // Service is responding successfully
            if result.connection_time_ms < 1000 {
                HealthStatus::Healthy
            } else {
                HealthStatus::Degraded // Slow but working
            }
        } else {
            // Check the type of failure
            match result.validation_type.as_str() {
                "auth_required" => {
                    // Service requires auth but IS responding - it's healthy!
                    HealthStatus::Healthy
                },
                "port_unavailable" | "connection_failed" | "timeout" => HealthStatus::Unhealthy,
                _ => HealthStatus::Unknown,
            }
        };

        let details = HealthCheckDetails {
            service_name: config.name.clone(),
            tools_available: result.tools_count,
            requires_auth: config.oauth.required,
            validation_type: result.validation_type.clone(),
            error_message: result.error_message.clone(),
            server_version: result.server_info.as_ref().map(|info| info.version.clone()),
        };

        Self {
            status,
            latency_ms: result.connection_time_ms,
            connection_result: Some(result),
            details,
        }
    }

    pub fn unhealthy(config: &McpServerConfig, error: String) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            connection_result: None,
            latency_ms: 0,
            details: HealthCheckDetails {
                service_name: config.name.clone(),
                tools_available: 0,
                requires_auth: config.oauth.required,
                validation_type: "error".to_string(),
                error_message: Some(error),
                server_version: None,
            },
        }
    }
}

pub async fn check_service_health(config: &McpServerConfig) -> Result<HealthStatus> {
    let result = perform_health_check(config).await?;
    Ok(result.status)
}

pub async fn perform_health_check(config: &McpServerConfig) -> Result<HealthCheckResult> {
    use crate::services::client::validate_connection_with_auth;

    let connection_result = timeout(
        Duration::from_secs(30),
        validate_connection_with_auth(
            &config.name,
            &config.host,
            config.port,
            config.oauth.required,
        ),
    )
    .await;

    match connection_result {
        Ok(Ok(mcp_result)) => Ok(HealthCheckResult::from_connection_result(
            mcp_result, config,
        )),
        Ok(Err(e)) => Ok(HealthCheckResult::unhealthy(
            config,
            format!("Connection error: {e}"),
        )),
        Err(_) => Ok(HealthCheckResult::unhealthy(
            config,
            "Health check timeout".to_string(),
        )),
    }
}

pub async fn monitor_health_continuously(
    config: &McpServerConfig,
    report_interval: Duration,
    db_pool: DbPool,
) -> Result<()> {
    let mut ticker = interval(report_interval);
    let log = LogService::system(db_pool);
    let mut previous_status: Option<HealthStatus> = None;
    let mut failure_count = 0;
    let mut last_failure_time: Option<DateTime<Utc>> = None;

    loop {
        ticker.tick().await;

        match perform_health_check(config).await {
            Ok(result) => {
                log_health_status(&result);

                match result.status {
                    HealthStatus::Unhealthy => {
                        failure_count += 1;
                        last_failure_time = Some(Utc::now());

                        if previous_status != Some(HealthStatus::Unhealthy) {
                            let degradation_reason = result
                                .details
                                .error_message
                                .as_deref()
                                .filter(|e| !e.is_empty())
                                .unwrap_or("[no error message]");
                            let _ = log.log(LogLevel::Info,
                                "mcp_health_monitor",
                                "MCP service health degraded",
                                Some(serde_json::json!({
                                    "service_name": config.name,
                                    "health_score": result.status.as_str(),
                                    "degradation_reason": degradation_reason,
                                    "impact_level": "high",
                                    "recovery_actions": ["restart_service", "check_port_availability"]
                                }))
                            ).await;
                        }

                        let error_msg = result
                            .details
                            .error_message
                            .as_deref()
                            .filter(|e| !e.is_empty())
                            .unwrap_or("[no error message]");
                        let _ = log
                            .error(
                                "mcp_health_monitor",
                                &format!("Service {} is unhealthy: {}", config.name, error_msg),
                            )
                            .await;
                    },
                    HealthStatus::Healthy => {
                        if previous_status == Some(HealthStatus::Unhealthy) && failure_count > 0 {
                            let downtime = if let Some(failure_time) = last_failure_time {
                                Utc::now().signed_duration_since(failure_time).num_seconds()
                            } else {
                                0
                            };

                            let _ = log
                                .log(
                                    LogLevel::Info,
                                    "mcp_health_monitor",
                                    "MCP service recovered",
                                    Some(serde_json::json!({
                                        "service_name": config.name,
                                        "downtime_duration": downtime,
                                        "recovery_method": "automatic",
                                        "health_score": "healthy",
                                        "failure_count": failure_count
                                    })),
                                )
                                .await;

                            failure_count = 0;
                            last_failure_time = None;
                        }
                    },
                    HealthStatus::Degraded => {
                        if previous_status == Some(HealthStatus::Healthy) {
                            let _ = log
                                .log(
                                    LogLevel::Info,
                                    "mcp_health_monitor",
                                    "MCP service performance degraded",
                                    Some(serde_json::json!({
                                        "service_name": config.name,
                                        "latency_ms": result.latency_ms,
                                        "performance_threshold_exceeded": true,
                                        "impact_level": "medium"
                                    })),
                                )
                                .await;
                        }
                    },
                    HealthStatus::Unknown => {},
                }

                previous_status = Some(result.status);
            },
            Err(e) => {
                let _ = log
                    .log(
                        LogLevel::Info,
                        "mcp_health_monitor",
                        "Health check failed",
                        Some(serde_json::json!({
                            "service_name": config.name,
                            "error": e.to_string(),
                            "check_type": "continuous_monitoring"
                        })),
                    )
                    .await;

                let _ = log
                    .error(
                        "mcp_health_monitor",
                        &format!("Health check failed for service {}: {}", config.name, e),
                    )
                    .await;
            },
        }
    }
}

const fn log_health_status(_result: &HealthCheckResult) {
    // Health status logging is now handled by audit logging above
    // This function is kept for API compatibility but does nothing
}
