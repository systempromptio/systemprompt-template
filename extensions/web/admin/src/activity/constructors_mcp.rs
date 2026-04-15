use super::constructors::truncate;
use super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::types::{ActivityEntityRef, NewActivity};

impl NewActivity {
    #[must_use]
    pub fn mcp_access_granted(user_id: &str, server_name: &str, tool_name: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::McpAccess,
            action: ActivityAction::Authenticated,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::McpServer,
                entity_id: None,
                entity_name: Some(server_name.to_string()),
            }),
            description: format!("Authenticated to {server_name} for '{tool_name}'"),
            metadata: serde_json::json!({ "tool_name": tool_name, "server": server_name }),
        }
    }

    #[must_use]
    pub fn mcp_access_rejected(server_name: &str, tool_name: &str, reason: &str) -> Self {
        let reason_short = truncate(reason, 120);
        Self {
            user_id: "unknown".to_string(),
            category: ActivityCategory::McpAccess,
            action: ActivityAction::Rejected,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::McpServer,
                entity_id: None,
                entity_name: Some(server_name.to_string()),
            }),
            description: format!("Access rejected on {server_name}: {reason_short}"),
            metadata: serde_json::json!({ "tool_name": tool_name, "server": server_name, "reason": reason }),
        }
    }

    #[must_use]
    pub fn mcp_tool_executed(user_id: &str, server_name: &str, tool_name: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::McpAccess,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Tool,
                entity_id: None,
                entity_name: Some(tool_name.to_string()),
            }),
            description: format!("Executed '{tool_name}' on {server_name}"),
            metadata: serde_json::json!({ "tool_name": tool_name, "server": server_name }),
        }
    }
}
