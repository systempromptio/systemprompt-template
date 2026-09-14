//! Revision-attributed impact trends independent of evaluator workers.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug)]
pub struct VersionImpactFilter {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub revision: Option<String>,
    pub traffic_class: Option<String>,
    pub skill: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct VersionImpactRow {
    pub cohort_day: NaiveDate,
    pub skill: String,
    pub revision_id: Option<String>,
    pub publication_generation: Option<i64>,
    pub traffic_class: String,
    pub invocations: i64,
    pub attributed: i64,
    pub requests: i64,
    pub accounted_requests: i64,
    pub failures: i64,
    pub assessed_sessions: i64,
    pub successful_sessions: i64,
    pub average_quality_score: Option<f64>,
    pub tokens: i64,
    pub related_conversation_cost_microdollars: i64,
    pub average_latency_ms: Option<f64>,
}

#[derive(Clone, Copy, Debug, FromRow, Serialize)]
pub struct VersionImpactSummary {
    pub invocations: i64,
    pub attributed: i64,
    pub distinct_requests: i64,
    pub accounted_requests: i64,
    pub deduplicated_request_cost_microdollars: i64,
}

pub async fn list_version_impact(
    pool: &PgPool,
    filter: &VersionImpactFilter,
) -> Result<Vec<VersionImpactRow>, sqlx::Error> {
    sqlx::query_as!(VersionImpactRow, r#"WITH events AS (
        SELECT * FROM analysis_skill_version_events WHERE invoked_at >= $1 AND invoked_at < $2
          AND ($3::TEXT IS NULL OR revision_id=$3) AND ($4::TEXT IS NULL OR traffic_class=$4) AND ($5::TEXT IS NULL OR skill=$5)
    ), event_totals AS (
        SELECT invoked_at::date AS cohort_day,skill,revision_id,publication_generation,traffic_class,count(DISTINCT invocation_id) AS invocations,
               count(DISTINCT invocation_id) FILTER(WHERE attribution_status='verified') AS attributed FROM events GROUP BY 1,2,3,4,5
    ), request_links AS (
        SELECT DISTINCT e.invoked_at::date AS cohort_day,e.skill,e.revision_id,e.publication_generation,COALESCE(er.traffic_class,e.traffic_class) AS traffic_class,
               e.user_id,e.session_id,r.id AS request_id,r.status,r.cost_microdollars,r.input_tokens,r.output_tokens,r.cache_read_tokens,r.cache_creation_tokens,r.latency_ms,
               a.outcome,a.quality_score
          FROM events e JOIN ai_requests r ON r.user_id=e.user_id AND r.client_session_id=e.session_id
          LEFT JOIN eval_request_reservations er ON er.request_id=r.id
          LEFT JOIN session_analyses a ON a.user_id=e.user_id AND a.session_id=e.session_id
         WHERE r.created_at >= $1 AND r.created_at < $2
           AND ($4::TEXT IS NULL OR COALESCE(er.traffic_class,e.traffic_class)=$4)
    ), request_totals AS (
        SELECT cohort_day,skill,revision_id,publication_generation,traffic_class,count(request_id) AS requests,
               count(request_id) FILTER(WHERE input_tokens IS NOT NULL AND output_tokens IS NOT NULL AND status='completed') AS accounted_requests,
               count(request_id) FILTER(WHERE status='failed') AS failures,
               count(DISTINCT (user_id,session_id)) FILTER(WHERE outcome IS NOT NULL) AS assessed_sessions,
               count(DISTINCT (user_id,session_id)) FILTER(WHERE outcome IN ('success','successful','completed')) AS successful_sessions,
               avg(quality_score)::FLOAT8 AS average_quality_score,
               COALESCE(sum(COALESCE(input_tokens,0)+COALESCE(output_tokens,0)+COALESCE(cache_read_tokens,0)+COALESCE(cache_creation_tokens,0)),0)::BIGINT AS tokens,
               COALESCE(sum(cost_microdollars),0)::BIGINT AS related_conversation_cost_microdollars,avg(latency_ms)::FLOAT8 AS average_latency_ms
          FROM request_links GROUP BY 1,2,3,4,5
    ), session_scores AS (
        SELECT DISTINCT cohort_day,skill,revision_id,publication_generation,traffic_class,user_id,session_id,quality_score FROM request_links
    ), cohorts AS (
        SELECT cohort_day,skill,revision_id,publication_generation,traffic_class FROM event_totals UNION
        SELECT cohort_day,skill,revision_id,publication_generation,traffic_class FROM request_totals
    ) SELECT c.cohort_day AS "cohort_day!",c.skill AS "skill!",c.revision_id,c.publication_generation,c.traffic_class AS "traffic_class!",COALESCE(e.invocations,0)::BIGINT AS "invocations!",
        COALESCE(e.attributed,0)::BIGINT AS "attributed!",COALESCE(r.requests,0)::BIGINT AS "requests!",COALESCE(r.accounted_requests,0)::BIGINT AS "accounted_requests!",
        COALESCE(r.failures,0)::BIGINT AS "failures!",COALESCE(r.assessed_sessions,0)::BIGINT AS "assessed_sessions!",COALESCE(r.successful_sessions,0)::BIGINT AS "successful_sessions!",(SELECT avg(s.quality_score)::FLOAT8 FROM session_scores s WHERE s.cohort_day=c.cohort_day AND s.skill=c.skill AND s.revision_id IS NOT DISTINCT FROM c.revision_id AND s.publication_generation IS NOT DISTINCT FROM c.publication_generation AND s.traffic_class=c.traffic_class) AS average_quality_score,
        COALESCE(r.tokens,0)::BIGINT AS "tokens!",COALESCE(r.related_conversation_cost_microdollars,0)::BIGINT AS "related_conversation_cost_microdollars!",r.average_latency_ms
      FROM cohorts c
      LEFT JOIN event_totals e ON e.cohort_day=c.cohort_day AND e.skill=c.skill AND e.revision_id IS NOT DISTINCT FROM c.revision_id AND e.publication_generation IS NOT DISTINCT FROM c.publication_generation AND e.traffic_class=c.traffic_class
      LEFT JOIN request_totals r ON r.cohort_day=c.cohort_day AND r.skill=c.skill AND r.revision_id IS NOT DISTINCT FROM c.revision_id AND r.publication_generation IS NOT DISTINCT FROM c.publication_generation AND r.traffic_class=c.traffic_class
     ORDER BY c.cohort_day,c.skill,c.publication_generation NULLS FIRST"#,
        filter.start, filter.end, filter.revision.as_deref(), filter.traffic_class.as_deref(), filter.skill.as_deref()).fetch_all(pool).await
}

