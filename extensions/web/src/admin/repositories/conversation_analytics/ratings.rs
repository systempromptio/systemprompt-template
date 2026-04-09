use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::admin::types::conversation_analytics::{SessionRating, SkillRating};

pub async fn upsert_session_rating(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &str,
    rating: i16,
    outcome: &str,
    notes: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO session_ratings (id, user_id, session_id, rating, outcome, notes)
        VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5)
        ON CONFLICT (user_id, session_id)
        DO UPDATE SET rating = $3, outcome = $4, notes = $5, updated_at = NOW()",
        user_id.as_str(),
        session_id,
        rating,
        outcome,
        notes,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_skill_rating(
    pool: &PgPool,
    user_id: &UserId,
    skill_name: &str,
    rating: i16,
    notes: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO skill_ratings (id, user_id, skill_name, rating, notes)
        VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4)
        ON CONFLICT (user_id, skill_name)
        DO UPDATE SET rating = $3, notes = $4, updated_at = NOW()",
        user_id.as_str(),
        skill_name,
        rating,
        notes,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_all_session_ratings(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<SessionRating>, sqlx::Error> {
    sqlx::query_as!(
        SessionRating,
        r"SELECT session_id, rating, outcome, notes, updated_at
        FROM session_ratings
        WHERE user_id = $1
        ORDER BY updated_at DESC
        LIMIT 200",
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_all_skill_ratings(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<SkillRating>, sqlx::Error> {
    sqlx::query_as!(
        SkillRating,
        r"SELECT skill_name, rating, notes, updated_at
        FROM skill_ratings
        WHERE user_id = $1
        ORDER BY updated_at DESC
        LIMIT 200",
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}
