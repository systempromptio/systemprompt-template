use sqlx::PgPool;

use crate::admin::types::{
    AllVersionsSummaryRow, MarketplaceChangelogEntry, MarketplaceVersion,
    MarketplaceVersionSummary, NewChangelogEntry,
};

pub async fn get_latest_version_number(pool: &PgPool, user_id: &str) -> Result<i32, sqlx::Error> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT COALESCE(MAX(version_number), 0) FROM marketplace_versions WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map_or(0, |r| r.0))
}

pub async fn create_marketplace_version(
    pool: &PgPool,
    user_id: &str,
    version_number: i32,
    version_type: &str,
    snapshot_path: &str,
    skills_snapshot: &serde_json::Value,
) -> Result<MarketplaceVersion, sqlx::Error> {
    sqlx::query_as::<_, MarketplaceVersion>(
        r"
        INSERT INTO marketplace_versions (id, user_id, version_number, version_type, snapshot_path, skills_snapshot)
        VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5)
        RETURNING id, user_id, version_number, version_type, snapshot_path, skills_snapshot, created_at
        ",
    )
    .bind(user_id)
    .bind(version_number)
    .bind(version_type)
    .bind(snapshot_path)
    .bind(skills_snapshot)
    .fetch_one(pool)
    .await
}

pub async fn get_marketplace_version(
    pool: &PgPool,
    user_id: &str,
    version_id: &str,
) -> Result<Option<MarketplaceVersion>, sqlx::Error> {
    sqlx::query_as::<_, MarketplaceVersion>(
        r"
        SELECT id, user_id, version_number, version_type, snapshot_path, skills_snapshot, created_at
        FROM marketplace_versions
        WHERE user_id = $1 AND id = $2
        ",
    )
    .bind(user_id)
    .bind(version_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_marketplace_versions(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<MarketplaceVersionSummary>, sqlx::Error> {
    sqlx::query_as::<_, MarketplaceVersionSummary>(
        r"
        SELECT id, user_id, version_number, version_type, snapshot_path,
               COALESCE(jsonb_array_length(skills_snapshot), 0) AS skills_count,
               COALESCE(
                   (SELECT jsonb_agg(jsonb_build_object('skill_id', elem->>'skill_id', 'name', elem->>'name'))
                    FROM jsonb_array_elements(skills_snapshot) AS elem),
                   '[]'::jsonb
               ) AS skill_names,
               created_at
        FROM marketplace_versions
        WHERE user_id = $1
        ORDER BY version_number DESC
        LIMIT 3
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn prune_old_versions(
    pool: &PgPool,
    user_id: &str,
    keep_count: i32,
) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
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
    )
    .bind(user_id)
    .bind(keep_count)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn insert_changelog_entries(
    pool: &PgPool,
    entries: &[NewChangelogEntry],
) -> Result<Vec<MarketplaceChangelogEntry>, sqlx::Error> {
    let mut results = Vec::with_capacity(entries.len());

    for entry in entries {
        let row = sqlx::query_as::<_, MarketplaceChangelogEntry>(
            r"
            INSERT INTO marketplace_changelog (id, user_id, version_id, action, skill_id, skill_name, detail)
            VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, version_id, action, skill_id, skill_name, detail, created_at
            ",
        )
        .bind(&entry.user_id)
        .bind(&entry.version_id)
        .bind(&entry.action)
        .bind(&entry.skill_id)
        .bind(&entry.skill_name)
        .bind(&entry.detail)
        .fetch_one(pool)
        .await?;

        results.push(row);
    }

    Ok(results)
}

pub async fn list_changelog(
    pool: &PgPool,
    user_id: &str,
    limit: i64,
) -> Result<Vec<MarketplaceChangelogEntry>, sqlx::Error> {
    sqlx::query_as::<_, MarketplaceChangelogEntry>(
        r"
        SELECT id, user_id, version_id, action, skill_id, skill_name, detail, created_at
        FROM marketplace_changelog
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        ",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_all_versions_summary(
    pool: &PgPool,
) -> Result<Vec<AllVersionsSummaryRow>, sqlx::Error> {
    sqlx::query_as::<_, AllVersionsSummaryRow>(
        r"
        SELECT mv.id, mv.user_id, u.email, u.display_name,
               mv.version_number, mv.version_type,
               COALESCE(jsonb_array_length(mv.skills_snapshot), 0) AS skills_count,
               COALESCE(
                   (SELECT jsonb_agg(jsonb_build_object('skill_id', elem->>'skill_id', 'name', elem->>'name', 'base_skill_id', elem->>'base_skill_id'))
                    FROM jsonb_array_elements(mv.skills_snapshot) AS elem),
                   '[]'::jsonb
               ) AS skill_names,
               mv.created_at
        FROM marketplace_versions mv
        JOIN users u ON u.id = mv.user_id
        ORDER BY mv.created_at DESC
        ",
    )
    .fetch_all(pool)
    .await
}
