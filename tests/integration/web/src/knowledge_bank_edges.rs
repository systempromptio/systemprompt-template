//! Filter, limit, and gate behaviour of the knowledge-bank handlers.
//!
//! `mcp_dispatch` pins the happy path of each tool and the admin refusal; this
//! module drives the branches inside the handlers that the happy path does not
//! reach — the search limit and its clamp, the short-term filter in the store's
//! scorer, the two-way listing filter, the slugifier's trimming, and the admin
//! gate called directly rather than through dispatch.

use std::sync::Arc;

use rmcp::model::CallToolRequestParams;
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId};
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{McpArtifactRepository, McpToolExecutor};
use systemprompt::models::auth::{AuthenticatedUser, Permission};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt_mcp_knowledge_bank::server::tool::{
    Dispatch, dispatch_tool, document_summary, require_admin,
};
use systemprompt_mcp_knowledge_bank::store::KnowledgeStore;
use systemprompt_mcp_knowledge_bank::tools::{TOOL_LIST, TOOL_SEARCH, TOOL_UPLOAD};

use crate::tempdb::TempDb;

const PROJECT: &str = "acme-storefront";

fn executor(pool: &Arc<PgPool>) -> McpToolExecutor {
    let db_pool = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let usage = Arc::new(ToolUsageRepository::new(&db_pool).expect("tool usage repository"));
    let artifacts = Arc::new(McpArtifactRepository::new(&db_pool).expect("artifact repository"));
    McpToolExecutor::new(usage, artifacts, "knowledge-bank")
}

fn request_context() -> SysRequestContext {
    SysRequestContext::new(
        SessionId::new("kb-edge-session"),
        TraceId::new("kb-edge-trace"),
        ContextId::new("00000000-0000-4000-8000-00000000e46e"),
        AgentName::new("kb-edge-agent"),
    )
}

fn admin_context() -> SysRequestContext {
    request_context().with_user(AuthenticatedUser::new(
        uuid::Uuid::new_v4(),
        "kb-admin".to_owned(),
        "kb-admin@example.com".to_owned(),
        vec![Permission::Admin],
    ))
}

fn seeded_store() -> Arc<KnowledgeStore> {
    Arc::new(KnowledgeStore::seeded().expect("the bundled fixtures parse"))
}

fn call(tool: &'static str, arguments: serde_json::Value) -> CallToolRequestParams {
    let object = arguments
        .as_object()
        .expect("tool arguments are a JSON object")
        .clone();
    CallToolRequestParams::new(tool).with_arguments(object)
}

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

