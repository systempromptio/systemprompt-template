use crate::error::MarketplaceError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SkillFile {
    pub id: String,
    pub skill_id: String,
    pub file_path: String,
    pub content: String,
    pub category: String,
    pub language: String,
    pub executable: bool,
    pub size_bytes: i64,
    pub checksum: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct SyncResult {
    pub created: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub skills_scanned: usize,
}

pub async fn list_skill_files(
    pool: &PgPool,
    skill_id: &str,
) -> Result<Vec<SkillFile>, MarketplaceError> {
    let rows = sqlx::query_as!(
        SkillFile,
        "SELECT id, skill_id, file_path, content, category, language, executable, size_bytes, checksum, created_at, updated_at FROM skill_files WHERE skill_id = $1 ORDER BY category, file_path",
        skill_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_skill_file(
    pool: &PgPool,
    skill_id: &str,
    file_path: &str,
) -> Result<Option<SkillFile>, MarketplaceError> {
    let row = sqlx::query_as!(
        SkillFile,
        "SELECT id, skill_id, file_path, content, category, language, executable, size_bytes, checksum, created_at, updated_at FROM skill_files WHERE skill_id = $1 AND file_path = $2",
        skill_id,
        file_path,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn update_skill_file_content(
    pool: &PgPool,
    skill_id: &str,
    file_path: &str,
    content: &str,
    services_path: &Path,
) -> Result<bool, MarketplaceError> {
    let size_bytes = i64::try_from(content.len()).unwrap_or(0);
    let checksum = compute_checksum(content);

    let result = sqlx::query!(
        r"UPDATE skill_files
          SET content = $3, size_bytes = $4, checksum = $5, updated_at = NOW()
          WHERE skill_id = $1 AND file_path = $2",
        skill_id,
        file_path,
        content,
        size_bytes,
        checksum,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        let fs_path = services_path.join("skills").join(skill_id).join(file_path);
        if let Some(parent) = fs_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&fs_path, content).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn sync_skill_files(
    pool: &PgPool,
    services_path: &Path,
) -> Result<SyncResult, MarketplaceError> {
    let skills_dir = services_path.join("skills");
    let mut result = SyncResult {
        created: 0,
        updated: 0,
        unchanged: 0,
        skills_scanned: 0,
    };

    if !skills_dir.exists() {
        return Ok(result);
    }

    let mut skill_entries = tokio::fs::read_dir(&skills_dir).await?;
    while let Some(entry) = skill_entries.next_entry().await? {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        let skill_id = match entry_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        result.skills_scanned += 1;

        for walk_entry in WalkDir::new(&entry_path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "__pycache__"
            })
            .filter_map(Result::ok)
        {
            let path = walk_entry.path();
            if !path.is_file() {
                continue;
            }
            if !is_text_file(path) {
                continue;
            }

            let rel_path = match path.strip_prefix(&entry_path) {
                Ok(p) => p.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };

            let Ok(content) = tokio::fs::read_to_string(path).await else {
                continue;
            };

            upsert_skill_file(pool, &skill_id, &rel_path, &content, path, &mut result).await?;
        }
    }

    Ok(result)
}

async fn upsert_skill_file(
    pool: &PgPool,
    skill_id: &str,
    rel_path: &str,
    content: &str,
    path: &Path,
    result: &mut SyncResult,
) -> Result<(), MarketplaceError> {
    let checksum = compute_checksum(content);
    let category = detect_category(rel_path);
    let filename = match path.file_name() {
        Some(name) => name.to_str().unwrap_or(""),
        None => "",
    };
    let language = detect_language(filename);
    let size_bytes = i64::try_from(content.len()).unwrap_or(0);
    let executable = language == "python" || language == "bash";
    let id = uuid::Uuid::new_v4().to_string();

    let query_result = sqlx::query!(
        r"INSERT INTO skill_files (id, skill_id, file_path, content, category, language, executable, size_bytes, checksum)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
          ON CONFLICT (skill_id, file_path) DO UPDATE
          SET content = EXCLUDED.content,
              category = EXCLUDED.category,
              language = EXCLUDED.language,
              executable = EXCLUDED.executable,
              size_bytes = EXCLUDED.size_bytes,
              checksum = EXCLUDED.checksum,
              updated_at = NOW()
          WHERE skill_files.checksum != EXCLUDED.checksum",
        id,
        skill_id,
        rel_path,
        content,
        category,
        language,
        executable,
        size_bytes,
        checksum,
    )
    .execute(pool)
    .await?;

    if query_result.rows_affected() == 0 {
        result.unchanged += 1;
    } else {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM skill_files WHERE id = $1) as \"exists!\"",
            id,
        )
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if exists {
            result.created += 1;
        } else {
            result.updated += 1;
        }
    }
    Ok(())
}

fn detect_category(rel_path: &str) -> &'static str {
    let first_component = rel_path.split('/').next().unwrap_or("");
    match first_component {
        "scripts" => "script",
        "references" => "reference",
        "templates" => "template",
        "diagnostics" => "diagnostic",
        "data" => "data",
        "assets" => "asset",
        _ => "config",
    }
}

fn detect_language(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("");
    match ext {
        "py" => "python",
        "sh" => "bash",
        "md" => "markdown",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "toml" => "toml",
        "txt" => "text",
        _ => "",
    }
}

fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn is_text_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    !matches!(
        ext.as_str(),
        "pyc"
            | "pyo"
            | "so"
            | "dll"
            | "exe"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "bmp"
            | "ico"
            | "ttf"
            | "woff"
            | "woff2"
            | "eot"
            | "zip"
            | "tar"
            | "gz"
            | "bz2"
            | "7z"
            | "rar"
            | "pdf"
            | "doc"
            | "docx"
            | "xls"
            | "xlsx"
    )
}
