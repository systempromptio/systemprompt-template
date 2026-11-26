//! `SystempromptToolset` constructed against a real pool.
//!
//! Construction is what needs a database: the toolset takes an
//! `ArtifactIngest` built off the pool before it can serve anything. Its id,
//! instructions and resource surface are asserted on the constructed toolset,
//! and the CLI and reporting tools it lists are pinned against
//! `tools::list_tools`.

use std::sync::Arc;

use rmcp::model::{ReadResourceRequestParams, ResourceContents};
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::extension::mcp::{McpToolset, ResourceList, ResourceRead};
use systemprompt::governance::policy::secrets::SecretScannerCell;
use systemprompt::identifiers::{ArtifactId, McpServerId};
use systemprompt::mcp::artifact_resource_uri;
use systemprompt::mcp::services::ui_renderer::UiRendererRegistry;
use systemprompt_mcp_agent::server::TOOLSET_ID;
use systemprompt_mcp_agent::{CliLocation, SystempromptToolset, tools};

use template_test_common::TempDb;

fn toolset_named(pool: &Arc<PgPool>, server: &str) -> SystempromptToolset {
    let db_pool = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let ingest = Arc::new(systemprompt::mcp::ArtifactIngest::from_db(
        &db_pool,
        SecretScannerCell::default(),
    ));
    SystempromptToolset::new(
        McpServerId::try_new(server).expect("valid fixture identifier"),
        CliLocation::from_profile(&template_test_common::env::fixture_profile()),
        ingest,
        systemprompt::mcp::McpArtifactRepository::new(&db_pool),
        Arc::new(UiRendererRegistry::with_builtins()),
    )
}

fn caller() -> systemprompt::extension::mcp::VerifiedCaller {
    let ctx = systemprompt::models::execution::context::RequestContext::new(
        systemprompt::identifiers::SessionId::new("server-session"),
        systemprompt::identifiers::TraceId::new("server-trace"),
        systemprompt::identifiers::ContextId::try_new("00000000-0000-4000-8000-0000000005e7")
            .expect("valid fixture identifier"),
        systemprompt::identifiers::AgentName::try_new("server-agent")
            .expect("valid fixture agent name"),
        systemprompt::identifiers::Actor::anonymous(systemprompt::identifiers::UserId::generate()),
    );
    template_test_common::hop::verified_caller(&McpServerId::new("systemprompt"), &ctx)
}

fn toolset(pool: &Arc<PgPool>) -> SystempromptToolset {
    toolset_named(pool, "systemprompt")
}

#[tokio::test]
async fn new_builds_a_toolset_that_advertises_its_instructions() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let built = toolset(&db.pool);

    assert_eq!(built.id(), TOOLSET_ID);
    assert!(
        !built.instructions().is_empty(),
        "a constructed toolset advertises its usage instructions"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_toolset_serves_the_artifact_viewer_resource() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let built = toolset(&db.pool);

    assert!(built.serves_resources(), "it serves resources");
    assert!(
        !built
            .list_resources(ResourceList { caller: &caller() })
            .await
            .resources
            .is_empty(),
        "it lists the artifact-viewer resource"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_different_server_id_keeps_the_toolset_id_and_instructions() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let renamed = toolset_named(&db.pool, "sp-staging");

    assert_eq!(renamed.id(), TOOLSET_ID);
    assert_eq!(
        renamed.instructions(),
        toolset(&db.pool).instructions(),
        "the instructions do not depend on the server id"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_toolset_exposes_the_cli_and_read_only_reporting_tools() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let server = McpServerId::try_new("systemprompt").expect("valid fixture identifier");
    let built = toolset(&db.pool);

    let listed = built.tools();

    // The CLI plus one read-only report and six typed analytics tools, all
    // over this platform's own data. The two Atlassian-backed tools that used
    // to sit here are gone: the client reaches Jira and Confluence through the
    // `atlassian` server it already holds, rather than through a proxy here
    // that re-translated an unversioned shape.
    assert_eq!(listed.len(), 8);
    for report in &listed[1..] {
        assert_eq!(
            report.annotations.as_ref().and_then(|a| a.read_only_hint),
            Some(true),
            "{} must advertise readOnlyHint",
            report.name
        );
    }
    assert_eq!(listed[0].name.as_ref(), "systemprompt");
    assert_eq!(listed[0].title.as_deref(), Some("SystemPrompt CLI"));
    assert_eq!(listed[1].name.as_ref(), "admin_report");
    let typed: Vec<&str> = listed[2..].iter().map(|t| t.name.as_ref()).collect();
    let expected: Vec<String> = tools::list_tools(&server)
        .iter()
        .map(|t| t.name.to_string())
        .collect();
    let names: Vec<String> = listed.iter().map(|t| t.name.to_string()).collect();
    assert_eq!(
        names, expected,
        "the toolset lists exactly tools::list_tools"
    );
    assert_eq!(
        typed,
        [
            "user_activity",
            "conversation_list",
            "usage_by_user",
            "request_log",
            "conversation_audit",
            "users"
        ]
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_cli_tool_requires_a_command_string() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let _built = toolset(&db.pool);

    let schema = tools::input_schema();

    assert_eq!(schema.get("type").and_then(|t| t.as_str()), Some("object"));
    assert!(
        schema
            .get("properties")
            .and_then(|p| p.get("command"))
            .is_some(),
        "the tool takes a `command` property: {schema}"
    );
    assert!(
        schema
            .get("required")
            .and_then(|r| r.as_array())
            .is_some_and(|r| r.iter().any(|v| v.as_str() == Some("command"))),
        "`command` is required: {schema}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_viewer_uri_is_served_and_unknown_artifact_uris_are_refused() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let built = toolset(&db.pool);

    let caller = caller();
    let listed = built.list_resources(ResourceList { caller: &caller }).await;
    let viewer = listed
        .resources
        .first()
        .expect("artifact viewer is advertised");
    assert_eq!(viewer.uri, "ui://systemprompt/artifact-viewer");

    let read = built
        .read_resource(ResourceRead {
            request: &ReadResourceRequestParams::new(viewer.uri.clone()),
            caller: &caller,
        })
        .await
        .expect("the toolset owns its viewer URI")
        .expect("read artifact viewer resource");
    let ResourceContents::TextResourceContents {
        uri,
        mime_type,
        text,
        ..
    } = read.contents.first().expect("viewer has one HTML resource")
    else {
        panic!("viewer must return text HTML")
    };
    assert_eq!(uri, "ui://systemprompt/artifact-viewer");
    assert_eq!(mime_type.as_deref(), Some("text/html;profile=mcp-app"));
    assert!(
        !text.is_empty(),
        "the viewer serves the admin artifact shell"
    );

    let artifact_uri =
        artifact_resource_uri(&McpServerId::new("systemprompt"), &ArtifactId::generate());
    assert!(
        built
            .read_resource(ResourceRead {
                request: &ReadResourceRequestParams::new(artifact_uri),
                caller: &caller,
            })
            .await
            .is_some_and(|read| read.is_err()),
        "a stored-artifact URI the caller does not own is refused by the toolset"
    );
    assert!(
        built
            .read_resource(ResourceRead {
                request: &ReadResourceRequestParams::new("ui://other/artifact-viewer"),
                caller: &caller,
            })
            .await
            .is_some_and(|read| read.is_err()),
        "an unknown resource URI is refused"
    );

    db.cleanup().await;
}

// `call` needs a `ToolExecution` and a hop-verified caller; its dispatch is
// driven directly in `mcp_dispatch` and `mcp_cli`.
