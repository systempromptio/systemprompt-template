//! Per-call dispatch for both bundled MCP servers, driven directly.
//!
//! `call_tool` cannot be reached from a test process — it takes an rmcp
//! `RequestContext<RoleServer>` whose `Peer` only exists once a transport is
//! serving. `dispatch_tool` is the seam below it: it takes the already
//! authenticated core `RequestContext`, so every branch `call_tool` can reach
//! is reachable here, including the admin gate on `upload_document` and both
//! unknown-tool arms.
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
use systemprompt::models::auth::{AuthenticatedUser, Permission};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt_mcp_knowledge_bank::store::KnowledgeStore;
use systemprompt_mcp_knowledge_bank::tools::{TOOL_LIST, TOOL_SEARCH, TOOL_UPLOAD};

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

fn user_with(permission: Permission) -> AuthenticatedUser {
    AuthenticatedUser::new(
        uuid::Uuid::new_v4(),
        "dispatcher".to_owned(),
        "dispatcher@example.com".to_owned(),
        vec![permission],
    )
}

fn call(tool: &'static str, arguments: serde_json::Value) -> CallToolRequestParams {
    let object = arguments
        .as_object()
        .expect("tool arguments are a JSON object")
        .clone();
    CallToolRequestParams::new(tool).with_arguments(object)
}

// The rendered body is not in the text content block — that block carries the
// one-line summary, and the artifact the handler built is serialised into the
// structured result alongside an HTML rendering of it.
fn body_of(result: &rmcp::model::CallToolResult) -> String {
    result
        .structured_content
        .as_ref()
        .and_then(|v| v.pointer("/content"))
        .and_then(|v| v.as_str())
        .expect("the executor returns the handler's artifact as structured content")
        .to_owned()
}

fn summary_of(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|block| block.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

async fn dispatch_kb(
    db: &TempDb,
    store: &Arc<KnowledgeStore>,
    ctx: &SysRequestContext,
    tool: &'static str,
    arguments: serde_json::Value,
) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
    let executor = executor(&db.pool, "knowledge-bank");
    let request = call(tool, arguments);
    let profile = client();
    systemprompt_mcp_knowledge_bank::server::tool::dispatch_tool(
        &systemprompt_mcp_knowledge_bank::server::tool::Dispatch {
            executor: &executor,
            request: &request,
            request_context: ctx,
            client: &profile,
        },
        store,
        tool,
    )
    .await
}

fn client() -> systemprompt::mcp::ClientProfile {
    systemprompt::mcp::ClientProfile {
        protocol_version: Some(rmcp::model::ProtocolVersion::V_2025_06_18),
        ..systemprompt::mcp::ClientProfile::default()
    }
}


fn seeded_store() -> Arc<KnowledgeStore> {
    Arc::new(KnowledgeStore::seeded().expect("the bundled fixtures parse"))
}

