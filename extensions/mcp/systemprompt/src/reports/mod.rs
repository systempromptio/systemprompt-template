//! Deterministic reports over this platform's OWN data.
//!
//! The rule this module now follows: wrap our own data, never someone else's
//! MCP server. An earlier version reached Atlassian's hosted MCP from here and
//! translated its payloads, which produced four wire-shape defects in a single
//! afternoon — an undocumented `data` envelope, `cloudId` where the reader
//! expected `id`, a site URL that is never sent being treated as fatal, and a
//! `view` preset that silently withheld the status field every metric depended
//! on. The last one scored a 76-issue project GREEN.
//!
//! Jira and Confluence are reached by the client directly, through the
//! `atlassian` server it already holds, and analysed by the `admin_daily_brief`
//! and `admin_critical_projects` skills, which carry the thresholds. What stays
//! here is what has no third party in the path: the CLI and the audit tables.
//!
//! The one artifact that survives is `admin-ai-usage`, and it survives because
//! its data is deterministic: our own audit tables, no third party, exact
//! rather than sampled. A dashboard over data we cannot vouch for is worse than
//! no dashboard, which is why the Jira traffic lights are not one.

mod handler;
mod platform;
mod renderer;
mod shape;

pub use handler::ReportHandler;
pub use platform::{normalize_cli, with_dollar_siblings};
pub use renderer::admin_artifact_shell;
pub use shape::{ReportInput, ReportKind, ReportOutput, ReportTable, SourceStatus};

pub(crate) fn invalid(message: impl Into<String>) -> rmcp::ErrorData {
    rmcp::ErrorData::invalid_params(message.into(), None)
}

pub(crate) fn failure(message: impl Into<String>) -> rmcp::ErrorData {
    rmcp::ErrorData::internal_error(message.into(), None)
}
