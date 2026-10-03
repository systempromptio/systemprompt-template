//! Real MCP protocol coverage for the bundled systemprompt server.
//!
//! A duplex transport gives rmcp a real peer, so these tests exercise the
//! `ServerHandler` methods that cannot be called with a hand-built request
//! context: handshake, resource discovery/read, and the fail-closed auth gate.

use std::sync::{Arc, Once};
use std::time::Duration;

use axum::body::Body;
use axum::http;
use http_body_util::BodyExt;
use rmcp::ServiceExt;
use rmcp::model::{
    ClientCapabilities, ClientInfo, Implementation, ReadResourceRequestParams, ResourceContents,
};
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::identifiers::{
    Actor, AgentName, ArtifactId, ContextId, McpServerId, SessionId, TraceId, UserId,
};
use systemprompt::mcp::{ArtifactIngest, artifact_resource_uri};
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt::models::execution::context::RequestContext;
use systemprompt::models::mcp::ExecutionSource;
use systemprompt::security::authz::{DenyAllHook, SharedAuthzHook};
use systemprompt::traits::InjectContextHeaders;
use systemprompt_mcp_agent::SystempromptServer;
use tower::ServiceExt as _;

use crate::tempdb::TempDb;

const MCP_PROTOCOL_VERSION: &str = "2025-06-18";
const MAX_MCP_RESPONSE_BYTES: usize = 64 * 1024;

fn db_pool(pool: &Arc<PgPool>) -> systemprompt::database::DbPool {
    Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ))
}

fn request_context() -> RequestContext {
    RequestContext::new(
        SessionId::new("protocol-session"),
        TraceId::new("protocol-trace"),
        ContextId::try_new("00000000-0000-4000-8000-00000000a11c").expect("valid fixture context"),
        AgentName::try_new("protocol-agent").expect("valid fixture agent"),
        systemprompt::identifiers::Actor::anonymous(systemprompt::identifiers::UserId::generate()),
    )
}

fn http_request_context() -> RequestContext {
    request_context().with_actor(Actor::user(UserId::new(
        "00000000-0000-4000-8000-00000000a11d",
    )))
}

fn server(pool: &Arc<PgPool>) -> (SystempromptServer, Arc<ArtifactIngest>) {
    let db_pool = db_pool(pool);
    let ingest = Arc::new(ArtifactIngest::new(
        systemprompt::mcp::repository::ArtifactIngestRepositories::new(&db_pool),
        None,
    ));
    let server = SystempromptServer::new(
        db_pool,
        McpServerId::try_new("systemprompt").expect("valid service id"),
        deny_all_hook(),
        Arc::clone(&ingest),
    );
    (server, ingest)
}

fn deny_all_hook() -> SharedAuthzHook {
    Arc::new(DenyAllHook::null())
}

fn client_info() -> ClientInfo {
    ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new("protocol-contract-client", "1.0"),
    )
}

fn install_services_profile() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let root = template_test_common::repo_root();
        let dir = root.join(format!(
            "tests/target/mcp-protocol-profile-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join("bin")).expect("create MCP protocol fixture directory");
        let profile = include_str!("../../../contract/admin/fixtures/profile.yaml")
            .replace("__PROFILE_DIR__", &dir.to_string_lossy())
            .replace("__REPO__", &root.to_string_lossy())
            .replace(
                &format!("services: {}/services", dir.display()),
                &format!("services: {}/services", root.display()),
            );
        let path = dir.join("profile.yaml");
        std::fs::write(&path, profile).expect("write MCP protocol fixture profile");
        systemprompt::config::ProfileBootstrap::init_from_path(&path)
            .expect("install profile that registers the systemprompt MCP service");
    });
}

fn mcp_request(body: serde_json::Value, session: Option<&str>) -> http::Request<Body> {
    let mut request = http::Request::builder()
        .method(http::Method::POST)
        .uri("/mcp")
        .header(http::header::HOST, "127.0.0.1")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::ACCEPT, "application/json, text/event-stream")
        .body(Body::from(body.to_string()))
        .expect("build MCP request");
    http_request_context().inject_headers(request.headers_mut());
    if let Some(session) = session {
        request.headers_mut().insert(
            "mcp-session-id",
            http::HeaderValue::from_str(session).expect("session header"),
        );
        request.headers_mut().insert(
            "mcp-protocol-version",
            http::HeaderValue::from_static(MCP_PROTOCOL_VERSION),
        );
    }
    request
}

