//! One conversation's fact row, read by `/admin/analysis/conversations/{id}`.
//!
//! The page refreshes the row first so what it shows is exact, not up to a
//! rollup tick old. The other planes (turns, tools, decisions, skills,
//! safety) are in `planes`. The manual judge request lives here too.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};
use systemprompt_web_shared::{GroupId, ProjectId};

use super::{ContinuationLink, ConversationFactRow};

// Why: re-derives the fact row for one context; `false` when the context has
// no conversation requests (nothing to show).
pub async fn refresh_conversation_facts(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<bool, sqlx::Error> {
    let written = sqlx::query_scalar!(
        r#"SELECT refresh_conversation_facts(ARRAY[$1::text]) AS "written!""#,
        context_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(written > 0)
}

// Why: the fact row's columns, named so the mapping below is a plain `From`
// and the query stays one statement.
struct FactRecord {
    context_id: ContextId,
    user_id: UserId,
    display_name: Option<String>,
    session_id: Option<SessionId>,
    client_session_id: Option<String>,
    group_id: Option<GroupId>,
    project_id: Option<ProjectId>,
    group_name: Option<String>,
    project_name: Option<String>,
    client_kind: String,
    client_attestation: String,
    wire_protocol: String,
    model: Option<String>,
    provider: Option<String>,
    models: Vec<String>,
    providers: Vec<String>,
    request_count: i64,
    turn_count: i64,
    side_call_count: i64,
    side_call_cost_microdollars: i64,
    error_count: i64,
    rejected_count: i64,
    streaming_count: i64,
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_creation_tokens: i64,
    reasoning_tokens: i64,
    cost_microdollars: i64,
    p50_latency_ms: Option<i32>,
    p95_latency_ms: Option<i32>,
    max_latency_ms: Option<i32>,
    tool_calls_intended: i64,
    tool_calls_executed: i64,
    tool_calls_failed: i64,
    artifact_count: i64,
    artifact_files: i64,
    artifact_cards: i64,
    safety_findings: i64,
    safety_blocked: i64,
    gov_allow: i64,
    gov_warn: i64,
    gov_deny: i64,
    prompt_count: i64,
    hook_event_count: i64,
    hook_status: Option<String>,
    skill_invocations: i64,
    skills: Vec<String>,
    first_at: DateTime<Utc>,
    last_at: DateTime<Utc>,
    duration_seconds: i64,
    judge_status: Option<String>,
    judge_title: Option<String>,
    category: Option<String>,
    summary: Option<String>,
    tags: Option<Vec<String>>,
    skills_used: Option<Vec<String>>,
    outcome: Option<String>,
    completion: Option<i16>,
    completion_rationale: Option<String>,
    confidence: Option<f32>,
    classified_at: Option<DateTime<Utc>>,
    judge_model: Option<String>,
    judge_cost_microdollars: Option<i64>,
    judge_tokens: Option<i64>,
    judge_trigger: Option<String>,
    title: String,
}

impl From<FactRecord> for ConversationFactRow {
    fn from(r: FactRecord) -> Self {
        Self {
            context_id: r.context_id,
            title: r.title,
            user_id: r.user_id,
            display_name: r.display_name,
            session_id: r.session_id,
            client_session_id: r.client_session_id,
            group_id: r.group_id,
            project_id: r.project_id,
            group_name: r.group_name,
            project_name: r.project_name,
            client_kind: r.client_kind,
            client_attestation: r.client_attestation,
            wire_protocol: r.wire_protocol,
            model: r.model,
            provider: r.provider,
            models: r.models,
            providers: r.providers,
            request_count: r.request_count,
            turn_count: r.turn_count,
            side_call_count: r.side_call_count,
            side_call_cost_microdollars: r.side_call_cost_microdollars,
            error_count: r.error_count,
            rejected_count: r.rejected_count,
            streaming_count: r.streaming_count,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cache_read_tokens: r.cache_read_tokens,
            cache_creation_tokens: r.cache_creation_tokens,
            reasoning_tokens: r.reasoning_tokens,
            total_tokens: r.input_tokens + r.output_tokens,
            cost_microdollars: r.cost_microdollars,
            p50_latency_ms: r.p50_latency_ms,
            p95_latency_ms: r.p95_latency_ms,
            max_latency_ms: r.max_latency_ms,
            tool_calls_intended: r.tool_calls_intended,
            tool_calls_executed: r.tool_calls_executed,
            tool_calls_failed: r.tool_calls_failed,
            artifact_count: r.artifact_count,
            artifact_files: r.artifact_files,
            artifact_cards: r.artifact_cards,
            safety_findings: r.safety_findings,
            safety_blocked: r.safety_blocked,
            gov_allow: r.gov_allow,
            gov_warn: r.gov_warn,
            gov_deny: r.gov_deny,
            prompt_count: r.prompt_count,
            hook_event_count: r.hook_event_count,
            hook_status: r.hook_status,
            skill_invocations: r.skill_invocations,
            skills: r.skills,
            first_at: r.first_at,
            last_at: r.last_at,
            duration_seconds: r.duration_seconds,
            active_ms: 0,
            judge_status: r.judge_status,
            judge_title: r.judge_title,
            category: r.category,
            summary: r.summary,
            tags: r.tags.unwrap_or_default(),
            skills_used: r.skills_used.unwrap_or_default(),
            outcome: r.outcome,
            completion: r.completion,
            completion_rationale: r.completion_rationale,
            confidence: r.confidence,
            classified_at: r.classified_at,
            judge_model: r.judge_model,
            judge_cost_microdollars: r.judge_cost_microdollars,
            judge_tokens: r.judge_tokens.unwrap_or(0),
            judge_trigger: r.judge_trigger,
            turn_tokens: Vec::new(),
            continuation: ContinuationLink::default(),
        }
    }
}

pub async fn find_conversation_facts(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Option<ConversationFactRow>, sqlx::Error> {
    // Why: `f` is the driving relation, but PostgreSQL's outer-join plan can
    // make SQLx infer its non-null columns as nullable. These `!` aliases
    // mirror the `conversation_facts` schema; they change only compile-time
    // decoding, not the query's runtime join semantics.
    let row = sqlx::query_as!(
        FactRecord,
        r#"SELECT f.context_id AS "context_id!: ContextId", f.user_id AS "user_id!: UserId",
                  u.display_name, f.session_id AS "session_id?: SessionId", f.client_session_id,
                  f.group_id AS "group_id?: GroupId", f.project_id AS "project_id?: ProjectId",
                  g.name AS group_name, p.name AS project_name,
                  f.client_kind AS "client_kind!", f.client_attestation AS "client_attestation!",
                  f.wire_protocol AS "wire_protocol!", f.model, f.provider,
                  f.models AS "models!", f.providers AS "providers!",
                  f.request_count AS "request_count!", f.turn_count AS "turn_count!",
                  f.side_call_count AS "side_call_count!",
                  f.side_call_cost_microdollars AS "side_call_cost_microdollars!",
                  f.error_count AS "error_count!", f.rejected_count AS "rejected_count!",
                  f.streaming_count AS "streaming_count!", f.input_tokens AS "input_tokens!",
                  f.output_tokens AS "output_tokens!",
                  f.cache_read_tokens AS "cache_read_tokens!",
                  f.cache_creation_tokens AS "cache_creation_tokens!",
                  f.reasoning_tokens AS "reasoning_tokens!",
                  f.cost_microdollars AS "cost_microdollars!", f.p50_latency_ms, f.p95_latency_ms,
                  f.max_latency_ms, f.tool_calls_intended AS "tool_calls_intended!",
                  f.tool_calls_executed AS "tool_calls_executed!",
                  f.tool_calls_failed AS "tool_calls_failed!",
                  f.artifact_count AS "artifact_count!", f.artifact_files AS "artifact_files!",
                  f.artifact_cards AS "artifact_cards!", f.safety_findings AS "safety_findings!",
                  f.safety_blocked AS "safety_blocked!", f.gov_allow AS "gov_allow!",
                  f.gov_warn AS "gov_warn!", f.gov_deny AS "gov_deny!",
                  f.prompt_count AS "prompt_count!", f.hook_event_count AS "hook_event_count!",
                  f.hook_status, f.skill_invocations AS "skill_invocations!",
                  f.skills AS "skills!", f.first_at AS "first_at!", f.last_at AS "last_at!",
                  f.duration_seconds AS "duration_seconds!",
                  a.status AS "judge_status?", a.title AS "judge_title?", a.category AS "category?",
                  a.summary AS "summary?", a.tags AS "tags?", a.skills_used AS "skills_used?",
                  a.outcome AS "outcome?", a.completion AS "completion?",
                  a.completion_rationale AS "completion_rationale?", a.confidence AS "confidence?",
                  a.classified_at AS "classified_at?", a.model AS "judge_model?",
                  a.cost_microdollars AS "judge_cost_microdollars?",
                  (COALESCE(a.input_tokens, 0) + COALESCE(a.output_tokens, 0))::bigint AS "judge_tokens?",
                  a.trigger AS "judge_trigger?",
                  conversation_title(f.context_id, f.client_session_id) AS "title!"
           FROM conversation_facts f
           LEFT JOIN conversation_analyses a ON a.context_id = f.context_id
           LEFT JOIN users u ON u.id = f.user_id
           LEFT JOIN groups g ON g.id = f.group_id
           LEFT JOIN projects p ON p.id = f.project_id
           WHERE f.context_id = $1"#,
        context_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(ConversationFactRow::from))
}

// Why: queues one conversation for the judge as a manual request; `false`
// when the context has no conversation requests to judge.
pub async fn insert_manual_judgement(
    pool: &PgPool,
    context_id: &ContextId,
    requested_by: &UserId,
) -> Result<bool, sqlx::Error> {
    let user_id = sqlx::query_scalar!(
        r#"SELECT f.user_id AS "user_id!" FROM conversation_facts f WHERE f.context_id = $1"#,
        context_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    let Some(user_id) = user_id else {
        return Ok(false);
    };
    sqlx::query!(
        r#"INSERT INTO conversation_analyses (context_id, user_id, status, trigger, requested_by)
           VALUES ($1, $2, 'pending', 'manual', $3)
           ON CONFLICT (context_id) DO UPDATE
               SET status = 'pending', attempts = 0, next_attempt = clock_timestamp(),
                   lease_token = NULL, lease_until = NULL, trigger = 'manual',
                   requested_by = EXCLUDED.requested_by, updated_at = clock_timestamp()"#,
        context_id.as_str(),
        user_id,
        requested_by.as_str()
    )
    .execute(pool)
    .await?;
    Ok(true)
}

// Why: queues many conversations at once for the judge — the bulk bar's
// "Judge selected" and the toolbar's "Judge all unjudged in view". Returns
// how many rows were queued; contexts without a fact row are skipped.
pub async fn insert_manual_judgements(
    pool: &PgPool,
    context_ids: &[String],
    requested_by: &UserId,
) -> Result<i64, sqlx::Error> {
    if context_ids.is_empty() {
        return Ok(0);
    }
    let queued = sqlx::query_scalar!(
        r#"WITH queued AS (
               INSERT INTO conversation_analyses (context_id, user_id, status, trigger, requested_by)
               SELECT f.context_id, f.user_id, 'pending', 'manual', $2
               FROM conversation_facts f WHERE f.context_id = ANY($1)
               ON CONFLICT (context_id) DO UPDATE
                   SET status = 'pending', attempts = 0, next_attempt = clock_timestamp(),
                       lease_token = NULL, lease_until = NULL, trigger = 'manual',
                       requested_by = EXCLUDED.requested_by, updated_at = clock_timestamp()
               RETURNING 1)
           SELECT COUNT(*)::bigint AS "queued!" FROM queued"#,
        context_ids,
        requested_by.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(queued)
}
