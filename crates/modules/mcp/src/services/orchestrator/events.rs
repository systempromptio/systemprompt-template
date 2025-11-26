use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpEvent {
    ServiceStartRequested {
        service_name: String,
    },
    ServiceStarted {
        service_name: String,
        process_id: u32,
        port: u16,
    },
    ServiceFailed {
        service_name: String,
        error: String,
    },
    ServiceStopped {
        service_name: String,
        exit_code: Option<i32>,
    },
    HealthCheckFailed {
        service_name: String,
        reason: String,
    },
    SchemaUpdated {
        service_name: String,
        tool_count: usize,
    },
    ServiceRestartRequested {
        service_name: String,
        reason: String,
    },
}

impl McpEvent {
    pub fn service_name(&self) -> &str {
        match self {
            Self::ServiceStartRequested { service_name }
            | Self::ServiceStarted { service_name, .. }
            | Self::ServiceFailed { service_name, .. }
            | Self::ServiceStopped { service_name, .. }
            | Self::HealthCheckFailed { service_name, .. }
            | Self::SchemaUpdated { service_name, .. }
            | Self::ServiceRestartRequested { service_name, .. } => service_name,
        }
    }

    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::ServiceStartRequested { .. } => "service_start_requested",
            Self::ServiceStarted { .. } => "service_started",
            Self::ServiceFailed { .. } => "service_failed",
            Self::ServiceStopped { .. } => "service_stopped",
            Self::HealthCheckFailed { .. } => "health_check_failed",
            Self::SchemaUpdated { .. } => "schema_updated",
            Self::ServiceRestartRequested { .. } => "service_restart_requested",
        }
    }
}
