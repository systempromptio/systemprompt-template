//! Observed skill signals and owner-scoped request correlation.
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use systemprompt::identifiers::{SessionId, UserId};
pub mod portfolio;
pub mod version_impact;

#[derive(Debug)]
pub struct AnalysisFilter {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub skill: Option<String>,
    pub user: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, FromRow, Serialize)]
pub struct Totals {
    pub uses: i64,
    pub conversations: i64,
    pub linked: i64,
    pub requests: i64,
    pub measured: i64,
    pub cost: i64,
    pub tokens: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct SkillRow {
    pub skill: String,
    pub uses: i64,
    pub conversations: i64,
    pub linked: i64,
    pub measured: i64,
    pub cost: i64,
    pub tokens: i64,
    pub failures: i64,
    pub total_rows: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct AnalysisConversationRow {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub uses: i64,
    pub requests: i64,
    pub measured: i64,
    pub cost: i64,
    pub tokens: i64,
    pub failures: i64,
    pub contexts: Vec<String>,
    pub tools: String,
    pub observed_tools: String,
    pub assessment: Option<String>,
    pub rating: Option<i16>,
    pub total_rows: i64,
}

macro_rules! filtered {
    ($sql:expr, $f:expr, $row:ty) => {
        sqlx::query_as::<_, $row>($sql)
            .bind($f.start)
            .bind($f.end)
            .bind(&$f.skill)
            .bind(&$f.user)
            .bind(&$f.model)
    };
}

pub async fn get_totals(pool: &PgPool, f: &AnalysisFilter) -> Result<Totals, sqlx::Error> {
    let sql = concat!(
        include_str!("facts.sql"),
        "SELECT coalesce(sum(uses),0)::bigint AS uses, count(DISTINCT (user_id,session_id)) AS conversations, count(DISTINCT (user_id,session_id)) FILTER(WHERE requests>0) AS linked, (SELECT count(DISTINCT id) FROM requests) AS requests, (SELECT count(DISTINCT id) FROM requests WHERE input_tokens IS NOT NULL AND output_tokens IS NOT NULL AND status='completed') AS measured, (SELECT coalesce(sum(cost_microdollars),0)::bigint FROM (SELECT DISTINCT id,cost_microdollars FROM requests WHERE cost_microdollars IS NOT NULL) d) AS cost, (SELECT coalesce(sum(tokens),0)::bigint FROM (SELECT DISTINCT id,coalesce(input_tokens,0)::bigint+coalesce(output_tokens,0)+coalesce(cache_read_tokens,0)+coalesce(cache_creation_tokens,0) AS tokens FROM requests) d) AS tokens FROM facts"
    );
    filtered!(sql, f, Totals).fetch_one(pool).await
}

pub async fn list_skills(
    pool: &PgPool,
    f: &AnalysisFilter,
    offset: i64,
) -> Result<Vec<SkillRow>, sqlx::Error> {
    let sql = concat!(
        include_str!("facts.sql"),
        "SELECT skill,sum(uses)::bigint AS uses,count(*) AS conversations,count(*) FILTER(WHERE requests>0) AS linked,sum(measured)::bigint AS measured,sum(cost)::bigint AS cost,sum(tokens)::bigint AS tokens,sum(failures)::bigint AS failures,count(*) OVER() AS total_rows FROM facts GROUP BY skill ORDER BY uses DESC,skill LIMIT 50 OFFSET $6"
    );
    filtered!(sql, f, SkillRow)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn list_conversations(
    pool: &PgPool,
    f: &AnalysisFilter,
    offset: i64,
) -> Result<Vec<AnalysisConversationRow>, sqlx::Error> {
    let sql = concat!(
        include_str!("facts.sql"),
        "SELECT f.session_id,f.user_id,f.uses,f.requests,f.measured,f.cost,f.tokens,f.failures, ARRAY(SELECT DISTINCT context_id FROM requests r WHERE r.user_id=f.user_id AND r.native_session=f.session_id ORDER BY context_id) AS contexts,coalesce((SELECT string_agg(name,', ' ORDER BY name) FROM (SELECT DISTINCT t.tool_name AS name FROM requests r JOIN ai_request_tool_calls t ON t.request_id=r.id WHERE r.user_id=f.user_id AND r.native_session=f.session_id) tools),'') AS tools,coalesce((SELECT string_agg(tool_name || ' (' || uses || ')',', ' ORDER BY tool_name) FROM (SELECT tool_name,count(*) AS uses FROM plugin_usage_events e WHERE e.user_id=f.user_id AND e.session_id=f.session_id AND e.event_type IN ('PostToolUse','PostToolUseFailure') AND e.tool_name IS NOT NULL AND e.created_at >= $1 AND e.created_at < $2 GROUP BY tool_name) observed),'') AS observed_tools,a.outcome AS assessment,h.rating,count(*) OVER() AS total_rows FROM facts f LEFT JOIN session_analyses a ON a.session_id=f.session_id AND a.user_id=f.user_id LEFT JOIN session_ratings h ON h.session_id=f.session_id AND h.user_id=f.user_id ORDER BY f.cost DESC,f.session_id,f.user_id LIMIT 50 OFFSET $6"
    );
    filtered!(sql, f, AnalysisConversationRow)
        .bind(offset)
        .fetch_all(pool)
        .await
}
