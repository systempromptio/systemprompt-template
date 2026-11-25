//! The `systemprompt_mcp` composition member: contributes the `systemprompt`
//! toolset, the admin report renderer and the MCP access recorder.

use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::database::DbPool;
use systemprompt::extension::mcp::{
    AccessRecorderSlot, McpAccessEvent, McpAccessOutcome, McpAccessRecorder,
    SharedMcpAccessRecorder,
};
use systemprompt::extension::prelude::*;
use systemprompt::mcp::contract::UiSlot;
use systemprompt::mcp::host::ToolsetSlot;
use systemprompt::mcp::services::ui_renderer::UiRenderer;
use systemprompt_mcp_shared::{record_mcp_access, record_mcp_access_rejected};

use crate::reports::AdminRenderer;
use crate::server::{TOOLSET_ID, build};

/// The template's MCP member.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystempromptMcpExtension;

impl Extension for SystempromptMcpExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "systemprompt_mcp",
            name: "systemprompt MCP toolset",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn contribute(&self, c: &mut Contributions<'_>) -> Result<(), ContributionError> {
        c.slot::<ToolsetSlot>().toolset(TOOLSET_ID, build);
        c.slot::<UiSlot>().renderer(
            |_r: &Resolved<'_>| -> Result<Arc<dyn UiRenderer>, FactoryError> {
                Ok(Arc::new(AdminRenderer))
            },
        );
        c.slot::<AccessRecorderSlot>().recorder(
            |r: &Resolved<'_>| -> Result<SharedMcpAccessRecorder, FactoryError> {
                Ok(Arc::new(UserActivityRecorder {
                    db: r.get::<DbPool>()?,
                }))
            },
        );
        Ok(())
    }
}

#[derive(Debug)]
struct UserActivityRecorder {
    db: DbPool,
}

#[async_trait]
impl McpAccessRecorder for UserActivityRecorder {
    async fn record(&self, event: McpAccessEvent<'_>) {
        match (event.outcome, event.user_id) {
            (McpAccessOutcome::Rejected { reason } | McpAccessOutcome::Forbidden { reason }, _) => {
                record_mcp_access_rejected(&self.db, event.server.as_str(), event.tool, reason)
                    .await;
            },
            (outcome, Some(user_id)) => {
                record_mcp_access(
                    &self.db,
                    user_id,
                    event.server.as_str(),
                    event.tool,
                    outcome.action(),
                )
                .await;
            },
            (_, None) => {},
        }
    }
}
