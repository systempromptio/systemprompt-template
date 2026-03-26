use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{AgentSkill, CreateSkillRequest, UpdateUserSkillRequest, UserSkill};

pub async fn get_agent_skills_enabled_map(
    pool: &Arc<PgPool>,
) -> Result<HashMap<String, bool>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, bool)>("SELECT skill_id, enabled FROM agent_skills")
        .fetch_all(pool.as_ref())
        .await?;
    Ok(rows.into_iter().collect())
}

pub async fn list_agent_skills(pool: &Arc<PgPool>) -> Result<Vec<AgentSkill>, sqlx::Error> {
    sqlx::query_as::<_, AgentSkill>(
        r"
        SELECT skill_id, name, description, enabled, tags, category_id, source_id, created_at, updated_at
        FROM agent_skills
        ORDER BY name ASC
        ",
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_agent_skill(
    pool: &Arc<PgPool>,
    skill_id: &str,
) -> Result<Option<AgentSkill>, sqlx::Error> {
    sqlx::query_as::<_, AgentSkill>(
        r"
        SELECT skill_id, name, description, enabled, tags, category_id, source_id, created_at, updated_at
        FROM agent_skills
        WHERE skill_id = $1
        ",
    )
    .bind(skill_id)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn update_agent_skill_enabled(
    pool: &Arc<PgPool>,
    skill_id: &str,
    enabled: bool,
) -> Result<Option<AgentSkill>, sqlx::Error> {
    sqlx::query_as::<_, AgentSkill>(
        r"
        UPDATE agent_skills
        SET enabled = $2, updated_at = NOW()
        WHERE skill_id = $1
        RETURNING skill_id, name, description, enabled, tags, category_id, source_id, created_at, updated_at
        ",
    )
    .bind(skill_id)
    .bind(enabled)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn list_user_skills(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<UserSkill>, sqlx::Error> {
    sqlx::query_as::<_, UserSkill>(
        r"
        SELECT id, user_id, skill_id, name, description, content, enabled, version, tags, base_skill_id, created_at, updated_at
        FROM user_skills
        WHERE user_id = $1
        ORDER BY created_at DESC
        ",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn create_user_skill(
    pool: &Arc<PgPool>,
    user_id: &str,
    req: &CreateSkillRequest,
) -> Result<UserSkill, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as::<_, UserSkill>(
        r"
        INSERT INTO user_skills (id, user_id, skill_id, name, description, content, tags, base_skill_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, user_id, skill_id, name, description, content, enabled, version, tags, base_skill_id, created_at, updated_at
        ",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&req.skill_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.content)
    .bind(&req.tags)
    .bind(&req.base_skill_id)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn delete_user_skill(
    pool: &Arc<PgPool>,
    user_id: &str,
    skill_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_skills WHERE user_id = $1 AND skill_id = $2")
        .bind(user_id)
        .bind(skill_id)
        .execute(pool.as_ref())
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_user_skill(
    pool: &Arc<PgPool>,
    user_id: &str,
    skill_id: &str,
    req: &UpdateUserSkillRequest,
) -> Result<Option<UserSkill>, sqlx::Error> {
    sqlx::query_as::<_, UserSkill>(
        r"
        UPDATE user_skills SET
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            content = COALESCE($5, content),
            enabled = COALESCE($6, enabled),
            tags = COALESCE($7, tags),
            updated_at = NOW()
        WHERE user_id = $1 AND skill_id = $2
        RETURNING id, user_id, skill_id, name, description, content, enabled, version, tags, base_skill_id, created_at, updated_at
        ",
    )
    .bind(user_id)
    .bind(skill_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.content)
    .bind(req.enabled)
    .bind(&req.tags)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn fetch_skill_usage_counts(
    pool: &Arc<PgPool>,
    skill_ids: &[String],
) -> HashMap<String, i64> {
    if skill_ids.is_empty() {
        return HashMap::new();
    }
    let rows = sqlx::query_as::<_, (String, i64)>(
        r"SELECT tool_name, SUM(event_count)::BIGINT as usage_count
          FROM plugin_usage_daily
          WHERE tool_name = ANY($1)
          GROUP BY tool_name",
    )
    .bind(skill_ids)
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch skill usage counts");
        vec![]
    });
    rows.into_iter().collect()
}

pub async fn fetch_skill_avg_ratings(
    pool: &Arc<PgPool>,
    skill_ids: &[String],
) -> HashMap<String, (f64, i64)> {
    if skill_ids.is_empty() {
        return HashMap::new();
    }
    let rows = sqlx::query_as::<_, (String, f64, i64)>(
        r"SELECT plugin_id, COALESCE(AVG(rating)::FLOAT8, 0.0) as avg_rating, COUNT(*)::BIGINT as rating_count
          FROM plugin_ratings
          WHERE plugin_id = ANY($1)
          GROUP BY plugin_id",
    )
    .bind(skill_ids)
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch skill avg ratings");
        vec![]
    });
    rows.into_iter()
        .map(|(id, avg, cnt)| (id, (avg, cnt)))
        .collect()
}

pub async fn fetch_agent_usage_counts(
    pool: &Arc<PgPool>,
    agent_ids: &[String],
) -> HashMap<String, i64> {
    if agent_ids.is_empty() {
        return HashMap::new();
    }
    let rows = sqlx::query_as::<_, (String, i64)>(
        r"SELECT plugin_id, SUM(event_count)::BIGINT as usage_count
          FROM plugin_usage_daily
          WHERE plugin_id = ANY($1)
          GROUP BY plugin_id",
    )
    .bind(agent_ids)
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch agent usage counts");
        vec![]
    });
    rows.into_iter().collect()
}
