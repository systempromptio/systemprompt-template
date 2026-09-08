//! How well a session went — the stored AI analysis and the human ratings.
//!
//! Both are written elsewhere (the hooks pipeline stores one analysis per
//! session, people rate their own sessions), and both are read only here, by
//! `/admin/sessions/{id}`. They are the only judgement of outcome the console
//! holds: everything else on that page counts what happened, not whether it
//! worked.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

/// The stored AI verdict on one session. Absent until the hooks pipeline has
/// summarised the run, which is normal for a session still in flight.
#[derive(Debug, Clone)]
pub struct SessionAnalysisSummary {
    pub title: String,
    pub summary: String,
    pub category: String,
    pub outcome: String,
    pub goal_achieved: String,
    pub quality_score: i16,
    pub tags: Vec<String>,
    pub recommendations: Option<String>,
    pub improvement_hints: Option<String>,
    pub error_analysis: Option<String>,
    pub corrections_count: i32,
    pub session_duration_minutes: Option<i32>,
    pub total_turns: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

/// One person's rating of one session, newest first.
#[derive(Debug, Clone)]
pub struct SessionRatingRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub rating: i16,
    pub outcome: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

pub async fn find_session_analysis(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Option<SessionAnalysisSummary>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
               title             AS "title!",
               summary           AS "summary!",
               category          AS "category!",
               outcome           AS "outcome!",
               goal_achieved     AS "goal_achieved!",
               quality_score     AS "quality_score!",
               tags              AS "tags!",
               recommendations,
               improvement_hints,
               error_analysis,
               corrections_count AS "corrections_count!",
               session_duration_minutes,
               total_turns,
               updated_at        AS "updated_at!"
           FROM session_analyses
           WHERE session_id = $1"#,
        session_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| SessionAnalysisSummary {
        title: r.title,
        summary: r.summary,
        category: r.category,
        outcome: r.outcome,
        goal_achieved: r.goal_achieved,
        quality_score: r.quality_score,
        // Why: stored as one comma-joined string by the hooks writer; empty
        // segments would render as blank chips, so they are dropped here.
        tags: r
            .tags
            .split(',')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
            .collect(),
        recommendations: r.recommendations,
        improvement_hints: r.improvement_hints,
        error_analysis: r.error_analysis,
        corrections_count: r.corrections_count,
        session_duration_minutes: r.session_duration_minutes,
        total_turns: r.total_turns,
        updated_at: r.updated_at,
    }))
}

pub async fn list_session_ratings(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionRatingRow>, sqlx::Error> {
    sqlx::query_as!(
        SessionRatingRow,
        r#"SELECT
               r.user_id      AS "user_id!: UserId",
               u.display_name AS "display_name?",
               r.rating       AS "rating!",
               r.outcome      AS "outcome!",
               r.notes        AS "notes!",
               r.created_at   AS "created_at!"
           FROM session_ratings r
           LEFT JOIN users u ON u.id = r.user_id
           WHERE r.session_id = $1
           ORDER BY r.created_at DESC
           LIMIT 50"#,
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await
}
