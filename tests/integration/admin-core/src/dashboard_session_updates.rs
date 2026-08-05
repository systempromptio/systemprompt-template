//! `repositories::dashboard::usage_aggregations::session_updates` and
//! `::ai_summaries` — the in-place edits a session row receives as its events
//! and its AI analysis arrive.
//!
//! `update_session_metadata` treats the empty string as "no opinion" and keeps
//! whatever is already stored, which is what lets a later event with partial
//! data avoid clobbering an earlier complete one. That is the whole contract of
//! the function, so it is pinned in both directions.

use systemprompt::identifiers::SessionId;
use systemprompt_web_admin::repositories::dashboard::usage_aggregations::{
    update_session_ai_summary_structured, update_session_metadata, update_session_permission_mode,
    update_session_title_if_empty,
};
use systemprompt_web_admin::test_support::SessionAnalysis;

use crate::fixtures::{SummarySpec, insert_summary, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

fn analysis() -> SessionAnalysis {
    SessionAnalysis {
        title: "Fixed the parser".to_owned(),
        description: "A short description".to_owned(),
        goal_summary: "Make the parser accept trailing commas".to_owned(),
        outcomes: vec!["Parser updated".to_owned(), "Tests added".to_owned()],
        tags: vec!["rust".to_owned(), "parser".to_owned()],
        goal_achieved: "yes".to_owned(),
        quality_score: 82,
        outcome: "success".to_owned(),
        error_analysis: None,
        skill_assessment: None,
        recommendations: None,
        skill_scores: None,
        category: None,
        goal_outcome_map: None,
        efficiency_metrics: None,
        best_practices_checklist: None,
        improvement_hints: None,
        automation_ratio: None,
        plan_mode_used: None,
        client_surface: None,
    }
}

struct MetadataRow {
    client_source: Option<String>,
    model: Option<String>,
    permission_mode: Option<String>,
    ai_title: Option<String>,
}

async fn read_metadata(pool: &sqlx::PgPool, session_id: &str) -> MetadataRow {
    let row: (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT client_source, model, permission_mode, ai_title
         FROM plugin_session_summaries WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await
    .expect("read session metadata");
    MetadataRow {
        client_source: row.0,
        model: row.1,
        permission_mode: row.2,
        ai_title: row.3,
    }
}

async fn seeded_session(pool: &sqlx::PgPool, label: &str) -> String {
    let user = insert_user(pool, &unique("user"), &unclaimed_email(label)).await;
    let session = unique("session");
    insert_summary(pool, &SummarySpec::open(&session, &user)).await;
    session
}

#[tokio::test]
async fn update_session_metadata_writes_every_non_empty_field() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = seeded_session(&db.pool, "meta").await;

    update_session_metadata(
        &db.pool,
        &SessionId::new(session.clone()),
        "claude-code",
        "claude-opus-4-5",
        "acceptEdits",
    )
    .await;

    let row = read_metadata(&db.pool, &session).await;
    assert_eq!(row.client_source.as_deref(), Some("claude-code"));
    assert_eq!(row.model.as_deref(), Some("claude-opus-4-5"));
    assert_eq!(row.permission_mode.as_deref(), Some("acceptEdits"));
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_metadata_keeps_the_stored_value_when_a_field_is_blank() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = seeded_session(&db.pool, "metablank").await;
    let id = SessionId::new(session.clone());
    update_session_metadata(&db.pool, &id, "claude-code", "claude-opus-4-5", "plan").await;

    update_session_metadata(&db.pool, &id, "", "", "acceptEdits").await;

    let row = read_metadata(&db.pool, &session).await;
    assert_eq!(row.client_source.as_deref(), Some("claude-code"));
    assert_eq!(row.model.as_deref(), Some("claude-opus-4-5"));
    assert_eq!(row.permission_mode.as_deref(), Some("acceptEdits"));
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_metadata_is_a_no_op_for_an_unknown_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    update_session_metadata(
        &db.pool,
        &SessionId::new(unique("session")),
        "claude-code",
        "m",
        "plan",
    )
    .await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plugin_session_summaries")
        .fetch_one(&*db.pool)
        .await
        .expect("count summaries");
    assert_eq!(count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_permission_mode_overwrites_unconditionally() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = seeded_session(&db.pool, "permmode").await;
    let id = SessionId::new(session.clone());
    update_session_permission_mode(&db.pool, &id, "plan").await;

    update_session_permission_mode(&db.pool, &id, "bypassPermissions").await;

    let row = read_metadata(&db.pool, &session).await;
    assert_eq!(row.permission_mode.as_deref(), Some("bypassPermissions"));
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_title_if_empty_sets_a_title_on_a_fresh_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = seeded_session(&db.pool, "title").await;

    update_session_title_if_empty(&db.pool, &SessionId::new(session.clone()), "First title").await;

    assert_eq!(
        read_metadata(&db.pool, &session).await.ai_title.as_deref(),
        Some("First title")
    );
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_title_if_empty_refuses_to_replace_an_existing_title() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = seeded_session(&db.pool, "titlekeep").await;
    let id = SessionId::new(session.clone());
    update_session_title_if_empty(&db.pool, &id, "First title").await;

    update_session_title_if_empty(&db.pool, &id, "Second title").await;

    assert_eq!(
        read_metadata(&db.pool, &session).await.ai_title.as_deref(),
        Some("First title")
    );
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_title_if_empty_replaces_a_blank_string_title() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("titleblank")).await;
    let session = unique("session");
    let mut spec = SummarySpec::open(&session, &user);
    spec.ai_title = Some("");
    insert_summary(&db.pool, &spec).await;

    update_session_title_if_empty(&db.pool, &SessionId::new(session.clone()), "Real title").await;

    assert_eq!(
        read_metadata(&db.pool, &session).await.ai_title.as_deref(),
        Some("Real title")
    );
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_ai_summary_structured_writes_the_composed_summary_and_tags() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = seeded_session(&db.pool, "aisummary").await;

    update_session_ai_summary_structured(&db.pool, &SessionId::new(session.clone()), &analysis())
        .await;

    let row: (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT ai_title, ai_description, ai_summary, ai_tags
         FROM plugin_session_summaries WHERE session_id = $1",
    )
    .bind(&session)
    .fetch_one(&*db.pool)
    .await
    .expect("read ai summary columns");

    assert_eq!(row.0.as_deref(), Some("Fixed the parser"));
    assert_eq!(row.1.as_deref(), Some("A short description"));
    assert_eq!(row.3.as_deref(), Some("rust,parser"));
    let summary = row.2.expect("summary written");
    assert!(summary.starts_with("Make the parser accept trailing commas"));
    assert!(summary.contains("- Parser updated"));
    assert!(summary.contains("- Tests added"));
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_ai_summary_structured_overwrites_an_existing_title() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = seeded_session(&db.pool, "aioverwrite").await;
    let id = SessionId::new(session.clone());
    update_session_title_if_empty(&db.pool, &id, "Placeholder").await;

    update_session_ai_summary_structured(&db.pool, &id, &analysis()).await;

    assert_eq!(
        read_metadata(&db.pool, &session).await.ai_title.as_deref(),
        Some("Fixed the parser")
    );
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_ai_summary_structured_is_a_no_op_for_an_unknown_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    update_session_ai_summary_structured(&db.pool, &SessionId::new(unique("session")), &analysis())
        .await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plugin_session_summaries")
        .fetch_one(&*db.pool)
        .await
        .expect("count summaries");
    assert_eq!(count, 0);
    db.cleanup().await;
}
