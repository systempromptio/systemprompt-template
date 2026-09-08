//! Group membership, which has two writers that must not overwrite each
//! other.
//!
//! The directory replaces every `adfs` row at each sign-in — leaving an AD
//! group has to actually remove the membership — while `manual` rows an admin
//! added survive that replace. The primary key carries `source`, so both can
//! hold the same pair and a member row reports the set of sources behind it.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::types::groups::GroupMemberRow;

pub async fn list_group_members(
    pool: &PgPool,
    group_id: &str,
) -> Result<Vec<GroupMemberRow>, sqlx::Error> {
    sqlx::query_as!(
        GroupMemberRow,
        r#"
        SELECT
            ug.user_id AS "user_id!: UserId",
            u.display_name AS "display_name?",
            u.email AS "email?",
            COALESCE(
                (SELECT ARRAY_AGG(DISTINCT gm.source) FROM group_members gm
                 WHERE gm.group_id = ug.group_id AND gm.user_id = ug.user_id),
                ARRAY['derived']::TEXT[]
            ) AS "sources!",
            COALESCE(
                (SELECT ARRAY_AGG(DISTINCT gm.source_ad_group) FROM group_members gm
                 WHERE gm.group_id = ug.group_id AND gm.user_id = ug.user_id
                   AND gm.source_ad_group IS NOT NULL),
                ARRAY[]::TEXT[]
            ) AS "source_ad_groups!"
        FROM user_groups ug
        JOIN users u ON u.id = ug.user_id
        WHERE ug.group_id = $1
        ORDER BY u.display_name NULLS LAST, ug.user_id
        "#,
        group_id
    )
    .fetch_all(pool)
    .await
}

// Why: reads the `user_groups` view, not the table, so a user with no rows
// resolves to `unassigned` here exactly as they do in a listing. The
// `group` authz dimension binds to this, so the two cannot disagree.
pub async fn list_group_ids_for_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT group_id AS "group_id!" FROM user_groups WHERE user_id = $1 ORDER BY group_id"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

// Why: the AD groups that produced this user's directory memberships, read
// off the membership rows they created. The bridge shows these on its account
// card, so a person can see which directory group placed them.
pub async fn list_source_ad_groups(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT DISTINCT source_ad_group AS "source_ad_group!" FROM group_members
           WHERE user_id = $1 AND source = 'adfs' AND source_ad_group IS NOT NULL
           ORDER BY 1"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_unassigned_users(pool: &PgPool) -> Result<Vec<UserId>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT user_id AS "user_id!: UserId" FROM user_groups
           WHERE group_id = 'unassigned' ORDER BY user_id"#
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_group_member(
    pool: &PgPool,
    group_id: &str,
    user_id: &UserId,
    granted_by: &UserId,
) -> AdminResult<()> {
    let inserted = sqlx::query!(
        "INSERT INTO group_members (group_id, user_id, source, granted_by)
         VALUES ($1, $2, 'manual', $3) ON CONFLICT DO NOTHING",
        group_id,
        user_id.as_str(),
        granted_by.as_str()
    )
    .execute(pool)
    .await?;
    if inserted.rows_affected() == 0 {
        return Err(AdminError::Conflict(format!(
            "User {user_id} is already a manual member of {group_id}"
        )));
    }
    Ok(())
}

// Why: a directory-sourced membership cannot be removed here. It would
// reappear at the member's next sign-in, so refusing says what is actually
// true — the change belongs in AD.
pub async fn delete_group_member(
    pool: &PgPool,
    group_id: &str,
    user_id: &UserId,
) -> AdminResult<()> {
    let sources = sqlx::query_scalar!(
        r#"SELECT source AS "source!" FROM group_members WHERE group_id = $1 AND user_id = $2"#,
        group_id,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await?;

    if sources.is_empty() {
        return Err(AdminError::NotFound(format!(
            "User {user_id} is not a member of {group_id}"
        )));
    }
    if !sources.iter().any(|s| s == "manual") {
        return Err(AdminError::Conflict(format!(
            "User {user_id} is in {group_id} through the directory; remove them from the AD group"
        )));
    }
    sqlx::query!(
        "DELETE FROM group_members WHERE group_id = $1 AND user_id = $2 AND source = 'manual'",
        group_id,
        user_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: Replace this user's directory memberships with what the assertion's AD
// groups map to.
//
// Why replace and not union: an AD group the user has left must stop
// granting anything, and the assertion is the whole truth about their
// current directory membership. Manual rows are untouched, which is the
// point of keeping `source` in the key.
pub async fn replace_directory_group_memberships(
    pool: &PgPool,
    user_id: &UserId,
    ad_groups: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM group_members WHERE user_id = $1 AND source = 'adfs'",
        user_id.as_str()
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO group_members (group_id, user_id, source, source_ad_group)
         SELECT m.group_id, $1, 'adfs', m.ad_group FROM group_ad_mappings m
         WHERE m.ad_group = ANY($2) ON CONFLICT DO NOTHING",
        user_id.as_str(),
        ad_groups
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    crate::repositories::scope::defaults::recompute_scope_defaults_for_user(pool, user_id).await?;
    Ok(())
}
