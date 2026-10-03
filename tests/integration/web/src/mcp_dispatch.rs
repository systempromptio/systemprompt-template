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
use systemprompt::mcp::{ArtifactIngest, McpToolExecutor};

use crate::tempdb::TempDb;

fn db_pool(pool: &Arc<PgPool>) -> systemprompt::database::DbPool {
    Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ))
}

fn executor(db_pool: &systemprompt::database::DbPool, server_name: &str) -> McpToolExecutor {
    let usage = Arc::new(ToolUsageRepository::new(db_pool));
    let ingest = Arc::new(ArtifactIngest::new(
        systemprompt::mcp::repository::ArtifactIngestRepositories::new(db_pool),
        None,
    ));
    McpToolExecutor::new(
        usage,
        Arc::new(systemprompt::ai::repository::AiRequestRepository::new(
            db_pool,
        )),
        ingest,
        systemprompt::identifiers::McpServerId::new(server_name),
    )
}

fn request_context() -> systemprompt::models::execution::context::RequestContext {
    systemprompt::models::execution::context::RequestContext::new(
        SessionId::new("dispatch-session"),
        TraceId::new("dispatch-trace"),
        ContextId::try_new("00000000-0000-4000-8000-00000000d15b")
            .expect("valid fixture identifier"),
        AgentName::try_new("dispatch-agent").expect("valid fixture agent name"),
        systemprompt::identifiers::Actor::anonymous(systemprompt::identifiers::UserId::generate()),
    )
}

fn call(tool: &'static str, arguments: serde_json::Value) -> CallToolRequestParams {
    let object = arguments
        .as_object()
        .expect("tool arguments are a JSON object")
        .clone();
    CallToolRequestParams::new(tool).with_arguments(object)
}

fn client() -> systemprompt::mcp::ClientProfile {
    systemprompt::mcp::ClientProfile {
        protocol_version: Some(rmcp::model::ProtocolVersion::V_2025_06_18),
        ..systemprompt::mcp::ClientProfile::default()
    }
}

#[tokio::test]
async fn an_unknown_systemprompt_tool_points_the_caller_at_the_cli_skill() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let db_pool = db_pool(&db.pool);
    let executor = executor(&db_pool, "systemprompt");
    let ingest = Arc::new(ArtifactIngest::new(
        systemprompt::mcp::repository::ArtifactIngestRepositories::new(&db_pool),
        None,
    ));

    let request = call("not_a_tool", serde_json::json!({}));
    let profile = client();
    let error = systemprompt_mcp_agent::server::tool::dispatch_tool(
        &systemprompt_mcp_agent::server::tool::Dispatch {
            service_id: "systemprompt",
            db_pool: &db_pool,
            executor: &executor,
            request: &request,
            request_context: &request_context(),
            client: &profile,
            ingest: &ingest,
            // Why: these dispatch paths return before any handler is built, so
            // the location is never read.
            cli: &systemprompt_mcp_agent::CliLocation {
                bin: std::path::PathBuf::from("/nonexistent"),
                workdir: std::path::PathBuf::from("/nonexistent"),
            },
        },
        "not_a_tool",
        "unused-token",
    )
    .await
    .expect_err("an unknown tool name is refused");

    assert!(
        error.message.contains("systemprompt_cli"),
        "the refusal routes the caller to the CLI skill: {}",
        error.message
    );
    assert!(
        error.message.contains("request_log") && error.message.contains("conversation_audit"),
        "the refusal names the typed tools: {}",
        error.message
    );

    db.cleanup().await;
}

// `SystempromptToolHandler::handle` is deliberately not driven here: it shells
// out to the real `systemprompt` binary with the caller's bearer token, so a
// test that reached it would be running the CLI against whatever profile the
// machine has configured. Only the dispatch arm around it is asserted.
