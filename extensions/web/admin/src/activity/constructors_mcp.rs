use serde::Serialize;

use super::constructors::truncate;
use super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::types::{ActivityEntityRef, NewActivity};

#[derive(Debug, Serialize)]
struct McpAccessMeta<'a> {
    tool_name: &'a str,
    server: &'a str,
}

#[derive(Debug, Serialize)]
struct McpAccessRejectedMeta<'a> {
    tool_name: &'a str,
    server: &'a str,
    reason: &'a str,
}

impl NewActivity {
    #[must_use]
    pub fn mcp_access_granted(user_id: &str, server_name: &str, tool_name: &str) -> Self {
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::McpAccess,
            action: ActivityAction::Authenticated,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::McpServer,
                id: None,
                name: Some(server_name.to_owned()),
            }),
            description: format!("Authenticated to {server_name} for '{tool_name}'"),
            metadata: serde_json::to_value(McpAccessMeta {
                tool_name,
                server: server_name,
            })
            .unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn mcp_access_rejected(server_name: &str, tool_name: &str, reason: &str) -> Self {
        let reason_short = truncate(reason, 120);
        Self {
            user_id: "unknown".to_owned(),
            category: ActivityCategory::McpAccess,
            action: ActivityAction::Rejected,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::McpServer,
                id: None,
                name: Some(server_name.to_owned()),
            }),
            description: format!("Access rejected on {server_name}: {reason_short}"),
            metadata: serde_json::to_value(McpAccessRejectedMeta {
                tool_name,
                server: server_name,
                reason,
            })
            .unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn mcp_tool_executed(user_id: &str, server_name: &str, tool_name: &str) -> Self {
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::McpAccess,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Tool,
                id: None,
                name: Some(tool_name.to_owned()),
            }),
            description: format!("Executed '{tool_name}' on {server_name}"),
            metadata: serde_json::to_value(McpAccessMeta {
                tool_name,
                server: server_name,
            })
            .unwrap_or_default(),
        }
    }
}
