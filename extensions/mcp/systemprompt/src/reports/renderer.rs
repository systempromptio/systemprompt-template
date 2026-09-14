//! Render the same maintained assets in MCP Apps and the Cowork library.

use async_trait::async_trait;
use systemprompt::mcp::McpDomainResult;
use systemprompt::mcp::services::ui_renderer::templates::DashboardRenderer;
use systemprompt::mcp::services::ui_renderer::{UiRenderer, UiRendererRegistration, UiResource};
use systemprompt::models::a2a::{Artifact, Part};
use systemprompt::models::artifacts::ArtifactType;

struct AdminRenderer;

inventory::submit! {
    UiRendererRegistration { name: "systemprompt-admin-reports", factory: || std::sync::Arc::new(AdminRenderer) }
}

#[async_trait]
impl UiRenderer for AdminRenderer {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::Dashboard
    }

    async fn render(&self, artifact: &Artifact) -> McpDomainResult<UiResource> {
        // Why: report the deserialisation failure instead of swallowing it.
        // Falling through to DashboardRenderer looked like a safe default, but
        // that renderer expects a DashboardArtifact and cannot read a
        // ReportOutput — so it errored, core dropped the embedded resource, and
        // the tool returned success with no artifact and nothing to say why.
        let mut failure: Option<String> = None;
        let report = artifact.parts.iter().find_map(|part| {
            let Part::Data(data) = part else {
                return None;
            };
            // JSON: A2A data parts carry the serialized report protocol object.
            match serde_json::from_value::<super::ReportOutput>(serde_json::Value::Object(
                data.data.clone(),
            )) {
                Ok(report) => Some(report),
                Err(error) => {
                    failure = Some(error.to_string());
                    None
                },
            }
        });
        let Some(report) = report else {
            if let Some(error) = failure {
                return Err(systemprompt::mcp::McpDomainError::Internal(format!(
                    "admin report artifact does not match the ReportOutput contract: {error}"
                )));
            }
            return DashboardRenderer::new().render(artifact).await;
        };
        let template = include_str!("../../../../../services/artifacts/admin-ai-usage/view.html");
        let data = serde_json::to_string(&report)
            .map_err(|e| systemprompt::mcp::McpDomainError::Internal(e.to_string()))?
            .replace('<', "\\u003c")
            .replace('>', "\\u003e")
            .replace('&', "\\u0026");
        Ok(UiResource::new(
            template.replace("/*REPORT_DATA*/null", &data).replace(
                "/*MCP_BRIDGE*/",
                include_str!("../../../../../storage/files/js/admin-report-bridge.js"),
            ),
        ))
    }
}

#[must_use]
pub fn admin_artifact_shell() -> String {
    systemprompt::mcp::artifact_shell_template().replace(
        "    request(MCP_UI.INITIALIZE, {",
        &format!(
            "{}\n    request(MCP_UI.INITIALIZE, {{",
            include_str!("../../../../../storage/files/js/admin-report-shell.js")
        ),
    )
}
