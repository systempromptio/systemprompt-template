use sqlx::PgPool;
use systemprompt::identifiers::{SkillId, UserId};

use crate::types::conversation_analytics::{
    EntityEffectiveness, EntityHint, EntityLastUsed, EntityQualityTrend, HookSessionQuality,
    SkillEffectiveness,
};

pub async fn fetch_skill_effectiveness(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<SkillEffectiveness>, sqlx::Error> {
    sqlx::query_as!(
        SkillEffectiveness,
        r#"SELECT
            sel.entity_name AS skill_name,
            COALESCE(sel.entity_id, sel.entity_name) AS "skill_id!: SkillId",
            COALESCE(SUM(sel.usage_count), 0)::BIGINT AS "total_uses!",
            COUNT(DISTINCT sel.session_id)::BIGINT AS "sessions_used_in!",
            COALESCE(AVG(
                COALESCE(
                    (sa.skill_scores ->> sel.entity_name)::FLOAT8,
                    sa.quality_score::FLOAT8
                )
            ), 0.0) AS "avg_effectiveness!",
            COUNT(sa.quality_score)::BIGINT AS "scored_sessions!",
            COALESCE(
                COUNT(*) FILTER (WHERE sa.goal_achieved = 'yes') * 100.0 /
                NULLIF(COUNT(sa.session_id), 0)::FLOAT8,
                0.0
            ) AS "goal_achievement_pct!"
        FROM session_entity_links sel
        LEFT JOIN session_analyses sa
            ON sa.session_id = sel.session_id AND sa.user_id = sel.user_id
        WHERE sel.user_id = $1 AND sel.entity_type = 'skill'
        GROUP BY sel.entity_name, sel.entity_id
        ORDER BY 3 DESC"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_entity_effectiveness(
    pool: &PgPool,
    user_id: &UserId,
    entity_type: &str,
) -> Result<Vec<EntityEffectiveness>, sqlx::Error> {
    sqlx::query_as!(
        EntityEffectiveness,
        r#"SELECT
            sel.entity_name AS "entity_name!",
            COALESCE(SUM(sel.usage_count), 0)::BIGINT AS "total_uses!",
            COUNT(DISTINCT sel.session_id)::BIGINT AS "sessions_used_in!",
            COALESCE(AVG(sa.quality_score::FLOAT8), 0.0) AS "avg_effectiveness!",
            COUNT(sa.quality_score)::BIGINT AS "scored_sessions!",
            COALESCE(
                COUNT(*) FILTER (WHERE sa.goal_achieved = 'yes') * 100.0 /
                NULLIF(COUNT(sa.session_id), 0)::FLOAT8,
                0.0
            ) AS "goal_achievement_pct!"
        FROM session_entity_links sel
        LEFT JOIN session_analyses sa
            ON sa.session_id = sel.session_id AND sa.user_id = sel.user_id
        WHERE sel.user_id = $1 AND sel.entity_type = $2
        GROUP BY sel.entity_name
        ORDER BY 2 DESC"#,
        user_id.as_str(),
        entity_type,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_entity_last_used(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<EntityLastUsed>, sqlx::Error> {
    sqlx::query_as!(
        EntityLastUsed,
        r#"SELECT
            entity_type AS "entity_type!",
            entity_name AS "entity_name!",
            MAX(last_seen_at) AS "last_used!"
        FROM session_entity_links
        WHERE user_id = $1
        GROUP BY entity_type, entity_name"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_entity_quality_trend(
    pool: &PgPool,
    user_id: &UserId,
    entity_type: &str,
) -> Result<Vec<EntityQualityTrend>, sqlx::Error> {
    sqlx::query_as!(
        EntityQualityTrend,
        r#"SELECT
            sel.entity_name AS "entity_name!",
            COALESCE(AVG(sa.quality_score::FLOAT8) FILTER (
                WHERE sa.created_at >= NOW() - INTERVAL '7 days'
            ), 0.0)::FLOAT8 AS "recent_avg!",
            COALESCE(AVG(sa.quality_score::FLOAT8) FILTER (
                WHERE sa.created_at >= NOW() - INTERVAL '14 days'
                  AND sa.created_at < NOW() - INTERVAL '7 days'
            ), 0.0)::FLOAT8 AS "previous_avg!"
        FROM session_entity_links sel
        JOIN session_analyses sa
            ON sa.session_id = sel.session_id AND sa.user_id = sel.user_id
        WHERE sel.user_id = $1 AND sel.entity_type = $2
        GROUP BY sel.entity_name"#,
        user_id.as_str(),
        entity_type,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_entity_improvement_hints(
    pool: &PgPool,
    user_id: &UserId,
    entity_type: &str,
) -> Result<Vec<EntityHint>, sqlx::Error> {
    sqlx::query_as!(
        EntityHint,
        r#"SELECT DISTINCT ON (sel.entity_name)
            sel.entity_name AS "entity_name!",
            sa.improvement_hints AS "hint!"
        FROM session_entity_links sel
        JOIN session_analyses sa
            ON sa.session_id = sel.session_id AND sa.user_id = sel.user_id
        WHERE sel.user_id = $1 AND sel.entity_type = $2
          AND sa.improvement_hints IS NOT NULL AND sa.improvement_hints != ''
        ORDER BY sel.entity_name, sa.created_at DESC"#,
        user_id.as_str(),
        entity_type,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_hook_session_quality(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<HookSessionQuality>, sqlx::Error> {
    sqlx::query_as!(
        HookSessionQuality,
        r#"SELECT
            pue.event_type AS "event_type!",
            COUNT(DISTINCT pue.session_id)::BIGINT AS "session_count!",
            COALESCE(AVG(sa.quality_score::FLOAT8), 0.0) AS "avg_quality!",
            COALESCE(
                COUNT(*) FILTER (WHERE sa.goal_achieved = 'yes') * 100.0 /
                NULLIF(COUNT(sa.session_id), 0)::FLOAT8,
                0.0
            ) AS "goal_achievement_pct!"
        FROM plugin_usage_events pue
        JOIN session_analyses sa
            ON sa.session_id = pue.session_id AND sa.user_id = pue.user_id
        WHERE pue.user_id = $1
        GROUP BY pue.event_type"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}
