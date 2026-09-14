//! A failed completion write must retain encrypted evidence and replay exactly
//! once.
use crate::fixtures::{insert_session, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;
use std::sync::Arc;
use systemprompt::api::services::gateway::captures::CapturedToolUse;
use systemprompt::api::services::gateway::protocol::canonical::CanonicalRequest;
use systemprompt::api::services::gateway::protocol::canonical_response::{
    CanonicalResponse, CanonicalUsage,
};
use systemprompt::api::services::gateway::protocol::inbound::InboundAdapter;
use systemprompt::api::services::gateway::protocol::inbound::anthropic_messages::AnthropicMessagesInbound;
use systemprompt::api::services::gateway::{
    GatewayAudit, GatewayRepositories, GatewayRequestContext,
};
use systemprompt::database::{Database, DbPool};
use systemprompt::identifiers::{AiRequestId, AiToolCallId, ContextId, SessionId, TraceId, UserId};
use systemprompt_security::policy::types::AccessScope;

async fn profile() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("profile directory");
    let root = astound_test_common::repo_path("");
    let yaml = include_str!("../../../contract/admin/fixtures/profile.yaml")
        .replace("__PROFILE_DIR__", &dir.path().to_string_lossy())
        .replace("__REPO__", &root.to_string_lossy());
    std::fs::create_dir_all(dir.path().join("bin")).expect("bin directory");
    std::fs::write(dir.path().join("profile.yaml"), yaml).expect("fixture profile");
    let secrets = serde_json::json!({"database_url":"postgres://unused:unused@localhost:5432/postgres", "oauth_at_rest_pepper":"accounting-test-pepper-1234567890", "manifest_signing_secret_seed":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=", "encryption_master_key":"11".repeat(32)});
    std::fs::write(dir.path().join("secrets.json"), secrets.to_string()).expect("fixture secrets");
    systemprompt::config::ProfileBootstrap::init_from_path(&dir.path().join("profile.yaml"))
        .expect("profile bootstrap");
    systemprompt::config::SecretsBootstrap::init()
        .await
        .expect("secrets bootstrap");
    dir
}

fn audit(repos: &GatewayRepositories, user: &UserId, request: &CanonicalRequest) -> GatewayAudit {
    let conversation = request
        .derived_gateway_conversation_id()
        .expect("conversation");
    let ctx = GatewayRequestContext {
        ai_request_id: AiRequestId::generate(),
        user_id: user.clone(),
        session_id: Some(SessionId::generate()),
        context_id: ContextId::derived_from_gateway_conversation(user, &conversation),
        gateway_conversation_id: Some(conversation),
        client_session_id: None,
        trace_id: Some(TraceId::generate()),
        access_scope: AccessScope::Unknown,
        client_id: None,
        provider: "anthropic".into(),
        model: "test-model".into(),
        requested_model: None,
        max_tokens: Some(16),
        is_streaming: false,
        wire_protocol: "anthropic-messages".into(),
        access_log: None,
    };
    let audit = GatewayAudit::new(repos, ctx);
    audit
        .pin_pricing(Default::default())
        .expect("explicit fixture pricing");
    audit
}

#[tokio::test]
async fn terminal_receipt_survives_database_failure_and_replays_without_duplicate_tools() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let profile = profile().await;
    let user = insert_user(
        &db.pool,
        &unique("journal-owner"),
        &unclaimed_email("journal"),
    )
    .await;
    let pool: DbPool = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let materializer = Arc::new(systemprompt::agent::services::ContextProviderService::new(
        systemprompt::agent::repository::ContextRepository::new(&pool).expect("context repo"),
    ));
    let journal = systemprompt::api::services::gateway::audit::journal::GatewayJournal::open(
        systemprompt::config::ProfileBootstrap::get_path().expect("profile bootstrapped"),
        systemprompt::config::SecretsBootstrap::get().expect("secrets bootstrapped"),
    )
    .expect("gateway journal opens");
    let repos = GatewayRepositories::new(&pool, journal, materializer).expect("gateway repos");
    let raw = Vec::from(
        br#"{"model":"test-model","max_tokens":16,"messages":[{"role":"user","content":"test"}]}"#
            .as_slice(),
    )
    .into();
    let request = AnthropicMessagesInbound
        .parse_request(&raw)
        .expect("parse request");
    let first = audit(&repos, &user, &request);
    insert_session(
        &db.pool,
        first.ctx.session_id.as_ref().expect("session").as_str(),
        &user,
    )
    .await;
    first.open(&request, &raw).await.expect("admission");
    sqlx::raw_sql("CREATE FUNCTION fail_completion() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.status='completed' THEN RAISE EXCEPTION 'injected storage outage'; END IF; RETURN NEW; END $$; CREATE TRIGGER fail_completion BEFORE UPDATE ON ai_requests FOR EACH ROW EXECUTE FUNCTION fail_completion();")
        .execute(&*db.pool).await.expect("install fault");
    let usage = CanonicalUsage {
        input_tokens: 11,
        output_tokens: 7,
        ..Default::default()
    };
    let response = CanonicalResponse {
        id: "response".into(),
        model: "test-model".into(),
        content: vec![],
        stop_reason: None,
        usage,
        grounding: None,
        code_execution: None,
        raw_finish_reason: None,
        received_surface: Default::default(),
    };
    let body = Vec::from(br#"{"text":"private-recovery-evidence"}"#.as_slice()).into();
    let tools = vec![CapturedToolUse {
        ai_tool_call_id: AiToolCallId::new(unique("tool")),
        tool_name: "Read".into(),
        tool_input: "private-recovery-evidence".into(),
    }];
    assert!(
        first
            .complete(usage, tools, &response, &body)
            .await
            .is_err()
    );
    assert!(
        first
            .fail("late failure must not overwrite completion")
            .await
            .is_err()
    );
    let receipt = std::fs::read_dir(profile.path().join("gateway-journal"))
        .expect("journal")
        .map(|entry| entry.expect("entry").path())
        .find(|path| path.extension().is_some_and(|ext| ext == "receipt"))
        .expect("durable receipt");
    let bytes = std::fs::read(&receipt).expect("encrypted receipt");
    assert!(
        !bytes
            .windows(b"private-recovery-evidence".len())
            .any(|part| part == b"private-recovery-evidence")
    );
    sqlx::query("DROP TRIGGER fail_completion ON ai_requests")
        .execute(&*db.pool)
        .await
        .expect("restore storage");
    let recovered =
        systemprompt::api::services::gateway::audit::journal::recover(&repos.settlement())
            .await
            .expect("owned recovery settles the durable receipt");
    assert_eq!(recovered, 1);
    let second = audit(&repos, &user, &request);
    insert_session(
        &db.pool,
        second.ctx.session_id.as_ref().expect("session").as_str(),
        &user,
    )
    .await;
    second
        .open(&request, &raw)
        .await
        .expect("admission after receipt recovery");
    let row:(String,Option<i32>,i64)=sqlx::query_as("SELECT status,tokens_used,(SELECT count(*) FROM ai_request_tool_calls WHERE request_id=r.id) FROM ai_requests r WHERE id=$1")
        .bind(first.ctx.ai_request_id.as_str()).fetch_one(&*db.pool).await.expect("replayed completion");
    assert_eq!(row, ("completed".into(), Some(18), 1));
    assert!(!receipt.exists());
    second
        .fail("fixture complete")
        .await
        .expect("close fixture admission");
    db.cleanup().await;
}
