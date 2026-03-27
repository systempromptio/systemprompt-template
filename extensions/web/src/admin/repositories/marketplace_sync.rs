use std::path::{Path, PathBuf};
use std::sync::Arc;

use sqlx::PgPool;

use systemprompt::identifiers::{SkillId, UserId};

use crate::admin::types::{ParsedSkill, SyncDiff, UserSkill};
use crate::error::MarketplaceError;

pub use super::marketplace_sync_archive::extract_archive;
pub use super::marketplace_sync_parse::parse_skills_from_directory;

pub async fn compute_skill_diff(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    uploaded: &[ParsedSkill],
) -> Result<SyncDiff, MarketplaceError> {
    let existing = super::user_skills::list_user_skills(pool, user_id).await?;

    let existing_map: std::collections::HashMap<&str, &UserSkill> =
        existing.iter().map(|s| (s.skill_id.as_str(), s)).collect();

    let uploaded_map: std::collections::HashMap<&str, &ParsedSkill> =
        uploaded.iter().map(|s| (s.skill_id.as_str(), s)).collect();

    let mut added = Vec::new();
    let mut updated = Vec::new();

    for skill in uploaded {
        if let Some(existing_skill) = existing_map.get(skill.skill_id.as_str()) {
            let mut changes = Vec::new();
            if existing_skill.content.trim() != skill.content.trim() {
                changes.push("content changed");
            }
            if existing_skill.name != skill.name {
                changes.push("name changed");
            }
            if existing_skill.description != skill.description {
                changes.push("description changed");
            }
            if !changes.is_empty() {
                updated.push((skill.clone(), changes.join(", ")));
            }
        } else {
            added.push(skill.clone());
        }
    }

    let deleted: Vec<(SkillId, String)> = existing
        .iter()
        .filter(|s| !uploaded_map.contains_key(s.skill_id.as_str()))
        .map(|s| (s.skill_id.clone(), s.name.clone()))
        .collect();

    Ok(SyncDiff {
        added,
        updated,
        deleted,
    })
}

pub async fn apply_sync_diff(
    pool: &PgPool,
    user_id: &UserId,
    diff: &SyncDiff,
) -> Result<(), MarketplaceError> {
    let mut tx = pool.begin().await?;

    for skill in &diff.added {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query!(
            r"
            INSERT INTO user_skills (id, user_id, skill_id, name, description, content, tags, base_skill_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ",
            id,
            user_id.as_str(),
            skill.skill_id.as_str(),
            skill.name,
            skill.description,
            skill.content,
            &skill.tags as &[String],
            skill.base_skill_id.as_ref().map(SkillId::as_str),
        )
        .execute(&mut *tx)
        .await?;
    }

    for (skill, _detail) in &diff.updated {
        sqlx::query!(
            r"
            UPDATE user_skills
            SET name = $3, description = $4, content = $5, base_skill_id = $6, updated_at = NOW()
            WHERE user_id = $1 AND skill_id = $2
            ",
            user_id.as_str(),
            skill.skill_id.as_str(),
            skill.name,
            skill.description,
            skill.content,
            skill.base_skill_id.as_ref().map(SkillId::as_str),
        )
        .execute(&mut *tx)
        .await?;
    }

    for (skill_id, _name) in &diff.deleted {
        sqlx::query!(
            "DELETE FROM user_skills WHERE user_id = $1 AND skill_id = $2",
            user_id.as_str(),
            skill_id.as_str(),
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn snapshot_current_skills(
    pool: &Arc<PgPool>,
    user_id: &UserId,
) -> Result<serde_json::Value, MarketplaceError> {
    let skills = super::user_skills::list_user_skills(pool, user_id).await?;
    Ok(serde_json::to_value(&skills)?)
}

pub async fn restore_skills_from_snapshot(
    pool: &PgPool,
    user_id: &UserId,
    snapshot: &serde_json::Value,
) -> Result<usize, MarketplaceError> {
    let skills: Vec<UserSkill> = serde_json::from_value(snapshot.clone())?;

    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM user_skills WHERE user_id = $1",
        user_id.as_str(),
    )
    .execute(&mut *tx)
    .await?;

    for skill in &skills {
        sqlx::query!(
            r"
            INSERT INTO user_skills (id, user_id, skill_id, name, description, content, enabled, version, tags, base_skill_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ",
            skill.id,
            user_id.as_str(),
            skill.skill_id.as_str(),
            skill.name,
            skill.description,
            skill.content,
            skill.enabled,
            skill.version,
            &skill.tags as &[String],
            skill.base_skill_id.as_ref().map(SkillId::as_str),
            skill.created_at,
            skill.updated_at,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(skills.len())
}

pub fn invalidate_git_cache(user_id: &UserId) -> Result<(), std::io::Error> {
    let cache_dir = PathBuf::from(super::marketplace_git::CACHE_DIR).join(user_id.as_str());
    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir)?;
    }
    Ok(())
}

pub fn save_upload_archive(
    user_id: &UserId,
    version_number: i32,
    data: &[u8],
) -> Result<String, MarketplaceError> {
    let dir = PathBuf::from("storage/marketplace-versions").join(user_id.as_str());
    std::fs::create_dir_all(&dir)?;

    let filename = format!("{version_number}.tar.gz");
    let path = dir.join(&filename);
    std::fs::write(&path, data)?;

    Ok(path.to_string_lossy().to_string())
}

pub fn delete_snapshot_file(snapshot_path: &str) {
    let path = Path::new(snapshot_path);
    if !path.is_file() {
        return;
    }
    if let Err(e) = std::fs::remove_file(path) {
        tracing::warn!(path = %snapshot_path, error = %e, "Failed to delete version snapshot file");
    }
}