async fn first_sse_json(mut body: Body) -> serde_json::Value {
    tokio::time::timeout(Duration::from_secs(5), async move {
        let mut event = String::new();
        let mut received = 0;
        while let Some(frame) = body.frame().await {
            let frame = frame.expect("read MCP response frame");
            let Ok(data) = frame.into_data() else {
                continue;
            };
            received += data.len();
            assert!(
                received <= MAX_MCP_RESPONSE_BYTES,
                "MCP response exceeded {MAX_MCP_RESPONSE_BYTES} bytes before its first data event"
            );
            event.push_str(std::str::from_utf8(&data).expect("MCP response is UTF-8 SSE"));
            while let Some(end) = event.find("\n\n") {
                let complete: String = event.drain(..end + 2).collect();
                let data = complete
                    .lines()
                    .filter_map(|line| line.strip_prefix("data:"))
                    .map(str::trim_start)
                    .collect::<String>();
                if !data.is_empty() {
                    return serde_json::from_str(&data).unwrap_or_else(|error| {
                        panic!("invalid MCP SSE data: {error}; data={data}")
                    });
                }
            }
        }
        panic!("MCP response ended before a JSON-RPC SSE data event")
    })
    .await
    .expect("MCP response produced its first data event before timeout")
}

async fn post_mcp(
    router: axum::Router,
    body: serde_json::Value,
    session: Option<&str>,
) -> (http::StatusCode, http::HeaderMap, serde_json::Value) {
    let request = mcp_request(body, session);
    let response = router.oneshot(request).await.expect("MCP router responds");
    let status = response.status();
    let headers = response.headers().clone();
    assert_eq!(
        headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream"),
        "MCP response must use the legacy SSE transport"
    );
    let response = first_sse_json(response.into_body()).await;
    (status, headers, response)
}

async fn notify_initialized(router: axum::Router, session: &str) {
    let response = router
        .oneshot(mcp_request(
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }),
            Some(session),
        ))
        .await
        .expect("MCP router accepts initialized notification");
    assert!(
        response.status().is_success(),
        "initialized notification was refused: {}",
        response.status()
    );
}

async fn seed_artifact(ingest: &ArtifactIngest) -> systemprompt::identifiers::ArtifactId {
    let artifact = CliArtifact::text(
        TextArtifact::new("protocol artifact body").with_title("Protocol artifact"),
    );
    let artifact_type = artifact.artifact_type_str();
    let mut body = serde_json::to_value(&artifact).expect("artifact serialises");
    body.as_object_mut().expect("artifact is an object").insert(
        "x-artifact-type".to_owned(),
        serde_json::json!(artifact_type),
    );
    let mut result = rmcp::model::CallToolResult::success(Vec::new());
    result.structured_content = Some(body);
    ingest
        .ingest(systemprompt::mcp::IngestRequest {
            result,
            tool_name: systemprompt::identifiers::McpToolName::new("systemprompt"),
            server_name: Some(systemprompt::identifiers::McpServerId::new("systemprompt")),
            ai_tool_call_id: None,
            mcp_execution_id: None,
            ctx: request_context(),
            skill: None,
            source: ExecutionSource::InProcess,
            started_at: None,
            input: Some(serde_json::json!({"command": "core skills list"})),
        })
        .await
        .expect("persist test artifact")
        .artifact_id
}

