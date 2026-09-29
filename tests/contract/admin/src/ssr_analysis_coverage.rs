//! Populated SSR coverage for the Analysis conversations and skills pages.
//!
//! These pages read the deterministic `conversation_facts` rollup and the
//! hook-derived skill view.  Seeding those two records directly keeps this
//! contract focused on rendering, filters, and the console gate.

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

async fn seed_analysis(pool: &PgPool) -> (String, String, String) {
    let user = seed::insert_user(
        pool,
        &seed::unique("analysis-user"),
        "analysis-user@contract.test",
    )
    .await;
    let context = uuid::Uuid::new_v4().to_string();
    let session = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();
    seed::insert_session(pool, &session, &user).await;
    let request_id = seed::unique("analysis-request");
    seed::insert_request(
        pool,
        &seed::RequestSpec {
            id: request_id.clone(),
            user_id: &user,
            session_id: Some(&session),
            trace_id: None,
            context_id: Some(&context),
            status: "completed",
        },
    )
    .await;
    sqlx::query("UPDATE ai_requests SET client_session_id = $1 WHERE id = $2")
        .bind(&session)
        .bind(&request_id)
        .execute(pool)
        .await
        .expect("bind client session");
    sqlx::query(
        "INSERT INTO conversation_facts
         (context_id, user_id, client_session_id, client_kind, client_attestation,
          wire_protocol, model, provider, models, providers, request_count, turn_count,
          input_tokens, output_tokens, cost_microdollars, p95_latency_ms, skill_invocations,
          skills, first_at, last_at)
         VALUES ($1, $2, $3, 'contract-client', 'verified', 'openai', 'contract-model',
                 'anthropic', ARRAY['contract-model'], ARRAY['anthropic'], 2, 2,
                 200, 40, 5000, 250, 1, ARRAY['demo:skill'], $4, $4)",
    )
    .bind(&context)
    .bind(user.as_str())
    .bind(&session)
    .bind(now - Duration::minutes(1))
    .execute(pool)
    .await
    .expect("insert analysis fact");
    sqlx::query(
        "INSERT INTO conversation_analyses
         (context_id, user_id, status, title, category, outcome, completion, summary)
         VALUES ($1, $2, 'classified', 'Contract analysis', 'development', 'achieved', 92,
                 'The contract conversation completed.')",
    )
    .bind(&context)
    .bind(user.as_str())
    .execute(pool)
    .await
    .expect("insert analysis judgment");
    sqlx::query(
        "INSERT INTO plugin_usage_events
         (id, user_id, session_id, event_type, plugin_id, prompt_preview, created_at)
         VALUES ($1, $2, $3, 'UserPromptSubmit', 'demo', '/demo:skill', $4)",
    )
    .bind(seed::unique("skill-event"))
    .bind(user.as_str())
    .bind(&session)
    .bind(now - Duration::minutes(1))
    .execute(pool)
    .await
    .expect("insert skill invocation");
    // Why: the adoption table lists the marketplaces a services sync recorded
    // in `service_owned_ids`, and a contract database never runs a sync.
    sqlx::query(
        "INSERT INTO service_sources (name, kind, provenance)
         VALUES ('contract-base', 'base', 'contract fixture')
         ON CONFLICT (name) DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("record the fixture services source");
    sqlx::query(
        "INSERT INTO service_owned_ids (kind, id, source, marketplace_id)
         VALUES ('marketplace', 'demo-marketplace', 'contract-base', NULL),
                ('plugin', 'demo', 'contract-base', 'demo-marketplace')
         ON CONFLICT (kind, id) DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("declare the fixture marketplace");
    (user.as_str().to_owned(), context, request_id)
}

#[tokio::test(flavor = "multi_thread")]
async fn populated_analysis_pages_render_facts_and_skill_totals() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (_user, context, request_id) = seed_analysis(&db.pool).await;

    let (status, body) = app
        .call(Call::get("/admin/analysis/conversations", Principal::Admin))
        .await;
    assert_eq!(status, StatusCode::OK, "conversation body: {body}");
    assert!(body.contains(&context), "conversation context is rendered");
    assert!(
        body.contains("Contract analysis"),
        "judge title is rendered"
    );

    let (status, body) = app
        .call(Call::get("/admin/analysis/skills", Principal::Admin))
        .await;
    assert_eq!(status, StatusCode::OK, "skills overview body: {body}");
    assert!(
        body.contains("Adoption by marketplace") && body.contains("sp-adopt-table"),
        "the default tab is the marketplace overview: {body}"
    );
    assert!(
        !body.contains("sp-askill-table"),
        "the overview does not render the skills table"
    );

    let (status, body) = app
        .call(Call::get(
            "/admin/analysis/skills?tab=activity",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "skills activity body: {body}");
    assert!(
        body.contains("data-chart") && body.contains("Skills in the window"),
        "the activity tab renders the chart and the top-skills list"
    );

    let (status, body) = app
        .call(Call::get(
            "/admin/analysis/skills?tab=skills&search=demo",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "skills body: {body}");
    assert!(body.contains("demo:skill"), "skill row is rendered");
    assert!(
        body.contains("$0.005000"),
        "skill cost preserves sub-cent precision"
    );

    let (status, body) = app
        .call(Call::get(
            "/admin/analysis/skills?tab=skills&skill=demo",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "skills body via skill=: {body}");
    assert!(
        body.contains("demo:skill"),
        "a report's skill= link searches the table"
    );

    let (status, _) = app
        .call(Call::get(
            "/admin/analysis/skills?tab=bogus",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "an unknown tab is refused");

    let (status, body) = app
        .call(Call::get(
            &format!("/admin/analysis/conversations/{context}"),
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "conversation detail body: {body}");
    assert!(
        body.contains("Contract analysis"),
        "detail includes judge title"
    );
    assert!(
        body.contains("claude-contract-model"),
        "detail renders the request ledger model"
    );
    assert!(
        body.contains(&format!("/admin/requests/{request_id}")),
        "detail links its request ledger row"
    );
    assert!(
        body.contains("The contract conversation completed."),
        "detail renders the judge summary"
    );
    assert!(body.contains("demo:skill"), "detail includes hooked skill");

    let (status, body) = app
        .call(Call::get(
            "/admin/analysis/skills/demo:skill",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "skill detail body: {body}");
    assert!(
        body.contains("demo:skill"),
        "skill detail includes its identity"
    );
    assert!(
        body.contains("Contract analysis"),
        "skill detail links the conversation"
    );

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn analysis_pages_require_console_access_and_unknown_conversation_is_404() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (_, context, _) = seed_analysis(&db.pool).await;

    let (non_admin, _) = app
        .call(Call::get(
            "/admin/analysis/conversations",
            Principal::NonAdmin,
        ))
        .await;
    assert_eq!(non_admin, StatusCode::SEE_OTHER);
    let (anonymous, _) = app
        .call(Call::get("/admin/analysis/skills", Principal::Anonymous))
        .await;
    assert_eq!(anonymous, StatusCode::TEMPORARY_REDIRECT);

    let unknown = uuid::Uuid::new_v4();
    let (status, body) = app
        .call(Call::get(
            &format!("/admin/analysis/conversations/{unknown}"),
            Principal::Admin,
        ))
        .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "unknown context body: {body}"
    );
    assert!(
        !body.contains(&context),
        "unknown page does not leak another context"
    );

    db.cleanup().await;
}
