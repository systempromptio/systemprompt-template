//! Git commits observed through Claude Code Bash tool calls.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone, Copy)]
pub struct NewUserCommit<'a> {
    pub user_id: &'a UserId,
    pub session_id: &'a SessionId,
    pub cwd: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub commit_hash: &'a str,
    pub message: &'a str,
    pub files_changed: Option<i32>,
    pub insertions: Option<i32>,
    pub deletions: Option<i32>,
}

// Why: duplicate hook deliveries of the same commit collapse on the
// `(user_id, cwd, commit_hash)` unique index — false is "already recorded",
// not an error.
pub async fn insert_user_commit(
    pool: &PgPool,
    params: &NewUserCommit<'_>,
) -> Result<bool, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let result = sqlx::query!(
        "INSERT INTO user_commits
            (id, user_id, session_id, cwd, branch, commit_hash, message,
             files_changed, insertions, deletions)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (user_id, COALESCE(cwd, ''), commit_hash) DO NOTHING",
        &id,
        params.user_id.as_str(),
        params.session_id.as_str(),
        params.cwd,
        params.branch,
        params.commit_hash,
        params.message,
        params.files_changed,
        params.insertions,
        params.deletions,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
