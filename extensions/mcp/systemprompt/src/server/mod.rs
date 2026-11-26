//! The `systemprompt` toolset: the template's admin tools, hosted by the
//! composed binary through core's MCP toolset host.
//!
//! Authentication, RBAC, access recording and the `tools/list` filter are the
//! host's; this module lists the tools, dispatches a call to its handler and
//! serves the artifact-viewer shell resource.

mod overflow;
#[doc(hidden)]
pub mod tool;

use std::sync::Arc;

use async_trait::async_trait;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Icon, ListResourcesResult, ReadResourceResult, Tool};
use systemprompt::database::DbPool;
use systemprompt::extension::mcp::capabilities::WEBSITE_URL;
use systemprompt::extension::mcp::{McpToolset, ResourceList, ResourceRead, ToolsetCall};
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::host::ToolsetBuildResult;
use systemprompt::mcp::services::ui_renderer::UiRendererRegistry;
use systemprompt::mcp::{
    ArtifactIngest, ArtifactViewerConfig, McpArtifactRepository, build_artifact_viewer_resource,
    parse_artifact_resource_uri, read_artifact_resource, read_artifact_viewer_resource,
};
use systemprompt::system::HostContext;

use crate::cli::CliLocation;
use crate::tools;
use tool::dispatch_tool;

pub const TOOLSET_ID: &str = "systemprompt";

const INSTRUCTIONS: &str = "Admin control plane for this platform. Typed tools first: \
user_activity (per person: conversations, active days, skills, titles), conversation_list (one \
row per conversation), usage_by_user (spend per user), request_log (requests by user/window, \
cursor-paged), conversation_audit (one request's transcript, paged and bounded), users \
(roster), admin_report (dashboard). The 'systemprompt' tool runs any other CLI command: 'core \
skills list', 'core skills show <id>', 'admin users role promote <id>'. Never pass --json or \
--export.";

#[derive(Debug)]
pub struct SystempromptToolset {
    server: McpServerId,
    cli: CliLocation,
    ingest: Arc<ArtifactIngest>,
    artifacts: McpArtifactRepository,
    renderers: Arc<UiRendererRegistry>,
    shell: String,
}

impl SystempromptToolset {
    #[must_use]
    pub fn new(
        server: McpServerId,
        cli: CliLocation,
        ingest: Arc<ArtifactIngest>,
        artifacts: McpArtifactRepository,
        renderers: Arc<UiRendererRegistry>,
    ) -> Self {
        Self {
            server,
            cli,
            ingest,
            artifacts,
            renderers,
            shell: crate::reports::admin_artifact_shell(),
        }
    }

    fn viewer(&self) -> ArtifactViewerConfig<'_> {
        ArtifactViewerConfig {
            server_name: &self.server,
            title: "systemprompt.io Artifact Viewer",
            description: "Interactive UI viewer for systemprompt.io artifacts. Receives the tool \
                          result via the MCP Apps ui/notifications/tool-result protocol and mounts \
                          the server-rendered artifact HTML it carries.",
            template: &self.shell,
            icons: Some(vec![
                Icon::new(format!("{WEBSITE_URL}/files/images/favicon-32x32.png"))
                    .with_mime_type("image/png")
                    .with_sizes(vec!["32x32".to_owned()]),
            ]),
        }
    }
}

pub fn build(host: &HostContext, server: &McpServerId) -> ToolsetBuildResult {
    let ingest = host.state().get::<Arc<ArtifactIngest>>().map(Arc::clone)?;
    let db = host.state().get::<DbPool>()?;
    let renderers = host
        .state()
        .get::<Arc<UiRendererRegistry>>()
        .map(Arc::clone)?;
    Ok(Some(Box::new(SystempromptToolset::new(
        server.clone(),
        CliLocation::from_profile(host.profile()),
        ingest,
        McpArtifactRepository::new(db),
        renderers,
    ))))
}

#[async_trait]
impl McpToolset for SystempromptToolset {
    fn id(&self) -> &str {
        TOOLSET_ID
    }

    fn instructions(&self) -> &'static str {
        INSTRUCTIONS
    }

    fn tools(&self) -> Vec<Tool> {
        tools::list_tools(&self.server)
    }

    async fn call(&self, call: ToolsetCall<'_>) -> Result<CallToolResult, McpError> {
        let token = call
            .caller
            .context()
            .auth_token()
            .map(|token| token.as_str().to_owned())
            .ok_or_else(|| {
                McpError::internal_error("authenticated call has no bearer token", None)
            })?;
        let tool_name = call.request.name.to_string();
        Box::pin(dispatch_tool(
            &tool::Dispatch {
                service_id: self.server.as_str(),
                executor: call.executor,
                request: call.request,
                caller: call.caller,
                client: call.client,
                cli: &self.cli,
                ingest: &self.ingest,
            },
            &tool_name,
            &token,
        ))
        .await
    }

    fn serves_resources(&self) -> bool {
        true
    }

    async fn list_resources(&self, _list: ResourceList<'_>) -> ListResourcesResult {
        build_artifact_viewer_resource(&self.viewer())
    }

    async fn read_resource(
        &self,
        read: ResourceRead<'_>,
    ) -> Option<Result<ReadResourceResult, McpError>> {
        if parse_artifact_resource_uri(&read.request.uri).is_some() {
            return Some(
                read_artifact_resource(
                    read.request,
                    &self.server,
                    &self.artifacts,
                    &self.renderers,
                    read.caller.context().user_id(),
                )
                .await,
            );
        }
        Some(read_artifact_viewer_resource(
            read.request,
            &self.server,
            &self.shell,
        ))
    }
}
