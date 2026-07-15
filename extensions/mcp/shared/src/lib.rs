//! Shared persistence helpers for MCP extension crates.
//!
//! Both functions exposed here ([`record_mcp_access`] and
//! [`record_mcp_access_rejected`]) are best-effort: they log a `tracing::warn!`
//! and return on any DB failure, so an MCP request that has already cleared
//! authz is never blocked by an audit-row insert. Callers do not need to
//! propagate errors.

use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;

mod repositories;

use repositories::{
    McpAccessParams, find_anonymous_user_id, insert_mcp_access, insert_mcp_access_rejection,
};

const ACTION_USED: &str = "used";

/// Maximum length (in bytes) of the reason text kept in a rejection
/// description before it is truncated. Truncated text gains a "..." suffix, so
/// the reason portion never exceeds `MAX_REASON_LEN + 3` bytes.
#[doc(hidden)]
pub const MAX_REASON_LEN: usize = 117;

/// Truncate `s` to at most `max_bytes`, appending "..." when truncation
/// occurred. The cut is snapped down to a UTF-8 char boundary so it can never
/// split a multi-byte codepoint (which would panic on a byte slice).
///
/// Exposed (behind `#[doc(hidden)]`) so the external test workspace can assert
/// the char-boundary and "..." suffix semantics directly; not part of the
/// public API.
#[doc(hidden)]
pub fn truncate_on_char_boundary(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_owned();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &s[..end])
}

pub async fn record_mcp_access(
    pool: &DbPool,
    user_id: &UserId,
    server: &str,
    tool: &str,
    action: &str,
) {
    let Some(pg_pool) = pool.pool() else {
        tracing::warn!("No PgPool available to record MCP access event");
        return;
    };
    let description = match action {
        "authenticated" => format!("Authenticated to {server} for '{tool}'"),
        ACTION_USED => format!("Executed '{tool}' on {server}"),
        _ => format!("{action} on {server}"),
    };
    let entity_type = if action == ACTION_USED {
        "tool"
    } else {
        "mcp_server"
    };
    let entity_name = if action == ACTION_USED { tool } else { server };
    let metadata = serde_json::json!({ "tool_name": tool, "server": server });

    let params = McpAccessParams {
        user_id,
        action,
        entity_type,
        entity_name,
        description: &description,
        metadata: &metadata,
    };

    if let Err(e) = insert_mcp_access(pg_pool.as_ref(), &params).await {
        tracing::warn!(error = %e, "Failed to record MCP access event (non-fatal)");
    }
}

pub async fn record_mcp_access_rejected(pool: &DbPool, server: &str, tool: &str, reason: &str) {
    let Some(pg_pool) = pool.pool() else {
        tracing::warn!("No PgPool available to record MCP access rejection");
        return;
    };
    let reason_text = truncate_on_char_boundary(reason, MAX_REASON_LEN);
    let description = format!("Access rejected on {server}: {reason_text}");
    let metadata = serde_json::json!({ "tool_name": tool, "server": server, "reason": reason });

    let anonymous_user_id = match find_anonymous_user_id(pg_pool.as_ref()).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            tracing::error!(
                server,
                tool,
                "Dropping MCP access-rejection audit row: no anonymous principal exists to \
                 attribute it to (refusing to attribute a rejection to an arbitrary user)"
            );
            return;
        },
        Err(e) => {
            tracing::error!(error = %e, server, tool, "Failed to resolve anonymous principal for MCP access-rejection audit; dropping row");
            return;
        },
    };

    if let Err(e) = insert_mcp_access_rejection(
        pg_pool.as_ref(),
        &anonymous_user_id,
        server,
        &description,
        &metadata,
    )
    .await
    {
        tracing::warn!(error = %e, "Failed to record MCP access rejection (non-fatal)");
    }
}
