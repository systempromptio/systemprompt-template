//! Paged client-reported session costs for the Sessions tab.
//!
//! `session_cost_snapshots` holds one statusline-reported row per session with
//! set-not-increment semantics, so every figure is the client's own running
//! total rather than a gateway measurement, and the page says so. Ratings are
//! joined from `session_ratings`, which a person fills in by hand and most
//! sessions never carry.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Clone)]
pub struct SessionCostRow {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub model: Option<String>,
    pub total_cost_microdollars: i64,
    pub context_window_size: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub rating: Option<i16>,
    pub outcome: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_session_costs_paged(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SessionCostRow>, i64), sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            s.session_id AS "session_id!: SessionId",
            s.user_id AS "user_id!: UserId",
            s.model,
            COALESCE(s.total_cost_microdollars, 0)::BIGINT AS "cost!",
            COALESCE(s.context_window_size, 0)::BIGINT AS "context!",
            COALESCE(s.input_tokens, 0)::BIGINT AS "input_tokens!",
            COALESCE(s.output_tokens, 0)::BIGINT AS "output_tokens!",
            COALESCE(s.cache_read_input_tokens, 0)::BIGINT AS "cache_read!",
            r.rating AS "rating?",
            r.outcome AS "outcome?",
            s.updated_at AS "updated_at!",
            COUNT(*) OVER ()::BIGINT AS "total!"
        FROM session_cost_snapshots s
        LEFT JOIN session_ratings r ON r.session_id = s.session_id
        WHERE s.updated_at >= $1 AND s.updated_at < $2
          AND ($3::TEXT[] IS NULL OR s.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR s.user_id = $4)
        ORDER BY COALESCE(s.total_cost_microdollars, 0) DESC, s.updated_at DESC
        LIMIT $5 OFFSET $6
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total);
    Ok((
        rows.into_iter()
            .map(|r| SessionCostRow {
                session_id: r.session_id,
                user_id: r.user_id,
                model: r.model,
                total_cost_microdollars: r.cost,
                context_window_size: r.context,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                cache_read_tokens: r.cache_read,
                rating: r.rating,
                outcome: r.outcome,
                updated_at: r.updated_at,
            })
            .collect(),
        total,
    ))
}

/// How sessions were rated in the window.
///
/// Separate from the listing because the answer is about the rated minority,
/// and a mean quoted without the count behind it invites reading four ratings
/// as a verdict.
#[derive(Debug, Default, Clone, Copy)]
pub struct SessionRatingStats {
    pub rated: i64,
    pub avg_rating: Option<f64>,
    pub good: i64,
    pub poor: i64,
}

pub async fn get_session_rating_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<SessionRatingStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::BIGINT AS "rated!",
            AVG(r.rating::DOUBLE PRECISION) AS avg_rating,
            COUNT(*) FILTER (WHERE r.rating >= 4)::BIGINT AS "good!",
            COUNT(*) FILTER (WHERE r.rating <= 2)::BIGINT AS "poor!"
        FROM session_ratings r
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR r.user_id = $4)
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
    )
    .fetch_one(pool)
    .await?;

    Ok(SessionRatingStats {
        rated: row.rated,
        avg_rating: row.avg_rating,
        good: row.good,
        poor: row.poor,
    })
}
