//! The live check a session-attested connector runs in place of an OAuth
//! probe: the caller's signed-in scope, then whether the MCP server answers.

use crate::env::AdminEnv;

use std::time::Instant;

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::mcp::services::client::{
    McpConnectionResult, validate_connection, validate_connection_by_url,
};
use systemprompt::models::auth::Permission;
use systemprompt::models::mcp::McpServerType;

use super::connector_accounts::{load_entitlement, scopes_granted};
use super::connector_oauth::{Provider, VerificationReport};
use crate::error::{AdminError, AdminResult};

// Why: a session-attested connector has no grant to open. Its check is the
// scope the caller signed in with and whether the server answers, reported in
// the same steps the Connectors page draws for an OAuth probe.
pub(crate) async fn probe_session_connection(
    env: &AdminEnv,
    pool: &PgPool,
    user: &UserId,
    provider: &Provider,
) -> AdminResult<VerificationReport> {
    let services = env.services();
    let scopes = provider
        .session_scopes(&services)
        .ok_or_else(|| AdminError::BadRequest("Not a session-attested connector".into()))?;
    let server = services
        .mcp_servers
        .get(provider.slug())
        .filter(|server| server.enabled)
        .ok_or_else(|| AdminError::Unavailable("Connector not configured".into()))?;
    let mut report = VerificationReport::for_provider(provider.slug());
    let started = Instant::now();
    let entitlement = load_entitlement(pool, user).await?;
    let granted = entitlement.active && scopes_granted(&entitlement.roles, scopes);
    let scope_names = scopes
        .iter()
        .map(Permission::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    let outcome = if granted {
        Ok(format!("Signed in with scope {scope_names}"))
    } else {
        Err(AdminError::Forbidden(format!(
            "Your session does not carry scope {scope_names}"
        )))
    };
    report.record("token", started, &outcome);
    if outcome.is_err() {
        return Ok(report.finish());
    }
    let started = Instant::now();
    let upstream =
        |error: systemprompt::mcp::McpDomainError| AdminError::Upstream(error.to_string());
    let probe: AdminResult<McpConnectionResult> =
        match (server.server_type, server.port, server.endpoint.as_deref()) {
            (McpServerType::Internal, Some(port), _) => validate_connection(
                &systemprompt::identifiers::ServiceName::new(provider.slug()),
                "127.0.0.1",
                port,
            )
            .await
            .map_err(upstream),
            (McpServerType::External, _, Some(endpoint)) => validate_connection_by_url(
                &systemprompt::identifiers::ServiceName::new(provider.slug()),
                endpoint,
            )
            .await
            .map_err(upstream),
            _ => Err(AdminError::Unavailable(
                "Connector declares neither a port nor an endpoint".into(),
            )),
        };
    record_server_probe(&mut report, started, probe);
    Ok(report.finish())
}

fn record_server_probe(
    report: &mut VerificationReport,
    started: Instant,
    probe: AdminResult<McpConnectionResult>,
) {
    let reachable = match &probe {
        Ok(result) if result.success => Ok(format!(
            "Server answered in {} ms",
            result.connection_time_ms
        )),
        Ok(result) => Err(AdminError::Upstream(
            result
                .error_message
                .clone()
                .unwrap_or_else(|| "Server did not answer".to_owned()),
        )),
        Err(error) => Err(AdminError::Upstream(error.to_string())),
    };
    report.record("initialize", started, &reachable);
    if let (Ok(()), Ok(result)) = (reachable.map(|_| ()), probe) {
        let tools = result.tools_count.map_or_else(
            || "Tool list is behind the server's own authorization".to_owned(),
            |count| format!("{count} tools available"),
        );
        report.record("tools", started, &Ok(tools));
    }
}