#[tokio::test]
async fn protocol_client_initializes_discovers_resources_and_reads_stored_artifacts() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (server, ingest) = server(&db.pool);
    let artifact_id = seed_artifact(&ingest).await;
    let (server_transport, client_transport) = tokio::io::duplex(16 * 1024);
    let server_task = tokio::spawn(async move {
        server
            .serve(server_transport)
            .await
            .expect("server starts")
            .waiting()
            .await
    });
    let client = tokio::time::timeout(
        Duration::from_secs(5),
        client_info().serve(client_transport),
    )
    .await
    .expect("MCP initialization completes before timeout")
    .expect("client completes MCP initialization");

    let tools = client.list_tools(None).await.expect("list tools");
    assert!(tools.tools.iter().any(|tool| tool.name == "systemprompt"));
    assert!(tools.tools.iter().any(|tool| tool.name == "request_log"));
    let templates = client
        .list_resource_templates(None)
        .await
        .expect("list resource templates");
    assert!(templates.resource_templates.is_empty());
    let resources = client.list_resources(None).await.expect("list resources");
    let viewer = resources
        .resources
        .first()
        .expect("artifact viewer is advertised");
    assert_eq!(viewer.uri, "ui://systemprompt/artifact-viewer");

    let viewer = client
        .read_resource(ReadResourceRequestParams::new(viewer.uri.clone()))
        .await
        .expect("read artifact viewer resource");
    let ResourceContents::TextResourceContents {
        uri,
        mime_type,
        text,
        ..
    } = viewer
        .contents
        .first()
        .expect("viewer has one HTML resource")
    else {
        panic!("viewer must return text HTML")
    };
    assert_eq!(uri, "ui://systemprompt/artifact-viewer");
    assert_eq!(mime_type.as_deref(), Some("text/html;profile=mcp-app"));
    assert!(text.contains("Artifact Viewer"));
    let artifact_uri = artifact_resource_uri(
        &systemprompt::identifiers::McpServerId::new("systemprompt"),
        &artifact_id,
    );
    let artifact = client
        .read_resource(ReadResourceRequestParams::new(artifact_uri))
        .await
        .expect("read stored artifact resource");
    let ResourceContents::TextResourceContents {
        mime_type, text, ..
    } = artifact
        .contents
        .first()
        .expect("artifact has one HTML resource")
    else {
        panic!("artifact must return text HTML")
    };
    assert_eq!(mime_type.as_deref(), Some("text/html;profile=mcp-app"));
    assert!(text.contains("protocol artifact body"));
    let missing_artifact_uri = artifact_resource_uri(
        &systemprompt::identifiers::McpServerId::new("systemprompt"),
        &ArtifactId::generate(),
    );
    assert!(
        client
            .read_resource(ReadResourceRequestParams::new(missing_artifact_uri))
            .await
            .is_err(),
        "a syntactically valid unknown artifact URI is refused"
    );
    assert!(
        client
            .read_resource(ReadResourceRequestParams::new("ui://other/artifact-viewer"))
            .await
            .is_err(),
        "an unknown resource URI is refused"
    );

    client.cancel().await.expect("cancel client");
    server_task
        .await
        .expect("server task joins")
        .expect("server cancels cleanly");
    db.cleanup().await;
}

#[tokio::test]
async fn unauthenticated_http_tool_call_reaches_oauth_and_creates_no_execution() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    install_services_profile();
    let (server, _) = server(&db.pool);
    let db_pool = db_pool(&db.pool);
    let router = systemprompt::mcp::create_router(
        server,
        Arc::new(systemprompt::mcp::repository::McpSessionRepository::new(
            &db_pool,
        )),
        systemprompt::mcp::McpHttpConfig::default(),
    );
    let (status, headers, initialized) = post_mcp(
        router.clone(),
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "protocol-contract-client", "version": "1.0"}
            }
        }),
        None,
    )
    .await;
    assert!(
        status.is_success(),
        "MCP initialization failed: {initialized}"
    );
    let session = headers
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .expect("initialization creates a session")
        .to_owned();
    notify_initialized(router.clone(), &session).await;
    let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_tool_executions")
        .fetch_one(&*db.pool)
        .await
        .expect("count executions before denial");
    let (status, _, denied) = post_mcp(
        router,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {"name": "users", "arguments": {}}
        }),
        Some(&session),
    )
    .await;
    assert!(
        status.is_success(),
        "MCP tool denial transport failed: {denied}"
    );
    let error = denied["error"]
        .as_object()
        .unwrap_or_else(|| panic!("OAuth refusal is not an MCP error: {denied}"));
    assert_eq!(
        error["code"],
        serde_json::json!(-32600),
        "unexpected missing-Bearer MCP response: {denied}"
    );
    let message = error["message"].as_str().expect("OAuth error message");
    assert!(
        message.contains("requires OAuth"),
        "missing-Bearer request reached the wrong MCP error: {denied}"
    );
    assert!(
        message.contains("no Bearer token"),
        "missing-Bearer request reached the wrong MCP error: {denied}"
    );
    let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_tool_executions")
        .fetch_one(&*db.pool)
        .await
        .expect("count executions after denial");
    assert_eq!(after, before, "denial must precede execution creation");

    db.cleanup().await;
}