pub async fn get_version_impact_summary(
    pool: &PgPool,
    filter: &VersionImpactFilter,
) -> Result<VersionImpactSummary, sqlx::Error> {
    sqlx::query_as!(VersionImpactSummary, r#"WITH events AS (
        SELECT * FROM analysis_skill_version_events WHERE invoked_at >= $1 AND invoked_at < $2
          AND ($3::TEXT IS NULL OR revision_id=$3) AND ($4::TEXT IS NULL OR traffic_class=$4) AND ($5::TEXT IS NULL OR skill=$5)
    ), linked_requests AS (
        SELECT DISTINCT r.id,r.status,r.cost_microdollars,r.input_tokens,r.output_tokens
          FROM events e JOIN ai_requests r ON r.user_id=e.user_id AND r.client_session_id=e.session_id
          LEFT JOIN eval_request_reservations er ON er.request_id=r.id
         WHERE r.created_at >= $1 AND r.created_at < $2 AND ($4::TEXT IS NULL OR COALESCE(er.traffic_class,e.traffic_class)=$4)
    ) SELECT
        (SELECT count(DISTINCT invocation_id) FROM events)::BIGINT AS "invocations!",
        (SELECT count(DISTINCT invocation_id) FROM events WHERE attribution_status='verified')::BIGINT AS "attributed!",
        (SELECT count(*) FROM linked_requests)::BIGINT AS "distinct_requests!",
        (SELECT count(*) FROM linked_requests WHERE status='completed' AND input_tokens IS NOT NULL AND output_tokens IS NOT NULL)::BIGINT AS "accounted_requests!",
        COALESCE((SELECT sum(cost_microdollars) FROM linked_requests),0)::BIGINT AS "deduplicated_request_cost_microdollars!""#,
        filter.start, filter.end, filter.revision.as_deref(), filter.traffic_class.as_deref(), filter.skill.as_deref()).fetch_one(pool).await
}

#[derive(Debug, FromRow, Serialize)]
pub struct InvocationDrilldown {
    pub invocation_id: String,
    pub user_id: UserId,
    pub session_id: SessionId,
    pub skill: String,
    pub revision_id: Option<String>,
    pub publication_generation: Option<i64>,
    pub traffic_class: String,
    pub attribution_status: String,
    pub request_ids: Vec<String>,
    pub request_count: i64,
    pub accounted_count: i64,
    pub tool_count: i64,
}

pub async fn list_invocation_drilldowns(
    pool: &PgPool,
    filter: &VersionImpactFilter,
) -> Result<Vec<InvocationDrilldown>, sqlx::Error> {
    sqlx::query_as!(InvocationDrilldown, r#"SELECT e.invocation_id AS "invocation_id!",e.user_id AS "user_id!: UserId",e.session_id AS "session_id!: SessionId",e.skill AS "skill!",e.revision_id,e.publication_generation,e.traffic_class AS "traffic_class!",e.attribution_status AS "attribution_status!",
        COALESCE(array_agg(DISTINCT r.id) FILTER(WHERE r.id IS NOT NULL),ARRAY[]::TEXT[]) AS "request_ids!",count(DISTINCT r.id) AS "request_count!",
        count(DISTINCT r.id) FILTER(WHERE r.input_tokens IS NOT NULL AND r.output_tokens IS NOT NULL AND r.status='completed') AS "accounted_count!",
        count(DISTINCT t.id) AS "tool_count!"
      FROM analysis_skill_version_events e LEFT JOIN ai_requests r ON r.user_id=e.user_id AND r.client_session_id=e.session_id AND r.created_at >= $1 AND r.created_at < $2
      LEFT JOIN ai_request_tool_calls t ON t.request_id=r.id
     WHERE e.invoked_at >= $1 AND e.invoked_at < $2 AND ($3::TEXT IS NULL OR e.revision_id=$3) AND ($4::TEXT IS NULL OR e.traffic_class=$4) AND ($5::TEXT IS NULL OR e.skill=$5)
     GROUP BY e.invocation_id,e.user_id,e.session_id,e.skill,e.revision_id,e.publication_generation,e.traffic_class,e.attribution_status,e.invoked_at ORDER BY e.invoked_at DESC LIMIT 200"#,
        filter.start, filter.end, filter.revision.as_deref(), filter.traffic_class.as_deref(), filter.skill.as_deref()).fetch_all(pool).await
}
