//! Authenticated report handlers dispatched through the existing MCP executor.

use super::{ReportInput, ReportOutput};
use crate::cli::CliLocation;
use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

#[derive(Debug, Clone, Copy)]
pub struct ReportHandler<'a> {
    pub cli: &'a CliLocation,
    pub token: &'a str,
}

impl McpToolHandler for ReportHandler<'_> {
    type Input = ReportInput;
    type Output = ReportOutput;
    fn tool_name(&self) -> &'static str {
        "admin_report"
    }
    fn description(&self) -> &'static str {
        "Read this platform's own AI usage and cost: spend, models, trends and sessions over a chosen window, with source coverage and an interactive dashboard. Reads the audit tables through the CLI; performs no administrative change. Jira and Confluence are NOT read here — use the atlassian server directly for those."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: ReportInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(ReportOutput, String), ErrorData> {
        let output = super::platform::run(&input, self.cli, self.token).await?;
        let summary = format!(
            "{} — {}. {}",
            output.title,
            output.checked_at,
            if output.complete {
                "Sources loaded"
            } else {
                "Partial data; inspect source warnings"
            }
        );
        Ok((output, summary))
    }
}
