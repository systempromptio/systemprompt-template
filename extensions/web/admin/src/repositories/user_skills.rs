use std::collections::HashMap;

use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, CategoryId, SkillId, SourceId, UserId};

use super::super::types::{AgentSkill, CreateSkillRequest, UpdateUserSkillRequest, UserSkill};

pub async fn list_agent_skills(pool: &PgPool) -> Result<Vec<AgentSkill>, sqlx::Error> {
    sqlx::query_as!(
        AgentSkill,
        r#"
        SELECT skill_id as "skill_id: SkillId", name, description, enabled, tags,
            category_id as "category_id: CategoryId",
            source_id as "source_id: SourceId",
            created_at, updated_at
        FROM agent_skills
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_agent_skill(
    pool: &PgPool,
    skill_id: &SkillId,
) -> Result<Option<AgentSkill>, sqlx::Error> {
    sqlx::query_as!(
        AgentSkill,
        r#"
        SELECT skill_id as "skill_id: SkillId", name, description, enabled, tags,
            category_id as "category_id: CategoryId",
            source_id as "source_id: SourceId",
            created_at, updated_at
        FROM agent_skills
        WHERE skill_id = $1
        "#,
        skill_id.as_str(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_user_skills(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<UserSkill>, sqlx::Error> {
    sqlx::query_as!(
        UserSkill,
        r#"SELECT id, user_id as "user_id: UserId", skill_id as "skill_id: SkillId",
            name, description, content, enabled, version, COALESCE(tags, '{}') as "tags!",
            base_skill_id as "base_skill_id: SkillId",
            created_at, updated_at
        FROM user_skills
        WHERE user_id = $1
        ORDER BY created_at DESC"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn create_user_skill(
    pool: &PgPool,
    user_id: &UserId,
    req: &CreateSkillRequest,
) -> Result<UserSkill, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as!(
        UserSkill,
        r#"INSERT INTO user_skills (id, user_id, skill_id, name, description, content, tags, base_skill_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, user_id as "user_id: UserId", skill_id as "skill_id: SkillId",
            name, description, content, enabled, version, COALESCE(tags, '{}') as "tags!",
            base_skill_id as "base_skill_id: SkillId",
            created_at, updated_at"#,
        id,
        user_id.as_str(),
        req.skill_id.as_str(),
        req.name,
        req.description,
        req.content,
        &req.tags,
        req.base_skill_id.as_ref().map(SkillId::as_str),
    )
    .fetch_one(pool)
    .await
}

pub async fn get_or_create_user_skill(
    pool: &PgPool,
    user_id: &UserId,
    req: &CreateSkillRequest,
) -> Result<UserSkill, sqlx::Error> {
    match create_user_skill(pool, user_id, req).await {
        Ok(skill) => Ok(skill),
        Err(_) => {
            sqlx::query_as!(
                UserSkill,
                r#"SELECT id, user_id as "user_id: UserId", skill_id as "skill_id: SkillId",
                    name, description, content, enabled, version, COALESCE(tags, '{}') as "tags!",
                    base_skill_id as "base_skill_id: SkillId",
                    created_at, updated_at
                FROM user_skills
                WHERE user_id = $1 AND skill_id = $2"#,
                user_id.as_str(),
                req.skill_id.as_str(),
            )
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn delete_user_skill(
    pool: &PgPool,
    user_id: &UserId,
    skill_id: &SkillId,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM user_skills WHERE user_id = $1 AND skill_id = $2",
        user_id.as_str(),
        skill_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

#[allow(trivial_casts)]
pub async fn update_user_skill(
    pool: &PgPool,
    user_id: &UserId,
    skill_id: &SkillId,
    req: &UpdateUserSkillRequest,
) -> Result<Option<UserSkill>, sqlx::Error> {
    sqlx::query_as!(
        UserSkill,
        r#"UPDATE user_skills SET
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            content = COALESCE($5, content),
            tags = COALESCE($6, tags),
            updated_at = NOW()
        WHERE user_id = $1 AND skill_id = $2
        RETURNING id, user_id as "user_id: UserId", skill_id as "skill_id: SkillId",
            name, description, content, enabled, version, COALESCE(tags, '{}') as "tags!",
            base_skill_id as "base_skill_id: SkillId",
            created_at, updated_at"#,
        user_id.as_str(),
        skill_id.as_str(),
        req.name.as_deref(),
        req.description.as_deref(),
        req.content.as_deref(),
        &req.tags as &Option<Vec<String>>,
    )
    .fetch_optional(pool)
    .await
}

#[allow(trivial_casts)]
pub async fn fetch_skill_usage_counts(
    pool: &PgPool,
    skill_ids: &[SkillId],
) -> HashMap<String, i64> {
    if skill_ids.is_empty() {
        return HashMap::new();
    }
    let ids: Vec<&str> = skill_ids.iter().map(AsRef::as_ref).collect();
    let rows = sqlx::query!(
        r#"SELECT tool_name as "tool_name!", SUM(event_count)::BIGINT as "usage_count!"
          FROM plugin_usage_daily
          WHERE tool_name = ANY($1)
          GROUP BY tool_name"#,
        &ids as &[&str],
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch skill usage counts");
        vec![]
    });
    rows.into_iter()
        .map(|r| (r.tool_name, r.usage_count))
        .collect()
}

#[allow(trivial_casts)]
pub async fn fetch_skill_avg_ratings(
    pool: &PgPool,
    skill_ids: &[SkillId],
) -> HashMap<String, (f64, i64)> {
    if skill_ids.is_empty() {
        return HashMap::new();
    }
    let ids: Vec<&str> = skill_ids.iter().map(AsRef::as_ref).collect();
    let rows = sqlx::query!(
        r#"SELECT plugin_id as "plugin_id!", COALESCE(AVG(rating)::FLOAT8, 0.0) as "avg_rating!", COUNT(*)::BIGINT as "rating_count!"
          FROM plugin_ratings
          WHERE plugin_id = ANY($1)
          GROUP BY plugin_id"#,
        &ids as &[&str],
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch skill avg ratings");
        vec![]
    });
    rows.into_iter()
        .map(|r| (r.plugin_id, (r.avg_rating, r.rating_count)))
        .collect()
}

pub async fn fetch_agent_usage_counts(
    _pool: &PgPool,
    _agent_ids: &[AgentId],
) -> HashMap<String, i64> {
    HashMap::new()
}
