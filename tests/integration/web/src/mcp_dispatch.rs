//! Per-call dispatch for the bundled `systemprompt` MCP server, driven
//! directly.
//!
//! `call_tool` cannot be reached from a test process — it takes an rmcp
//! `RequestContext<RoleServer>` whose `Peer` only exists once a transport is
//! serving. `dispatch_tool` is the seam below it: it takes the already
//! authenticated core `RequestContext`, so every branch `call_tool` can reach
//! is reachable here, including the unknown-tool arm.
//!
//! A live pool is needed because `McpToolExecutor` records every call through
//! a `ToolUsageRepository` and persists the artifact it returns.

use std::sync::Arc;

use rmcp::model::CallToolRequestParams;
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId};
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{McpArtifactRepository, McpToolExecutor};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

use crate::tempdb::TempDb;

fn executor(pool: &Arc<PgPool>, server_name: &str) -> McpToolExecutor {
    let db_pool = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let usage = Arc::new(ToolUsageRepository::new(&db_pool).expect("tool usage repository"));
    let artifacts = Arc::new(McpArtifactRepository::new(&db_pool).expect("artifact repository"));
    McpToolExecutor::new(usage, artifacts, server_name)
}

fn request_context() -> SysRequestContext {
    SysRequestContext::new(
        SessionId::new("dispatch-session"),
        TraceId::new("dispatch-trace"),
        ContextId::new("00000000-0000-4000-8000-00000000d15b"),
        AgentName::new("dispatch-agent"),
    )
}

fn call(tool: &'static str, arguments: serde_json::Value) -> CallToolRequestParams {
    let object = arguments
        .as_object()
        .expect("tool arguments are a JSON object")
        .clone();
    CallToolRequestParams::new(tool).with_arguments(object)
}

#[tokio::test]
async fn an_unknown_systemprompt_tool_points_the_caller_at_the_cli_skill() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let executor = executor(&db.pool, "systemprompt");

    let error = systemprompt_mcp_agent::server::tool::dispatch_tool(
        &executor,
        "not_a_tool",
        &call("not_a_tool", serde_json::json!({})),
        &request_context(),
        "unused-token",
    )
    .await
    .expect_err("an unknown tool name is refused");

    assert!(
        error.message.contains("systemprompt_cli"),
        "the refusal routes the caller to the CLI skill: {}",
        error.message
    );

    db.cleanup().await;
}

// `SystempromptToolHandler::handle` is deliberately not driven here: it shells
// out to the real `systemprompt` binary with the caller's bearer token, so a
// test that reached it would be running the CLI against whatever profile the
// machine has configured. Only the dispatch arm around it is asserted.
