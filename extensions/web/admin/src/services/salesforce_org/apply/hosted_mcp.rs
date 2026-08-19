//! Activate the org's standard hosted MCP servers.
//!
//! `McpServerAccess` is a Tooling object, `updateable: true` from API version
//! 67.0, so activation is a PATCH rather than the Setup click it used to be.

use super::ApplyReport;
use crate::handlers::salesforce_auth::SalesforceError;
use crate::services::salesforce_org::client::Connection;
use crate::services::salesforce_org::spec::OrgSpec;

/// Switch on the hosted MCP servers the spec wants active.
///
/// Additive like the rest of apply: a server the spec marks inactive is left
/// alone, and a server active in the org but absent from the spec is not
/// touched.
///
/// # Errors
/// Propagates the Tooling query failure. A PATCH Salesforce rejects is recorded
/// as a follow-up rather than aborting.
// JSON: Salesforce Tooling query rows are read raw — no fixed schema.
pub async fn apply_hosted_mcp_servers(
    conn: &Connection,
    spec: &OrgSpec,
    report: &mut ApplyReport,
) -> Result<(), SalesforceError> {
    if spec.hosted_mcp_servers.is_empty() {
        return Ok(());
    }
    let rows = conn
        .tooling_soql("SELECT Id,DeveloperName,Active FROM McpServerAccess")
        .await?;

    for server in &spec.hosted_mcp_servers {
        let Some(row) = rows.iter().find(|r| {
            r.get("DeveloperName").and_then(serde_json::Value::as_str)
                == Some(server.developer_name.as_str())
        }) else {
            // Why: an error, not a follow-up to shrug at. Absence means the org
            // does not offer this server at all, which activation cannot fix.
            report.manual_followups.push(format!(
                "hosted MCP server '{}' ({}) is not present in this org — no \
                 McpServerAccess record named {}. The org does not offer it; \
                 activation cannot fix that.",
                server.name, server.endpoint, server.developer_name
            ));
            continue;
        };
        // JSON: Salesforce Tooling query rows — no fixed schema.
        let active = row
            .get("Active")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if active || !server.active {
            continue;
        }
        let Some(id) = row.get("Id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        match conn
            .update_sobject(
                "McpServerAccess",
                id,
                &serde_json::json!({ "Active": true }),
                true,
            )
            .await
        {
            Ok(()) => report.servers_activated.push(server.name.clone()),
            Err(e) => report.manual_followups.push(format!(
                "could not activate hosted MCP server '{}': {e}",
                server.name
            )),
        }
    }
    Ok(())
}
