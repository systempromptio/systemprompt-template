//! Each person's primary group and project — the key exclusive attribution
//! counts by.
//!
//! The automatic rule is the same on both sides: among the containers a person
//! belongs to, prefer one the directory placed them in, then the one with the
//! most members, then the alphabetically first id. Recomputation rewrites only
//! the rows it decided itself: an operator's `manual` row survives every run,
//! and is the only way to override the rule.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// One person's attribution key, with who decided it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeDefaults {
    pub primary_group_id: Option<String>,
    pub primary_project_id: Option<String>,
    pub source: String,
}

pub async fn find_scope_defaults(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<ScopeDefaults>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT primary_group_id, primary_project_id, source
           FROM user_scope_defaults WHERE user_id = $1"#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| ScopeDefaults {
        primary_group_id: row.primary_group_id,
        primary_project_id: row.primary_project_id,
        source: row.source,
    }))
}

pub async fn set_scope_defaults(
    pool: &PgPool,
    user_id: &UserId,
    primary_group_id: Option<&str>,
    primary_project_id: Option<&str>,
) -> Result<ScopeDefaults, sqlx::Error> {
    let row = sqlx::query!(
        r#"INSERT INTO user_scope_defaults
               (user_id, primary_group_id, primary_project_id, source, updated_at)
           VALUES ($1, $2, $3, 'manual', NOW())
           ON CONFLICT (user_id) DO UPDATE SET
               primary_group_id = EXCLUDED.primary_group_id,
               primary_project_id = EXCLUDED.primary_project_id,
               source = 'manual',
               updated_at = NOW()
           RETURNING primary_group_id, primary_project_id, source"#,
        user_id.as_str(),
        primary_group_id,
        primary_project_id
    )
    .fetch_one(pool)
    .await?;
    Ok(ScopeDefaults {
        primary_group_id: row.primary_group_id,
        primary_project_id: row.primary_project_id,
        source: row.source,
    })
}

pub async fn recompute_scope_defaults(pool: &PgPool) -> Result<u64, sqlx::Error> {
    recompute(pool, None).await
}

// Why: a directory sign-in rewrites one person's memberships, and waiting for
// the hourly pass would leave them unattributed until it runs — on a fresh
// install, that is an estate reading zero for an hour.
pub async fn recompute_scope_defaults_for_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<u64, sqlx::Error> {
    recompute(pool, Some(user_id)).await
}

async fn recompute(pool: &PgPool, user_id: Option<&UserId>) -> Result<u64, sqlx::Error> {
    let written = sqlx::query!(
        r"INSERT INTO user_scope_defaults
              (user_id, primary_group_id, primary_project_id, source, updated_at)
          SELECT u.id,
                 (SELECT ug.group_id
                    FROM user_groups ug
                   WHERE ug.user_id = u.id
                   ORDER BY EXISTS (
                                SELECT 1 FROM group_members gm
                                WHERE gm.group_id = ug.group_id
                                  AND gm.user_id = ug.user_id
                                  AND gm.source = 'adfs') DESC,
                            (SELECT COUNT(DISTINCT x.user_id)
                               FROM user_groups x
                              WHERE x.group_id = ug.group_id) DESC,
                            ug.group_id
                   LIMIT 1),
                 (SELECT pm.project_id
                    FROM project_members pm
                   WHERE pm.user_id = u.id
                   ORDER BY (pm.source = 'adfs') DESC,
                            (SELECT COUNT(DISTINCT y.user_id)
                               FROM project_members y
                              WHERE y.project_id = pm.project_id) DESC,
                            pm.project_id
                   LIMIT 1),
                 'auto', NOW()
          FROM users u
          WHERE NOT ('anonymous' = ANY(u.roles))
            AND ($1::TEXT IS NULL OR u.id = $1)
          ON CONFLICT (user_id) DO UPDATE SET
              primary_group_id = EXCLUDED.primary_group_id,
              primary_project_id = EXCLUDED.primary_project_id,
              updated_at = NOW()
          WHERE user_scope_defaults.source = 'auto'",
        user_id.map(UserId::as_str)
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(written)
}