async fn dispatch(
    db: &TempDb,
    store: &Arc<KnowledgeStore>,
    ctx: &SysRequestContext,
    tool: &'static str,
    arguments: serde_json::Value,
) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
    let executor = executor(&db.pool);
    let request = call(tool, arguments);
    let profile = client();
    dispatch_tool(
        &Dispatch {
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


#[tokio::test]
async fn a_search_limit_caps_the_documents_returned() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = dispatch(
        &db,
        &seeded_store(),
        &request_context(),
        TOOL_SEARCH,
        serde_json::json!({ "query": "checkout release process performance", "limit": 1 }),
    )
    .await
    .expect("search dispatches to its handler");

    assert_eq!(
        summary_of(&result).split(' ').next(),
        Some("1"),
        "the limit caps the match count the summary reports: {}",
        summary_of(&result)
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_zero_limit_is_clamped_up_to_one_document() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = dispatch(
        &db,
        &seeded_store(),
        &request_context(),
        TOOL_SEARCH,
        serde_json::json!({ "query": "checkout release process performance", "limit": 0 }),
    )
    .await
    .expect("search dispatches to its handler");

    assert_eq!(
        summary_of(&result).split(' ').next(),
        Some("1"),
        "a zero limit returns one document rather than none: {}",
        summary_of(&result)
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_limit_above_the_ceiling_is_clamped_rather_than_refused() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let store = seeded_store();
    let seeded = store.list_documents(None, None).len();

    let result = dispatch(
        &db,
        &store,
        &request_context(),
        TOOL_SEARCH,
        serde_json::json!({ "query": "checkout release process performance", "limit": 9999 }),
    )
    .await
    .expect("an oversized limit is clamped, not rejected");

    let matched: usize = summary_of(&result)
        .split(' ')
        .next()
        .and_then(|n| n.parse().ok())
        .expect("the summary opens with the match count");
    assert!(
        matched <= seeded,
        "the clamp cannot invent documents beyond the seeded set"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_search_scoped_to_a_foreign_project_matches_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = dispatch(
        &db,
        &seeded_store(),
        &request_context(),
        TOOL_SEARCH,
        serde_json::json!({ "query": "checkout", "project_id": "some-other-client" }),
    )
    .await
    .expect("a project-scoped search that matches nothing still succeeds");

    assert!(
        body_of(&result).contains("No matching documents"),
        "the project filter is applied before scoring"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_search_scoped_to_the_seeded_project_still_matches() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = dispatch(
        &db,
        &seeded_store(),
        &request_context(),
        TOOL_SEARCH,
        serde_json::json!({ "query": "checkout", "project_id": PROJECT }),
    )
    .await
    .expect("search dispatches to its handler");

    assert!(
        !body_of(&result).contains("No matching documents"),
        "scoping to the project the fixtures belong to does not filter them out"
    );

    db.cleanup().await;
}

// Terms of three characters or fewer are dropped by the scorer, so a query
// made only of them scores every document zero.
#[tokio::test]
async fn a_query_of_only_short_words_matches_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = dispatch(
        &db,
        &seeded_store(),
        &request_context(),
        TOOL_SEARCH,
        serde_json::json!({ "query": "on of a to" }),
    )
    .await
    .expect("a stopword-only query is still a successful call");

    assert!(
        body_of(&result).contains("No matching documents"),
        "short terms are discarded rather than matching everything"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn listing_by_document_type_returns_only_that_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let store = seeded_store();
    let expected = store.list_documents(None, Some("jira")).len();

    let result = dispatch(
        &db,
        &store,
        &request_context(),
        TOOL_LIST,
        serde_json::json!({ "doc_type": "jira" }),
    )
    .await
    .expect("list dispatches to its handler");

    let body = body_of(&result);
    assert_eq!(body.lines().count(), expected);
    assert!(
        body.lines().all(|line| line.contains("jira")),
        "every listed line is a jira document: {body}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn listing_applies_the_project_and_type_filters_together() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = dispatch(
        &db,
        &seeded_store(),
        &request_context(),
        TOOL_LIST,
        serde_json::json!({ "project_id": PROJECT, "doc_type": "no-such-type" }),
    )
    .await
    .expect("list dispatches to its handler");

    assert!(
        body_of(&result).contains("holds no documents matching the filter"),
        "a matching project with a non-matching type still filters everything out"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn an_uploaded_title_is_slugified_and_its_edges_trimmed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let store = seeded_store();

    let result = dispatch(
        &db,
        &store,
        &admin_context(),
        TOOL_UPLOAD,
        serde_json::json!({
            "doc_type": "confluence",
            "project_id": PROJECT,
            "title": "  Release Notes: v2.1 (draft!)  ",
            "content": "Cut on Friday.",
        }),
    )
    .await
    .expect("an admin may upload");

    assert!(
        body_of(&result).contains("confluence-release-notes--v2-1--draft"),
        "every non-alphanumeric character becomes its own dash — interior runs are \
         not collapsed, only the leading and trailing ones are trimmed: {}",
        body_of(&result)
    );

    db.cleanup().await;
}

#[tokio::test]
async fn an_uploaded_document_is_immediately_searchable() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let store = seeded_store();
    dispatch(
        &db,
        &store,
        &admin_context(),
        TOOL_UPLOAD,
        serde_json::json!({
            "doc_type": "transcript",
            "project_id": PROJECT,
            "title": "Zanzibar Migration Sync",
            "content": "We agreed to defer the zanzibar migration.",
        }),
    )
    .await
    .expect("an admin may upload");

    let found = dispatch(
        &db,
        &store,
        &request_context(),
        TOOL_SEARCH,
        serde_json::json!({ "query": "zanzibar" }),
    )
    .await
    .expect("search dispatches to its handler");

    assert!(
        body_of(&found).contains("Zanzibar Migration Sync"),
        "the upload joined the same in-memory store search reads"
    );

    db.cleanup().await;
}

#[test]
fn the_admin_gate_admits_an_admin_and_refuses_everyone_else() {
    let user = request_context().with_user(AuthenticatedUser::new(
        uuid::Uuid::new_v4(),
        "kb-user".to_owned(),
        "kb-user@example.com".to_owned(),
        vec![Permission::User],
    ));

    assert!(require_admin(&admin_context()).is_ok(), "an admin passes");
    assert!(
        require_admin(&user).is_err(),
        "a signed-in non-admin is refused"
    );
    assert!(
        require_admin(&request_context()).is_err(),
        "an anonymous context is refused"
    );
}

#[test]
fn the_rendered_summary_carries_one_heading_per_document() {
    let store = KnowledgeStore::seeded().expect("the bundled fixtures parse");
    let documents = store.list_documents(None, None);

    let rendered = document_summary(&documents);

    assert_eq!(
        rendered.matches("\n## ").count() + 1,
        documents.len(),
        "each document gets its own markdown heading"
    );
    assert_eq!(
        document_summary(&[]),
        "No matching documents in the knowledge bank.",
        "an empty slice renders the sentinel instead of an empty string"
    );
}
