//! `repositories::dashboard::session_analyses` — the upsert that persists the
//! AI analysis of a finished session.
//!
//! `insert_session_analysis` swallows its errors, so every assertion reads the
//! row back. The interesting part is `prepare_upsert_params`: several columns
//! are not fields on `SessionAnalysis` but derived from it — `summary` is
//! composed, `tags` is joined, `category` defaults, and the three
//! `efficiency_metrics` counters are lifted out into their own columns.

use std::collections::HashMap;

use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_web_admin::repositories::dashboard::session_analyses::insert_session_analysis;
use systemprompt_web_admin::test_support::SessionAnalysis;
use systemprompt_web_admin::types::session_analysis::{
    BestPracticeItem, EfficiencyMetrics, GoalOutcomeMapping,
};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

fn minimal() -> SessionAnalysis {
    SessionAnalysis {
        title: "Title".to_owned(),
        description: "Description".to_owned(),
        goal_summary: "Goal".to_owned(),
        outcomes: Vec::new(),
        tags: Vec::new(),
        goal_achieved: "partial".to_owned(),
        quality_score: 55,
        outcome: "mixed".to_owned(),
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

struct StoredAnalysis {
    title: String,
    summary: String,
    tags: String,
    quality_score: i16,
    category: String,
    corrections_count: i32,
    session_duration_minutes: Option<i32>,
    total_turns: Option<i32>,
    plan_mode_used: bool,
    client_surface: String,
}

// The `session_analyses` columns in SELECT order, read positionally so the
// test sees exactly what was stored rather than what a struct decoder infers.
type AnalysisColumns = (
    String,
    String,
    String,
    i16,
    String,
    i32,
    Option<i32>,
    Option<i32>,
    bool,
    String,
);

async fn read_analysis(pool: &sqlx::PgPool, session_id: &str) -> StoredAnalysis {
    let row: AnalysisColumns = sqlx::query_as(
        "SELECT title, summary, tags, quality_score, category, corrections_count,
                    session_duration_minutes, total_turns, plan_mode_used, client_surface
             FROM session_analyses WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await
    .expect("read session analysis");
    StoredAnalysis {
        title: row.0,
        summary: row.1,
        tags: row.2,
        quality_score: row.3,
        category: row.4,
        corrections_count: row.5,
        session_duration_minutes: row.6,
        total_turns: row.7,
        plan_mode_used: row.8,
        client_surface: row.9,
    }
}

async fn analysis_user(pool: &sqlx::PgPool, label: &str) -> UserId {
    insert_user(pool, &unique("user"), &unclaimed_email(label)).await
}

#[tokio::test]
async fn insert_session_analysis_writes_a_minimal_analysis() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "analysis").await;
    let session = unique("session");

    insert_session_analysis(
        &db.pool,
        &SessionId::new(session.clone()),
        &user,
        &minimal(),
    )
    .await;

    let stored = read_analysis(&db.pool, &session).await;
    assert_eq!(stored.title, "Title");
    assert_eq!(stored.quality_score, 55);
    assert_eq!(stored.summary, "Goal", "no outcomes means the goal alone");
    assert_eq!(stored.tags, "");
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_analysis_defaults_a_missing_category_to_other() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "category").await;
    let session = unique("session");

    insert_session_analysis(
        &db.pool,
        &SessionId::new(session.clone()),
        &user,
        &minimal(),
    )
    .await;

    assert_eq!(read_analysis(&db.pool, &session).await.category, "other");
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_analysis_keeps_a_supplied_category() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "categoryset").await;
    let session = unique("session");
    let mut analysis = minimal();
    analysis.category = Some("debugging".to_owned());

    insert_session_analysis(&db.pool, &SessionId::new(session.clone()), &user, &analysis).await;

    assert_eq!(
        read_analysis(&db.pool, &session).await.category,
        "debugging"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_analysis_joins_the_tags_with_commas() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "tags").await;
    let session = unique("session");
    let mut analysis = minimal();
    analysis.tags = vec!["sql".to_owned(), "perf".to_owned(), "review".to_owned()];

    insert_session_analysis(&db.pool, &SessionId::new(session.clone()), &user, &analysis).await;

    assert_eq!(
        read_analysis(&db.pool, &session).await.tags,
        "sql,perf,review"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_analysis_composes_the_summary_from_the_goal_and_outcomes() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "composed").await;
    let session = unique("session");
    let mut analysis = minimal();
    analysis.outcomes = vec!["Shipped".to_owned(), "Documented".to_owned()];

    insert_session_analysis(&db.pool, &SessionId::new(session.clone()), &user, &analysis).await;

    let summary = read_analysis(&db.pool, &session).await.summary;
    assert_eq!(summary, "Goal\n\n- Shipped\n- Documented");
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_analysis_lifts_the_efficiency_counters_into_their_own_columns() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "efficiency").await;
    let session = unique("session");
    let mut analysis = minimal();
    analysis.efficiency_metrics = Some(EfficiencyMetrics {
        total_turns: 31,
        duration_minutes: 47,
        corrections_count: 5,
        avg_turns_per_goal: 3.5,
        unnecessary_loops: 2,
    });

    insert_session_analysis(&db.pool, &SessionId::new(session.clone()), &user, &analysis).await;

    let stored = read_analysis(&db.pool, &session).await;
    assert_eq!(stored.corrections_count, 5);
    assert_eq!(stored.session_duration_minutes, Some(47));
    assert_eq!(stored.total_turns, Some(31));
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_analysis_defaults_the_counters_when_no_metrics_are_supplied() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "noefficiency").await;
    let session = unique("session");

    insert_session_analysis(
        &db.pool,
        &SessionId::new(session.clone()),
        &user,
        &minimal(),
    )
    .await;

    let stored = read_analysis(&db.pool, &session).await;
    assert_eq!(stored.corrections_count, 0);
    assert!(stored.session_duration_minutes.is_none());
    assert!(stored.total_turns.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_analysis_stores_the_json_columns() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "json").await;
    let session = unique("session");
    let mut analysis = minimal();
    analysis.skill_scores = Some(HashMap::from([("sql".to_owned(), 7i16)]));
    analysis.goal_outcome_map = Some(vec![GoalOutcomeMapping {
        goal: "Ship".to_owned(),
        outcome: "Shipped".to_owned(),
        achieved: true,
    }]);
    analysis.best_practices_checklist = Some(vec![BestPracticeItem {
        practice: "Tests first".to_owned(),
        score: "good".to_owned(),
        note: "covered".to_owned(),
    }]);

    insert_session_analysis(&db.pool, &SessionId::new(session.clone()), &user, &analysis).await;

    let row: (serde_json::Value, serde_json::Value, serde_json::Value) = sqlx::query_as(
        "SELECT skill_scores, goal_outcome_map, best_practices_checklist
         FROM session_analyses WHERE session_id = $1",
    )
    .bind(&session)
    .fetch_one(&*db.pool)
    .await
    .expect("read json columns");

    assert_eq!(row.0["sql"], 7);
    assert_eq!(row.1[0]["achieved"], true);
    assert_eq!(row.2[0]["practice"], "Tests first");
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_analysis_records_the_plan_mode_and_client_surface_flags() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "surface").await;
    let session = unique("session");
    let mut analysis = minimal();
    analysis.plan_mode_used = Some(true);
    analysis.client_surface = Some("cowork".to_owned());

    insert_session_analysis(&db.pool, &SessionId::new(session.clone()), &user, &analysis).await;

    let stored = read_analysis(&db.pool, &session).await;
    assert!(stored.plan_mode_used);
    assert_eq!(stored.client_surface, "cowork");
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_analysis_upserts_rather_than_duplicating_the_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = analysis_user(&db.pool, "upsert").await;
    let session = unique("session");
    let id = SessionId::new(session.clone());
    insert_session_analysis(&db.pool, &id, &user, &minimal()).await;
    let mut second = minimal();
    second.title = "Revised title".to_owned();
    second.quality_score = 91;

    insert_session_analysis(&db.pool, &id, &user, &second).await;

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM session_analyses WHERE session_id = $1")
            .bind(&session)
            .fetch_one(&*db.pool)
            .await
            .expect("count analyses");
    assert_eq!(count, 1);
    let stored = read_analysis(&db.pool, &session).await;
    assert_eq!(stored.title, "Revised title");
    assert_eq!(stored.quality_score, 91);
    db.cleanup().await;
}
