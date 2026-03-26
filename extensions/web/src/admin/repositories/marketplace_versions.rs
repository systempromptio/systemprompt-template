use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::admin::types::{
    AllVersionsSummaryRow, MarketplaceChangelogEntry, MarketplaceVersion,
    MarketplaceVersionSummary, NewChangelogEntry,
};

pub async fn get_latest_version_number(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<i32, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT COALESCE(MAX(version_number), 0) as "max!" FROM marketplace_versions WHERE user_id = $1"#,
        user_id.as_str(),
    )
    .fetch_one(pool)
    .await?;

    Ok(row.max)
}

pub async fn create_marketplace_version(
    pool: &PgPool,
    user_id: &UserId,
    version_number: i32,
    version_type: &str,
    snapshot_path: &str,
    skills_snapshot: &serde_json::Value, // JSON: DB jsonb column
) -> Result<MarketplaceVersion, sqlx::Error> {
    sqlx::query_as!(
        MarketplaceVersion,
        r"
        INSERT INTO marketplace_versions (id, user_id, version_number, version_type, snapshot_path, skills_snapshot)
        VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5)
        RETURNING id, user_id, version_number, version_type, snapshot_path, skills_snapshot, created_at
        ",
        user_id.as_str(),
        version_number,
        version_type,
        snapshot_path,
        skills_snapshot,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_marketplace_version(
    pool: &PgPool,
    user_id: &UserId,
    version_id: &str,
) -> Result<Option<MarketplaceVersion>, sqlx::Error> {
    sqlx::query_as!(
        MarketplaceVersion,
        r"
        SELECT id, user_id, version_number, version_type, snapshot_path, skills_snapshot, created_at
        FROM marketplace_versions
        WHERE user_id = $1 AND id = $2
        ",
        user_id.as_str(),
        version_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_marketplace_versions(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<MarketplaceVersionSummary>, sqlx::Error> {
    sqlx::query_as!(
        MarketplaceVersionSummary,
        r#"
        SELECT id, user_id, version_number, version_type, snapshot_path,
               COALESCE(jsonb_array_length(skills_snapshot), 0) AS "skills_count!",
               COALESCE(
                   (SELECT jsonb_agg(jsonb_build_object('skill_id', elem->>'skill_id', 'name', elem->>'name'))
                    FROM jsonb_array_elements(skills_snapshot) AS elem),
                   '[]'::jsonb
               ) AS "skill_names!",
               created_at
        FROM marketplace_versions
        WHERE user_id = $1
        ORDER BY version_number DESC
        LIMIT 3
        "#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn prune_old_versions(
    pool: &PgPool,
    user_id: &UserId,
    keep_count: i32,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r"
        DELETE FROM marketplace_versions
        WHERE user_id = $1
          AND version_number NOT IN (
            SELECT version_number FROM marketplace_versions
            WHERE user_id = $1
            ORDER BY version_number DESC
            LIMIT $2
          )
        RETURNING snapshot_path
        ",
        user_id.as_str(),
        i64::from(keep_count),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.snapshot_path).collect())
}

pub async fn insert_changelog_entries(
    pool: &PgPool,
    entries: &[NewChangelogEntry],
) -> Result<Vec<MarketplaceChangelogEntry>, sqlx::Error> {
    let mut results = Vec::with_capacity(entries.len());

    for entry in entries {
        let row = sqlx::query_as!(
            MarketplaceChangelogEntry,
            r"
            INSERT INTO marketplace_changelog (id, user_id, version_id, action, skill_id, skill_name, detail)
            VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, version_id, action, skill_id, skill_name, detail, created_at
            ",
            entry.user_id.as_str(),
            entry.version_id,
            entry.action,
            entry.skill_id.as_str(),
            entry.skill_name,
            entry.detail,
        )
        .fetch_one(pool)
        .await?;

        results.push(row);
    }

    Ok(results)
}

pub async fn list_changelog(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
) -> Result<Vec<MarketplaceChangelogEntry>, sqlx::Error> {
    sqlx::query_as!(
        MarketplaceChangelogEntry,
        r"
        SELECT id, user_id, version_id, action, skill_id, skill_name, detail, created_at
        FROM marketplace_changelog
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        ",
        user_id.as_str(),
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_all_versions_summary(
    pool: &PgPool,
) -> Result<Vec<AllVersionsSummaryRow>, sqlx::Error> {
    sqlx::query_as!(
        AllVersionsSummaryRow,
        r#"
        SELECT mv.id, mv.user_id AS "user_id: _", u.email AS "email: _", u.display_name,
               mv.version_number, mv.version_type,
               COALESCE(jsonb_array_length(mv.skills_snapshot), 0)::INT AS "skills_count!",
               COALESCE(
                   (SELECT jsonb_agg(jsonb_build_object('skill_id', elem->>'skill_id', 'name', elem->>'name', 'base_skill_id', elem->>'base_skill_id'))
                    FROM jsonb_array_elements(mv.skills_snapshot) AS elem),
                   '[]'::jsonb
               ) AS "skill_names!",
               mv.created_at
        FROM marketplace_versions mv
        JOIN users u ON u.id = mv.user_id
        ORDER BY mv.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}