#[tokio::test]
async fn search_dispatch_returns_the_matching_documents_as_text() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let store = seeded_store();
    let any_document = store
        .list_documents(None, None)
        .first()
        .expect("the bundled fixtures seed at least one document")
        .clone();

    let result = dispatch_kb(
        &db,
        &store,
        &request_context(),
        TOOL_SEARCH,
        serde_json::json!({ "query": any_document.title }),
    )
    .await
    .expect("search dispatches to its handler");

    assert!(
        body_of(&result).contains(&any_document.title),
        "the matched document's title is rendered into the response body"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn search_dispatch_returns_the_sentinel_when_nothing_matches() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = dispatch_kb(
        &db,
        &seeded_store(),
        &request_context(),
        TOOL_SEARCH,
        serde_json::json!({ "query": "zzzz-no-document-mentions-this-zzzz" }),
    )
    .await
    .expect("a search that matches nothing is still a successful call");

    assert!(
        body_of(&result).contains("No matching documents"),
        "an empty result set renders the sentinel, not an empty body"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn list_dispatch_renders_one_line_per_document() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let store = seeded_store();
    let expected = store.list_documents(None, None).len();

    let result = dispatch_kb(
        &db,
        &store,
        &request_context(),
        TOOL_LIST,
        serde_json::json!({}),
    )
    .await
    .expect("list dispatches to its handler");

    assert_eq!(
        body_of(&result).lines().count(),
        expected,
        "every seeded document gets exactly one listing line"
    );
    assert_eq!(
        summary_of(&result),
        format!("{expected} document(s) in the knowledge bank"),
        "the summary the model reads counts the same documents the body lists"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn list_dispatch_reports_the_empty_filter_sentinel() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = dispatch_kb(
        &db,
        &seeded_store(),
        &request_context(),
        TOOL_LIST,
        serde_json::json!({ "project_id": "no-such-project" }),
    )
    .await
    .expect("a filter that matches nothing is still a successful call");

    assert!(
        body_of(&result).contains("holds no documents matching the filter"),
        "a filter with no matches renders the sentinel"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn upload_dispatch_is_refused_without_the_admin_role() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let ctx = request_context().with_user(user_with(Permission::User));

    let error = dispatch_kb(
        &db,
        &seeded_store(),
        &ctx,
        TOOL_UPLOAD,
        serde_json::json!({
            "doc_type": "workshop",
            "project_id": "acme",
            "title": "Rejected",
            "content": "should never be stored",
        }),
    )
    .await
    .expect_err("a non-admin cannot upload");

    assert!(
        error.message.contains("requires the admin role"),
        "the refusal names the missing role: {}",
        error.message
    );

    db.cleanup().await;
}

#[tokio::test]
async fn upload_dispatch_is_refused_when_the_context_carries_no_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let error = dispatch_kb(
        &db,
        &seeded_store(),
        &request_context(),
        TOOL_UPLOAD,
        serde_json::json!({
            "doc_type": "workshop",
            "project_id": "acme",
            "title": "Rejected",
            "content": "should never be stored",
        }),
    )
    .await
    .expect_err("an unauthenticated context cannot upload");

    assert!(error.message.contains("requires the admin role"));

    db.cleanup().await;
}

#[tokio::test]
async fn upload_dispatch_stores_the_document_for_an_admin() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let store = seeded_store();
    let before = store.count();
    let ctx = request_context().with_user(user_with(Permission::Admin));

    let result = dispatch_kb(
        &db,
        &store,
        &ctx,
        TOOL_UPLOAD,
        serde_json::json!({
            "doc_type": "workshop",
            "project_id": "acme",
            "title": "Kickoff Notes",
            "content": "Decisions from the kickoff.",
        }),
    )
    .await
    .expect("an admin may upload");

    assert_eq!(
        store.count(),
        before + 1,
        "the uploaded document joins the store"
    );
    assert!(
        body_of(&result).contains("workshop-kickoff-notes"),
        "the response reports the slugified id it assigned: {}",
        body_of(&result)
    );

    db.cleanup().await;
}

#[tokio::test]
async fn an_unknown_knowledge_bank_tool_lists_the_three_it_serves() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let error = dispatch_kb(
        &db,
        &seeded_store(),
        &request_context(),
        "delete_everything",
        serde_json::json!({}),
    )
    .await
    .expect_err("an unknown tool name is refused");

    for expected in [TOOL_SEARCH, TOOL_LIST, TOOL_UPLOAD] {
        assert!(
            error.message.contains(expected),
            "the refusal names the available tool {expected}: {}",
            error.message
        );
    }

    db.cleanup().await;
}

#[tokio::test]
async fn an_unknown_systemprompt_tool_points_the_caller_at_the_cli_skill() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let executor = executor(&db.pool, "systemprompt");

    let request = call("not_a_tool", serde_json::json!({}));
    let profile = client();
    let error = systemprompt_mcp_agent::server::tool::dispatch_tool(
        &systemprompt_mcp_agent::server::tool::Dispatch {
            executor: &executor,
            request: &request,
            request_context: &request_context(),
            client: &profile,
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

    db.cleanup().await;
}

// `SystempromptToolHandler::handle` is deliberately not driven here: it shells
// out to the real `systemprompt` binary with the caller's bearer token, so a
// test that reached it would be running the CLI against whatever profile the
// machine has configured. Only the dispatch arm around it is asserted.
